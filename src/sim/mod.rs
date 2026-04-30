use std::collections::BTreeMap;
use std::fmt::Write;

use crate::autoplay::{
    CombatChoice, RestChoice, RewardChoice, ShopChoice, choose_boss_module_reward,
    choose_combat_action as choose_autoplay_combat_action,
    pick_event_choice as pick_autoplay_event_choice, pick_map_node as pick_autoplay_map_node,
    pick_party_map_node, pick_rest_choice, pick_reward_choice, pick_shop_choice,
    pick_starter_module as pick_autoplay_starter_module,
};
use crate::combat::{Actor, CombatAction, CombatOutcome, CombatState, EncounterSetup};
use crate::content::{EventId, ModuleId, RewardTier, reward_choices, shop_offers};
use crate::dungeon::{DungeonProgress, DungeonRun, RoomKind};
use crate::party::{PartyCombatState, PartyRunState};
use crate::run_logic::{apply_post_victory_modules, combat_seed_for_dungeon};

const MAX_RUN_STEPS: usize = 512;
const MAX_COMBAT_TURNS: u32 = 100;
const MAX_ACTIONS_PER_TURN: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimulationConfig {
    pub runs: usize,
    pub seed_start: u64,
    pub players: usize,
    pub verbose: bool,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            runs: 1000,
            seed_start: 1,
            players: 1,
            verbose: false,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SimulationStats {
    pub players: usize,
    pub runs: usize,
    pub wins: usize,
    pub losses: usize,
    pub aborts: usize,
    pub total_combats_cleared: usize,
    pub total_elites_cleared: usize,
    pub total_bosses_cleared: usize,
    pub total_victory_hp: i32,
    pub total_surviving_heroes_on_victory: usize,
    pub party_wipes: usize,
    pub defeat_by_level: BTreeMap<usize, usize>,
    pub defeat_by_room: BTreeMap<&'static str, usize>,
    pub hero_deaths_by_slot: BTreeMap<usize, usize>,
    pub module_picks: BTreeMap<&'static str, usize>,
    pub abort_reasons: BTreeMap<&'static str, usize>,
}

impl SimulationStats {
    pub fn win_rate(&self) -> f64 {
        if self.runs == 0 {
            0.0
        } else {
            self.wins as f64 / self.runs as f64
        }
    }

    pub fn average_combats_cleared(&self) -> f64 {
        average(self.total_combats_cleared as f64, self.runs)
    }

    pub fn average_elites_cleared(&self) -> f64 {
        average(self.total_elites_cleared as f64, self.runs)
    }

    pub fn average_bosses_cleared(&self) -> f64 {
        average(self.total_bosses_cleared as f64, self.runs)
    }

    pub fn average_victory_hp(&self) -> f64 {
        average(self.total_victory_hp as f64, self.wins)
    }

    pub fn average_surviving_heroes_on_victory(&self) -> f64 {
        average(self.total_surviving_heroes_on_victory as f64, self.wins)
    }

    pub fn render_report(&self) -> String {
        let mut report = String::new();
        let _ = writeln!(report, "players: {}", self.players);
        let _ = writeln!(report, "runs: {}", self.runs);
        let _ = writeln!(report, "wins: {}", self.wins);
        let _ = writeln!(report, "losses: {}", self.losses);
        let _ = writeln!(report, "aborts: {}", self.aborts);
        let _ = writeln!(report, "win rate: {:.1}%", self.win_rate() * 100.0);
        let _ = writeln!(
            report,
            "avg combats cleared: {:.2}",
            self.average_combats_cleared()
        );
        let _ = writeln!(
            report,
            "avg elites cleared: {:.2}",
            self.average_elites_cleared()
        );
        let _ = writeln!(
            report,
            "avg bosses cleared: {:.2}",
            self.average_bosses_cleared()
        );
        let _ = writeln!(report, "avg victory HP: {:.2}", self.average_victory_hp());
        if self.players > 1 {
            let _ = writeln!(
                report,
                "avg surviving heroes on victory: {:.2}",
                self.average_surviving_heroes_on_victory()
            );
            let _ = writeln!(report, "party wipes: {}", self.party_wipes);
            append_breakdown(
                &mut report,
                "hero deaths by slot",
                &self.hero_deaths_by_slot,
            );
        }
        append_breakdown(&mut report, "defeats by level", &self.defeat_by_level);
        append_breakdown(&mut report, "defeats by room", &self.defeat_by_room);
        append_breakdown(&mut report, "module picks", &self.module_picks);
        if !self.abort_reasons.is_empty() {
            append_breakdown(&mut report, "abort reasons", &self.abort_reasons);
        }
        report
    }

    fn record(&mut self, run: &RunRecord) {
        self.players = self.players.max(run.party_size);
        self.runs += 1;
        self.total_combats_cleared += run.combats_cleared;
        self.total_elites_cleared += run.elites_cleared;
        self.total_bosses_cleared += run.bosses_cleared;

        for &module in &run.modules {
            *self.module_picks.entry(module_label(module)).or_default() += 1;
        }

        match run.outcome {
            RunOutcome::Victory => {
                self.wins += 1;
                self.total_victory_hp += run.player_hp;
                self.total_surviving_heroes_on_victory += run.surviving_heroes;
            }
            RunOutcome::Defeat => {
                self.losses += 1;
                if run.surviving_heroes == 0 && run.party_size > 1 {
                    self.party_wipes += 1;
                }
                *self.defeat_by_level.entry(run.current_level).or_default() += 1;
                if let Some(room) = run.final_room {
                    *self
                        .defeat_by_room
                        .entry(room_kind_label(room))
                        .or_default() += 1;
                }
            }
            RunOutcome::Abort(reason) => {
                self.aborts += 1;
                *self.abort_reasons.entry(reason).or_default() += 1;
            }
        }

        for (slot, &hp) in run.hero_hps.iter().enumerate() {
            if hp <= 0 {
                *self.hero_deaths_by_slot.entry(slot).or_default() += 1;
            }
        }
    }
}

pub fn run_simulations(config: &SimulationConfig) -> SimulationStats {
    let mut stats = SimulationStats::default();

    for offset in 0..config.runs {
        let seed = config.seed_start.wrapping_add(offset as u64);
        let run = if config.players > 1 {
            simulate_party_run(seed, config.players)
        } else {
            simulate_run(seed)
        };
        if config.verbose {
            println!("{}", run.summary_line());
        }
        stats.record(&run);
    }

    stats
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RunOutcome {
    Victory,
    Defeat,
    Abort(&'static str),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RunRecord {
    seed: u64,
    party_size: usize,
    outcome: RunOutcome,
    current_level: usize,
    final_room: Option<RoomKind>,
    player_hp: i32,
    player_max_hp: i32,
    deck_count: usize,
    surviving_heroes: usize,
    hero_hps: Vec<i32>,
    combats_cleared: usize,
    elites_cleared: usize,
    bosses_cleared: usize,
    modules: Vec<ModuleId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SimulationAdvance {
    Continue,
    Victory(Option<RoomKind>),
    Defeat(RoomKind),
    Abort(&'static str, Option<RoomKind>),
}

fn simulate_run(seed: u64) -> RunRecord {
    let mut dungeon = DungeonRun::new(seed);
    if let Some(module) = pick_autoplay_starter_module(&dungeon) {
        dungeon.add_module(module);
    }

    for _ in 0..MAX_RUN_STEPS {
        if dungeon.available_nodes.is_empty() {
            return build_run_record(seed, RunOutcome::Victory, None, &dungeon);
        }

        let Some(node_id) = pick_autoplay_map_node(&dungeon) else {
            return build_run_record(
                seed,
                RunOutcome::Abort("No available map node."),
                dungeon.current_room_kind(),
                &dungeon,
            );
        };
        let room_kind = dungeon.node(node_id).map(|node| node.kind);
        let selection = dungeon.select_node(node_id);

        let advance = match selection {
            Some(crate::dungeon::NodeSelection::Encounter(setup)) => {
                simulate_encounter(&mut dungeon, setup)
            }
            Some(crate::dungeon::NodeSelection::Rest) => simulate_rest(&mut dungeon),
            Some(crate::dungeon::NodeSelection::Shop) => simulate_shop(&mut dungeon),
            Some(crate::dungeon::NodeSelection::Event(event)) => {
                simulate_event(&mut dungeon, event)
            }
            None => {
                return build_run_record(
                    seed,
                    RunOutcome::Abort("Failed to select map node."),
                    room_kind,
                    &dungeon,
                );
            }
        };

        match advance {
            SimulationAdvance::Continue => {}
            SimulationAdvance::Victory(final_room) => {
                return build_run_record(seed, RunOutcome::Victory, final_room, &dungeon);
            }
            SimulationAdvance::Defeat(final_room) => {
                return build_run_record(seed, RunOutcome::Defeat, Some(final_room), &dungeon);
            }
            SimulationAdvance::Abort(reason, final_room) => {
                return build_run_record(seed, RunOutcome::Abort(reason), final_room, &dungeon);
            }
        }
    }

    build_run_record(
        seed,
        RunOutcome::Abort("Exceeded maximum run steps."),
        dungeon.current_room_kind(),
        &dungeon,
    )
}

fn simulate_party_run(seed: u64, players: usize) -> RunRecord {
    let party_size = players.clamp(1, 2);
    let mut party = PartyRunState::new(seed, party_size);
    for slot in 0..party.party_size() {
        let Some(dungeon) = party.active_dungeon(slot) else {
            continue;
        };
        if let Some(module) = pick_autoplay_starter_module(&dungeon) {
            party.add_module(slot, module);
        }
    }

    for _ in 0..MAX_RUN_STEPS {
        let Some(lead_dungeon) = party.active_dungeon(0) else {
            return build_party_run_record(
                seed,
                RunOutcome::Abort("Party dungeon was unavailable."),
                None,
                &party,
            );
        };
        if lead_dungeon.available_nodes.is_empty() {
            return build_party_run_record(seed, RunOutcome::Victory, None, &party);
        }

        let Some(node_id) = pick_party_map_node(&party) else {
            return build_party_run_record(
                seed,
                RunOutcome::Abort("No available party map node."),
                lead_dungeon.current_room_kind(),
                &party,
            );
        };
        let room_kind = lead_dungeon.node(node_id).map(|node| node.kind);
        let selection = party.select_node(node_id, false);

        let advance = match selection {
            Some(crate::dungeon::NodeSelection::Encounter(_)) => {
                simulate_party_encounter(&mut party)
            }
            Some(crate::dungeon::NodeSelection::Rest) => simulate_party_rest(&mut party),
            Some(crate::dungeon::NodeSelection::Shop) => simulate_party_shop(&mut party),
            Some(crate::dungeon::NodeSelection::Event(event)) => {
                simulate_party_event(&mut party, event)
            }
            None => {
                return build_party_run_record(
                    seed,
                    RunOutcome::Abort("Failed to select party map node."),
                    room_kind,
                    &party,
                );
            }
        };

        match advance {
            SimulationAdvance::Continue => {}
            SimulationAdvance::Victory(final_room) => {
                return build_party_run_record(seed, RunOutcome::Victory, final_room, &party);
            }
            SimulationAdvance::Defeat(final_room) => {
                return build_party_run_record(seed, RunOutcome::Defeat, Some(final_room), &party);
            }
            SimulationAdvance::Abort(reason, final_room) => {
                return build_party_run_record(seed, RunOutcome::Abort(reason), final_room, &party);
            }
        }
    }

    build_party_run_record(
        seed,
        RunOutcome::Abort("Exceeded maximum run steps."),
        party.current_room_kind(),
        &party,
    )
}

fn simulate_party_encounter(party: &mut PartyRunState) -> SimulationAdvance {
    let room_kind = party.current_room_kind().unwrap_or(RoomKind::Combat);
    let Some(seed_dungeon) = party.active_dungeon(0) else {
        return SimulationAdvance::Abort("Party combat dungeon was unavailable.", Some(room_kind));
    };
    let seed = combat_seed_for_dungeon(&seed_dungeon);
    let slot_count = party.party_size();
    let enemy_hp_multiplier = slot_count.max(1) as i32;
    let setups: Vec<_> = (0..slot_count)
        .map(|slot| {
            let mut slot_setup = party
                .current_encounter_setup(slot)
                .or_else(|| party.current_encounter_setup(0))
                .expect("party encounter setup");
            for enemy in &mut slot_setup.enemies {
                enemy.hp = enemy.hp.saturating_mul(enemy_hp_multiplier);
                enemy.max_hp = enemy.max_hp.saturating_mul(enemy_hp_multiplier);
            }
            slot_setup
        })
        .collect();
    let decks: Vec<_> = (0..slot_count)
        .map(|slot| {
            party
                .hero(slot)
                .map(|hero| hero.deck.clone())
                .unwrap_or_else(crate::content::starter_deck)
        })
        .collect();
    let modules: Vec<_> = (0..slot_count)
        .map(|slot| {
            party
                .hero(slot)
                .map(|hero| hero.modules.clone())
                .unwrap_or_default()
        })
        .collect();
    let Some(mut party_combat) = PartyCombatState::new(seed, &setups, &decks, &modules) else {
        return SimulationAdvance::Abort("Could not initialize party combat.", Some(room_kind));
    };
    for slot in 0..slot_count {
        if party
            .hero(slot)
            .map(|hero| hero.player_hp <= 0)
            .unwrap_or(false)
        {
            if let Some(hero) = party_combat.heroes.get_mut(slot) {
                hero.player.fighter.hp = 0;
                hero.ready = true;
            }
        }
    }

    loop {
        match party_combat.outcome() {
            Some(CombatOutcome::Victory) => {
                let player_hps = party_combat
                    .heroes
                    .iter()
                    .map(|hero| hero.player.fighter.hp)
                    .collect::<Vec<_>>();
                let Some((progress, _credits_gained)) =
                    party.resolve_combat_victory_all(&player_hps)
                else {
                    return SimulationAdvance::Abort(
                        "Party combat victory could not resolve dungeon progress.",
                        Some(room_kind),
                    );
                };
                for slot in 0..slot_count {
                    let _ = party.apply_post_victory_modules(slot);
                }

                if let Some((tier, reward_seed)) = reward_context_for_party_room(party) {
                    if matches!(progress, DungeonProgress::Continue) {
                        resolve_party_reward_phase(party, tier, reward_seed);
                        return SimulationAdvance::Continue;
                    }
                    return SimulationAdvance::Victory(Some(room_kind));
                }

                return advance_from_progress(progress, room_kind);
            }
            Some(CombatOutcome::Defeat) => {
                for (slot, hero) in party_combat.heroes.iter().enumerate() {
                    let _ = party.resolve_combat_defeat(slot, hero.player.fighter.hp);
                }
                return SimulationAdvance::Defeat(room_kind);
            }
            None => {}
        }

        if party_combat.turn > MAX_COMBAT_TURNS {
            return SimulationAdvance::Abort("Exceeded combat turn cap.", Some(room_kind));
        }
        if !matches!(party_combat.phase(), crate::combat::TurnPhase::PlayerTurn) {
            return SimulationAdvance::Abort("Combat ended outside player turn.", Some(room_kind));
        }

        for slot in 0..slot_count {
            if !party_combat.hero_is_alive(slot)
                || party_combat.heroes.get(slot).is_some_and(|hero| hero.ready)
            {
                continue;
            }

            let mut actions_this_turn = 0usize;
            loop {
                if actions_this_turn >= MAX_ACTIONS_PER_TURN {
                    return SimulationAdvance::Abort(
                        "Exceeded actions per turn cap.",
                        Some(room_kind),
                    );
                }
                let Some(view) = party_combat.view_for_slot(slot) else {
                    return SimulationAdvance::Abort(
                        "Party combat view was unavailable.",
                        Some(room_kind),
                    );
                };
                match choose_autoplay_combat_action(&view) {
                    CombatChoice::PlayCard {
                        hand_index,
                        target_enemy,
                    } => {
                        if !party_combat.play_card(slot, hand_index, target_enemy) {
                            return SimulationAdvance::Abort(
                                "Party combat rejected autoplay card play.",
                                Some(room_kind),
                            );
                        }
                        actions_this_turn += 1;
                    }
                    CombatChoice::EndTurn => {
                        if !party_combat.ready_hero(slot) {
                            return SimulationAdvance::Abort(
                                "Party combat rejected autoplay end turn.",
                                Some(room_kind),
                            );
                        }
                        break;
                    }
                }

                if party_combat.outcome().is_some()
                    || !matches!(party_combat.phase(), crate::combat::TurnPhase::PlayerTurn)
                    || !party_combat.hero_is_alive(slot)
                {
                    break;
                }
            }

            if party_combat.outcome().is_some()
                || !matches!(party_combat.phase(), crate::combat::TurnPhase::PlayerTurn)
            {
                break;
            }
        }
    }
}

fn simulate_party_rest(party: &mut PartyRunState) -> SimulationAdvance {
    let room_kind = party.current_room_kind().unwrap_or(RoomKind::Rest);
    for slot in alive_party_slots(party) {
        let Some(dungeon) = party.active_dungeon(slot) else {
            continue;
        };
        match pick_rest_choice(&dungeon) {
            Some(RestChoice::Heal) => {
                if party.apply_rest_heal(slot).is_none() {
                    return SimulationAdvance::Abort(
                        "Party rest heal could not resolve.",
                        Some(room_kind),
                    );
                }
            }
            Some(RestChoice::Upgrade(deck_index)) => {
                if party.apply_rest_upgrade(slot, deck_index).is_none() {
                    return SimulationAdvance::Abort(
                        "Party rest upgrade could not resolve.",
                        Some(room_kind),
                    );
                }
            }
            None => {
                return SimulationAdvance::Abort(
                    "No actionable party rest option.",
                    Some(room_kind),
                );
            }
        }
    }
    resolve_progress(
        party.complete_current_node_shared(),
        room_kind,
        "Party rest could not resolve shared progress.",
    )
}

fn simulate_party_shop(party: &mut PartyRunState) -> SimulationAdvance {
    let room_kind = party.current_room_kind().unwrap_or(RoomKind::Shop);
    let Some(seed) = party.current_room_seed() else {
        return SimulationAdvance::Abort("Party shop room seed was unavailable.", Some(room_kind));
    };
    for slot in alive_party_slots(party) {
        let Some(dungeon) = party.active_dungeon(slot) else {
            continue;
        };
        let offers = shop_offers(
            seed ^ ((slot as u64 + 1).wrapping_mul(0x517C_C1B7_2722_0A95)),
            party.current_level(),
        );
        match pick_shop_choice(&dungeon, &offers) {
            ShopChoice::Buy(index) => {
                let Some(offer) = offers.get(index).copied() else {
                    return SimulationAdvance::Abort(
                        "Party shop offer index was invalid.",
                        Some(room_kind),
                    );
                };
                if !party.apply_shop_purchase(slot, offer.card, offer.price) {
                    return SimulationAdvance::Abort(
                        "Party shop purchase could not resolve.",
                        Some(room_kind),
                    );
                }
            }
            ShopChoice::Leave => {}
        }
    }
    resolve_progress(
        party.complete_current_node_shared(),
        room_kind,
        "Party shop leave could not resolve shared progress.",
    )
}

fn simulate_party_event(party: &mut PartyRunState, event: EventId) -> SimulationAdvance {
    let room_kind = party.current_room_kind().unwrap_or(RoomKind::Event);
    for slot in alive_party_slots(party) {
        let Some(dungeon) = party.active_dungeon(slot) else {
            continue;
        };
        let Some(choice_index) = pick_autoplay_event_choice(&dungeon, event) else {
            return SimulationAdvance::Abort("Party event had no valid choices.", Some(room_kind));
        };
        if party
            .apply_event_choice(slot, event, choice_index)
            .is_none()
        {
            return SimulationAdvance::Abort(
                "Party event choice could not resolve.",
                Some(room_kind),
            );
        }
    }
    resolve_progress(
        party.complete_current_node_shared(),
        room_kind,
        "Party event could not resolve shared progress.",
    )
}

fn simulate_encounter(dungeon: &mut DungeonRun, setup: EncounterSetup) -> SimulationAdvance {
    let room_kind = dungeon.current_room_kind().unwrap_or(RoomKind::Combat);
    let (mut combat, _) = CombatState::new_with_deck(
        combat_seed_for_dungeon(dungeon),
        setup,
        dungeon.deck.clone(),
    );
    combat.apply_start_of_combat_modules(&dungeon.modules);

    let mut actions_this_turn = 0usize;
    loop {
        match combat.outcome() {
            Some(CombatOutcome::Victory) => {
                let reward_context = reward_context_for_room(dungeon);
                let Some((progress, _credits_gained)) =
                    dungeon.resolve_combat_victory(combat.player.fighter.hp)
                else {
                    return SimulationAdvance::Abort(
                        "Combat victory could not resolve dungeon progress.",
                        Some(room_kind),
                    );
                };
                let _effects = apply_post_victory_modules(dungeon);

                if let Some((tier, reward_seed)) = reward_context {
                    if matches!(progress, DungeonProgress::Continue) {
                        resolve_reward_phase(dungeon, tier, reward_seed);
                        return SimulationAdvance::Continue;
                    }
                    return SimulationAdvance::Victory(Some(room_kind));
                }

                return advance_from_progress(progress, room_kind);
            }
            Some(CombatOutcome::Defeat) => {
                dungeon.resolve_combat_defeat(combat.player.fighter.hp);
                return SimulationAdvance::Defeat(room_kind);
            }
            None => {}
        }

        if combat.turn > MAX_COMBAT_TURNS {
            return SimulationAdvance::Abort("Exceeded combat turn cap.", Some(room_kind));
        }
        if !combat.is_player_turn() {
            return SimulationAdvance::Abort("Combat ended outside player turn.", Some(room_kind));
        }
        if actions_this_turn >= MAX_ACTIONS_PER_TURN {
            return SimulationAdvance::Abort("Exceeded actions per turn cap.", Some(room_kind));
        }

        let action =
            autoplay_combat_action_to_action(&combat, choose_autoplay_combat_action(&combat));
        combat.dispatch(action);
        if matches!(action, CombatAction::EndTurn) {
            actions_this_turn = 0;
        } else {
            actions_this_turn += 1;
        }
    }
}

fn simulate_rest(dungeon: &mut DungeonRun) -> SimulationAdvance {
    let room_kind = dungeon.current_room_kind().unwrap_or(RoomKind::Rest);
    match pick_rest_choice(dungeon) {
        Some(RestChoice::Heal) => resolve_progress(
            dungeon.resolve_rest_heal().map(|(_, progress)| progress),
            room_kind,
            "Rest heal could not resolve.",
        ),
        Some(RestChoice::Upgrade(deck_index)) => resolve_progress(
            dungeon
                .resolve_rest_upgrade(deck_index)
                .map(|(_, _, progress)| progress),
            room_kind,
            "Rest upgrade could not resolve.",
        ),
        None => SimulationAdvance::Abort("No actionable rest option.", Some(room_kind)),
    }
}

fn simulate_shop(dungeon: &mut DungeonRun) -> SimulationAdvance {
    let room_kind = dungeon.current_room_kind().unwrap_or(RoomKind::Shop);
    let Some(seed) = dungeon.current_room_seed() else {
        return SimulationAdvance::Abort("Shop room seed was unavailable.", Some(room_kind));
    };
    let offers = shop_offers(seed, dungeon.current_level());
    match pick_shop_choice(dungeon, &offers) {
        ShopChoice::Buy(offer_index) => {
            let offer = offers[offer_index];
            resolve_progress(
                dungeon.resolve_shop_purchase(offer.card, offer.price),
                room_kind,
                "Shop purchase could not resolve.",
            )
        }
        ShopChoice::Leave => resolve_progress(
            dungeon.resolve_shop_leave(),
            room_kind,
            "Shop leave could not resolve.",
        ),
    }
}

fn simulate_event(dungeon: &mut DungeonRun, event: EventId) -> SimulationAdvance {
    let room_kind = dungeon.current_room_kind().unwrap_or(RoomKind::Event);
    let Some(choice_index) = pick_autoplay_event_choice(dungeon, event) else {
        return SimulationAdvance::Abort("Event had no valid choices.", Some(room_kind));
    };
    resolve_progress(
        dungeon
            .resolve_event_choice(event, choice_index)
            .map(|(_, progress)| progress),
        room_kind,
        "Event choice could not resolve.",
    )
}

fn build_run_record(
    seed: u64,
    outcome: RunOutcome,
    final_room: Option<RoomKind>,
    dungeon: &DungeonRun,
) -> RunRecord {
    RunRecord {
        seed,
        party_size: 1,
        outcome,
        current_level: dungeon.current_level(),
        final_room,
        player_hp: dungeon.player_hp,
        player_max_hp: dungeon.player_max_hp,
        deck_count: dungeon.deck.len(),
        surviving_heroes: usize::from(dungeon.player_hp > 0),
        hero_hps: vec![dungeon.player_hp],
        combats_cleared: dungeon.combats_cleared,
        elites_cleared: dungeon.elites_cleared,
        bosses_cleared: dungeon.bosses_cleared,
        modules: dungeon.modules.clone(),
    }
}

fn build_party_run_record(
    seed: u64,
    outcome: RunOutcome,
    final_room: Option<RoomKind>,
    party: &PartyRunState,
) -> RunRecord {
    let current_level = party.current_level();
    let hero_hps = (0..party.party_size())
        .map(|slot| party.hero(slot).map(|hero| hero.player_hp).unwrap_or(0))
        .collect::<Vec<_>>();
    let surviving_heroes = hero_hps.iter().filter(|hp| **hp > 0).count();
    let player_hp = hero_hps.iter().copied().sum::<i32>();
    let player_max_hp = (0..party.party_size())
        .map(|slot| party.hero(slot).map(|hero| hero.player_max_hp).unwrap_or(0))
        .sum::<i32>();
    let deck_count = (0..party.party_size())
        .map(|slot| party.hero(slot).map(|hero| hero.deck.len()).unwrap_or(0))
        .sum::<usize>();
    let modules = (0..party.party_size())
        .flat_map(|slot| {
            party
                .hero(slot)
                .map(|hero| hero.modules.clone())
                .unwrap_or_default()
        })
        .collect::<Vec<_>>();

    RunRecord {
        seed,
        party_size: party.party_size(),
        outcome,
        current_level,
        final_room,
        player_hp,
        player_max_hp,
        deck_count,
        surviving_heroes,
        hero_hps,
        combats_cleared: party.shared.combats_cleared,
        elites_cleared: party.shared.elites_cleared,
        bosses_cleared: party.shared.bosses_cleared,
        modules,
    }
}

fn reward_context_for_room(dungeon: &DungeonRun) -> Option<(RewardTier, u64)> {
    let tier = match dungeon.current_room_kind()? {
        RoomKind::Combat => RewardTier::Combat,
        RoomKind::Elite => RewardTier::Elite,
        RoomKind::Boss => RewardTier::Boss,
        RoomKind::Start | RoomKind::Rest | RoomKind::Shop | RoomKind::Event => return None,
    };
    Some((tier, dungeon.current_room_seed()?))
}

fn reward_context_for_party_room(party: &PartyRunState) -> Option<(RewardTier, u64)> {
    let dungeon = party.active_dungeon(0)?;
    reward_context_for_room(&dungeon)
}

fn resolve_reward_phase(dungeon: &mut DungeonRun, tier: RewardTier, seed: u64) {
    let options = reward_choices(seed, tier, dungeon.current_level());
    if let RewardChoice::Pick(index) = pick_reward_choice(dungeon, &options, tier) {
        if let Some(&card) = options.get(index) {
            dungeon.add_card(card);
        }
    }

    if matches!(tier, RewardTier::Boss) {
        let boss_level = dungeon.current_level().saturating_sub(1).max(1);
        if let Some(module) = choose_boss_module_reward(dungeon, boss_level) {
            dungeon.add_module(module);
        }
    }
}

fn autoplay_combat_action_to_action(combat: &CombatState, choice: CombatChoice) -> CombatAction {
    match choice {
        CombatChoice::PlayCard {
            hand_index,
            target_enemy,
        } => {
            let target = if combat.card_targets_all_enemies(hand_index) {
                None
            } else if let Some(enemy_index) = target_enemy {
                Some(Actor::Enemy(enemy_index))
            } else {
                Some(Actor::Player)
            };
            CombatAction::PlayCard { hand_index, target }
        }
        CombatChoice::EndTurn => CombatAction::EndTurn,
    }
}

fn resolve_party_reward_phase(party: &mut PartyRunState, tier: RewardTier, seed: u64) {
    let reward_level = party.current_level();
    for slot in alive_party_slots(party) {
        let Some(dungeon) = party.active_dungeon(slot) else {
            continue;
        };
        let slot_seed = seed ^ ((slot as u64 + 1).wrapping_mul(0x94D0_49BB_1331_11EB));
        let options = reward_choices(slot_seed, tier, reward_level);
        match pick_reward_choice(&dungeon, &options, tier) {
            RewardChoice::Pick(index) => {
                if let Some(&card) = options.get(index) {
                    let _ = party.add_card(slot, card);
                }
            }
            RewardChoice::Skip => {}
        }
    }

    if matches!(tier, RewardTier::Boss) {
        let boss_level = party.current_level().saturating_sub(1).max(1);
        for slot in alive_party_slots(party) {
            let Some(dungeon) = party.active_dungeon(slot) else {
                continue;
            };
            if let Some(module) = choose_boss_module_reward(&dungeon, boss_level) {
                let _ = party.add_module(slot, module);
            }
        }
    }
}

fn alive_party_slots(party: &PartyRunState) -> Vec<usize> {
    (0..party.party_size())
        .filter(|&slot| {
            party
                .hero(slot)
                .map(|hero| hero.player_hp > 0)
                .unwrap_or(false)
        })
        .collect()
}

fn advance_from_progress(progress: DungeonProgress, room_kind: RoomKind) -> SimulationAdvance {
    match progress {
        DungeonProgress::Continue => SimulationAdvance::Continue,
        DungeonProgress::Completed => SimulationAdvance::Victory(Some(room_kind)),
    }
}

fn resolve_progress(
    progress: Option<DungeonProgress>,
    room_kind: RoomKind,
    error: &'static str,
) -> SimulationAdvance {
    match progress {
        Some(progress) => advance_from_progress(progress, room_kind),
        None => SimulationAdvance::Abort(error, Some(room_kind)),
    }
}

fn room_kind_label(kind: RoomKind) -> &'static str {
    match kind {
        RoomKind::Start => "start",
        RoomKind::Combat => "combat",
        RoomKind::Elite => "elite",
        RoomKind::Boss => "boss",
        RoomKind::Rest => "rest",
        RoomKind::Shop => "shop",
        RoomKind::Event => "event",
    }
}

fn module_label(module: ModuleId) -> &'static str {
    match module {
        ModuleId::AegisDrive => "Aegis Drive",
        ModuleId::TargetingRelay => "Targeting Relay",
        ModuleId::Nanoforge => "Nanoforge",
        ModuleId::CapacitorBank => "Capacitor Bank",
        ModuleId::PrismScope => "Prism Scope",
        ModuleId::SalvageLedger => "Salvage Ledger",
        ModuleId::OverclockCore => "Overclock Core",
        ModuleId::SuppressionField => "Suppression Field",
        ModuleId::RecoveryMatrix => "Recovery Matrix",
    }
}

fn append_breakdown<K: std::fmt::Display>(
    report: &mut String,
    title: &str,
    values: &BTreeMap<K, usize>,
) {
    let _ = writeln!(report, "{title}:");
    if values.is_empty() {
        let _ = writeln!(report, "  none");
        return;
    }
    for (key, value) in values {
        let _ = writeln!(report, "  {key}: {value}");
    }
}

fn average(total: f64, count: usize) -> f64 {
    if count == 0 {
        0.0
    } else {
        total / count as f64
    }
}

impl RunRecord {
    fn summary_line(&self) -> String {
        let outcome = match self.outcome {
            RunOutcome::Victory => "victory".to_string(),
            RunOutcome::Defeat => "defeat".to_string(),
            RunOutcome::Abort(reason) => format!("abort ({reason})"),
        };
        let room = self.final_room.map(room_kind_label).unwrap_or("none");
        let modules = if self.modules.is_empty() {
            "none".to_string()
        } else {
            self.modules
                .iter()
                .map(|module| module_label(*module))
                .collect::<Vec<_>>()
                .join(", ")
        };
        format!(
            "seed={} players={} result={} level={} room={} hp={}/{} survivors={} deck={} combats={} elites={} bosses={} modules=[{}]",
            self.seed,
            self.party_size,
            outcome,
            self.current_level,
            room,
            self.player_hp,
            self.player_max_hp,
            self.surviving_heroes,
            self.deck_count,
            self.combats_cleared,
            self.elites_cleared,
            self.bosses_cleared,
            modules
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{
        RunOutcome, SimulationConfig, autoplay_combat_action_to_action, run_simulations,
        simulate_party_run,
    };
    use crate::autoplay::{
        RestChoice, RewardChoice, choose_combat_action, expected_enemy_threat, pick_event_choice,
        pick_map_node, pick_rest_choice, pick_reward_choice, pick_starter_module,
    };
    use crate::combat::{
        CombatAction, CombatState, DeckState, EnemyState, FighterState, PlayerState, StatusSet,
        TurnPhase, scale_axis_value,
    };
    use crate::content::{CardId, EnemyProfileId, EventId, ModuleId, RewardTier};
    use crate::dungeon::{DungeonNode, DungeonRun, RoomKind};

    const TEST_SEED: u64 = 0xA57A_7001;

    fn test_dungeon() -> DungeonRun {
        DungeonRun::new(TEST_SEED)
    }

    fn single_enemy_combat(hand: Vec<CardId>, draw_pile: Vec<CardId>, energy: u8) -> CombatState {
        CombatState::from_persisted_parts(
            PlayerState {
                fighter: FighterState {
                    hp: 20,
                    max_hp: 20,
                    block: 0,
                    statuses: StatusSet::default(),
                },
                energy,
                max_energy: 3,
            },
            vec![EnemyState {
                fighter: FighterState {
                    hp: 20,
                    max_hp: 20,
                    block: 0,
                    statuses: StatusSet::default(),
                },
                profile: EnemyProfileId::ScoutDrone,
                intent_index: 0,
                on_hit_bleed: 0,
            }],
            DeckState {
                draw_pile,
                hand,
                discard_pile: Vec::new(),
            },
            TurnPhase::PlayerTurn,
            1,
            TEST_SEED,
        )
    }

    fn threat_test_combat(profile: EnemyProfileId, intent_index: usize) -> CombatState {
        CombatState::from_persisted_parts(
            PlayerState {
                fighter: FighterState {
                    hp: 20,
                    max_hp: 20,
                    block: 0,
                    statuses: StatusSet::default(),
                },
                energy: 3,
                max_energy: 3,
            },
            vec![EnemyState {
                fighter: FighterState {
                    hp: 20,
                    max_hp: 20,
                    block: 0,
                    statuses: StatusSet::default(),
                },
                profile,
                intent_index,
                on_hit_bleed: 0,
            }],
            DeckState {
                draw_pile: Vec::new(),
                hand: Vec::new(),
                discard_pile: Vec::new(),
            },
            TurnPhase::PlayerTurn,
            1,
            TEST_SEED,
        )
    }

    fn assert_chosen_action(combat: &CombatState, expected: CombatAction) {
        assert_eq!(
            autoplay_combat_action_to_action(combat, choose_combat_action(combat)),
            expected
        );
    }

    fn assert_reward_choice(options: [CardId; 3], expected: RewardChoice) {
        let dungeon = test_dungeon();
        assert_eq!(
            pick_reward_choice(&dungeon, &options, RewardTier::Combat),
            expected
        );
    }

    #[test]
    fn simulation_stats_are_deterministic_for_same_config() {
        let config = SimulationConfig {
            runs: 12,
            seed_start: TEST_SEED,
            players: 1,
            verbose: false,
        };

        let first = run_simulations(&config);
        let second = run_simulations(&config);

        assert_eq!(first, second);
    }

    #[test]
    fn simulation_smoke_test_accounts_for_every_run() {
        let stats = run_simulations(&SimulationConfig {
            runs: 10,
            seed_start: TEST_SEED,
            players: 1,
            verbose: false,
        });

        assert_eq!(stats.runs, 10);
        assert_eq!(stats.wins + stats.losses + stats.aborts, stats.runs);
    }

    #[test]
    fn expected_enemy_threat_focus_only_affects_damage() {
        let neutral = threat_test_combat(EnemyProfileId::NeedlerDrone, 0);
        let mut boosted = neutral.clone();
        let mut broken = neutral.clone();
        let bleed_threat = |combat: &CombatState| {
            let intent = combat.current_intent(0).expect("needler intent");
            let enemy = &combat.enemies[0];
            scale_axis_value(intent.apply_bleed as i32, enemy.fighter.statuses.momentum) * 3
        };

        boosted.enemies[0].fighter.statuses.focus = 5;
        broken.enemies[0].fighter.statuses.focus = -7;

        assert_eq!(bleed_threat(&neutral), 3);
        assert_eq!(bleed_threat(&boosted), 3);
        assert_eq!(bleed_threat(&broken), 3);
        assert_eq!(expected_enemy_threat(&neutral), 7);
        assert_eq!(expected_enemy_threat(&boosted), 9);
        assert_eq!(expected_enemy_threat(&broken), 5);
    }

    #[test]
    fn map_heuristic_prefers_rest_when_hp_is_low() {
        let mut dungeon = test_dungeon();
        dungeon.player_hp = 10;
        dungeon.nodes = vec![
            DungeonNode {
                id: 0,
                depth: 0,
                lane: 3,
                kind: RoomKind::Start,
                next: vec![1, 2],
            },
            DungeonNode {
                id: 1,
                depth: 1,
                lane: 2,
                kind: RoomKind::Combat,
                next: vec![],
            },
            DungeonNode {
                id: 2,
                depth: 1,
                lane: 4,
                kind: RoomKind::Rest,
                next: vec![],
            },
        ];
        dungeon.available_nodes = vec![1, 2];

        assert_eq!(pick_map_node(&dungeon), Some(2));
    }

    #[test]
    fn low_hp_event_heuristic_prefers_heal() {
        let mut dungeon = test_dungeon();
        dungeon.player_hp = 8;

        assert_eq!(pick_event_choice(&dungeon, EventId::ClinicPod), Some(0));
    }

    #[test]
    fn combat_heuristic_chooses_immediate_lethal_action() {
        let mut combat =
            single_enemy_combat(vec![CardId::FlareSlash, CardId::GuardStep], vec![], 1);
        combat.enemies[0].fighter.hp = 5;
        combat.enemies[0].fighter.max_hp = 5;

        assert_chosen_action(
            &combat,
            CombatAction::PlayCard {
                hand_index: 0,
                target: Some(crate::combat::Actor::Enemy(0)),
            },
        );
    }

    #[test]
    fn combat_heuristic_prefers_defense_when_damage_is_uncovered() {
        let combat = single_enemy_combat(vec![CardId::FlareSlash, CardId::GuardStep], vec![], 2);

        assert_chosen_action(
            &combat,
            CombatAction::PlayCard {
                hand_index: 1,
                target: Some(crate::combat::Actor::Player),
            },
        );
    }

    #[test]
    fn combat_heuristic_plays_free_draw_card_before_ending_turn() {
        let combat = single_enemy_combat(
            vec![CardId::HardReset],
            vec![CardId::GuardStep, CardId::RazorRush],
            0,
        );

        assert_chosen_action(
            &combat,
            CombatAction::PlayCard {
                hand_index: 0,
                target: Some(crate::combat::Actor::Player),
            },
        );
    }

    #[test]
    fn starter_module_choice_always_uses_nanoforge() {
        let dungeon = test_dungeon();

        assert_eq!(pick_starter_module(&dungeon), Some(ModuleId::Nanoforge));
    }

    #[test]
    fn reward_pick_policy_prefers_zero_cost_then_defense_then_skip() {
        for (options, expected) in [
            (
                [CardId::GuardStep, CardId::RazorRush, CardId::QuickStrike],
                RewardChoice::Pick(1),
            ),
            (
                [CardId::GuardStep, CardId::QuickStrike, CardId::FlareSlash],
                RewardChoice::Pick(0),
            ),
            (
                [
                    CardId::QuickStrike,
                    CardId::FlareSlash,
                    CardId::SunderingArc,
                ],
                RewardChoice::Skip,
            ),
        ] {
            assert_reward_choice(options, expected);
        }
    }

    #[test]
    fn rest_upgrade_selects_useful_card_when_healing_not_needed() {
        let mut dungeon = test_dungeon();
        dungeon.player_hp = dungeon.player_max_hp;
        dungeon.deck = vec![CardId::ArcSpark, CardId::GuardStep];

        assert_eq!(pick_rest_choice(&dungeon), Some(RestChoice::Upgrade(0)));
    }

    #[test]
    fn two_player_simulation_smoke_records_party_size() {
        let run = simulate_party_run(TEST_SEED, 2);

        assert_eq!(run.party_size, 2);
        assert_eq!(run.hero_hps.len(), 2);
        assert!(matches!(
            run.outcome,
            RunOutcome::Victory | RunOutcome::Defeat | RunOutcome::Abort(_)
        ));
    }

    #[test]
    fn run_simulations_two_player_mode_reports_party_stats() {
        let stats = run_simulations(&SimulationConfig {
            runs: 1,
            seed_start: TEST_SEED,
            players: 2,
            verbose: false,
        });

        assert_eq!(stats.players, 2);
        assert_eq!(stats.runs, 1);
    }
}
