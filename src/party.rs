use crate::combat::{
    CombatAction, CombatEvent, CombatOutcome, CombatState, DeckState, EncounterSetup, EnemyState,
    PlayerState, TurnPhase,
};
use crate::content::{CardId, EventId, ModuleId, starter_deck};
use crate::dungeon::{
    DungeonNode, DungeonProgress, DungeonRun, EventResolution, NodeSelection, RoomKind,
};
use crate::run_logic::{PostVictoryModuleEffects, apply_post_victory_modules};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HeroRunState {
    pub(crate) deck: Vec<CardId>,
    pub(crate) modules: Vec<ModuleId>,
    pub(crate) player_hp: i32,
    pub(crate) player_max_hp: i32,
    pub(crate) credits: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SharedDungeonState {
    pub(crate) seed: u64,
    pub(crate) current_level: usize,
    pub(crate) nodes: Vec<DungeonNode>,
    pub(crate) current_node: Option<usize>,
    pub(crate) available_nodes: Vec<usize>,
    pub(crate) visited_nodes: Vec<usize>,
    pub(crate) combats_cleared: usize,
    pub(crate) elites_cleared: usize,
    pub(crate) rests_completed: usize,
    pub(crate) bosses_cleared: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PartyRunState {
    pub(crate) shared: SharedDungeonState,
    pub(crate) heroes: Vec<HeroRunState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct HeroCombatState {
    pub(crate) player: PlayerState,
    pub(crate) deck: DeckState,
    pub(crate) ready: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PartyCombatState {
    pub(crate) heroes: Vec<HeroCombatState>,
    inactive_slots: Vec<bool>,
    pub(crate) enemies: Vec<EnemyState>,
    pub(crate) phase: TurnPhase,
    pub(crate) turn: u32,
    pub(crate) rng_state: u64,
}

impl PartyRunState {
    pub(crate) fn new(seed: u64, party_size: usize) -> Self {
        let dungeon = DungeonRun::new(seed);
        let shared = Self::shared_from_dungeon(&dungeon);
        let hero = HeroRunState {
            deck: dungeon.deck.clone(),
            modules: dungeon.modules.clone(),
            player_hp: dungeon.player_hp,
            player_max_hp: dungeon.player_max_hp,
            credits: dungeon.credits,
        };
        Self {
            shared,
            heroes: vec![hero; party_size.max(1)],
        }
    }

    pub(crate) fn party_size(&self) -> usize {
        self.heroes.len()
    }

    pub(crate) fn from_dungeons(dungeons: Vec<DungeonRun>) -> Option<Self> {
        let first = dungeons.first()?;
        let shared = Self::shared_from_dungeon(first);
        let mut heroes = Vec::with_capacity(dungeons.len());
        for dungeon in dungeons {
            if Self::shared_from_dungeon(&dungeon) != shared {
                return None;
            }
            heroes.push(HeroRunState {
                deck: dungeon.deck,
                modules: dungeon.modules,
                player_hp: dungeon.player_hp,
                player_max_hp: dungeon.player_max_hp,
                credits: dungeon.credits,
            });
        }
        Some(Self { shared, heroes })
    }

    pub(crate) fn hero(&self, slot: usize) -> Option<&HeroRunState> {
        self.heroes.get(slot)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn hero_mut(&mut self, slot: usize) -> Option<&mut HeroRunState> {
        self.heroes.get_mut(slot)
    }

    pub(crate) fn active_dungeon(&self, slot: usize) -> Option<DungeonRun> {
        let hero = self.heroes.get(slot)?;
        Some(DungeonRun {
            seed: self.shared.seed,
            current_level: self.shared.current_level,
            nodes: self.shared.nodes.clone(),
            current_node: self.shared.current_node,
            available_nodes: self.shared.available_nodes.clone(),
            visited_nodes: self.shared.visited_nodes.clone(),
            deck: hero.deck.clone(),
            modules: hero.modules.clone(),
            player_hp: hero.player_hp,
            player_max_hp: hero.player_max_hp,
            credits: hero.credits,
            combats_cleared: self.shared.combats_cleared,
            elites_cleared: self.shared.elites_cleared,
            rests_completed: self.shared.rests_completed,
            bosses_cleared: self.shared.bosses_cleared,
        })
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn sync_from_active_dungeon(&mut self, slot: usize, dungeon: &DungeonRun) -> bool {
        let Some(hero) = self.heroes.get_mut(slot) else {
            return false;
        };
        hero.deck = dungeon.deck.clone();
        hero.modules = dungeon.modules.clone();
        hero.player_hp = dungeon.player_hp;
        hero.player_max_hp = dungeon.player_max_hp;
        hero.credits = dungeon.credits;
        self.shared = Self::shared_from_dungeon(dungeon);
        true
    }

    pub(crate) fn current_level(&self) -> usize {
        self.shared.current_level
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn total_levels(&self) -> usize {
        self.active_dungeon(0)
            .map(|dungeon| dungeon.total_levels())
            .unwrap_or(3)
    }

    pub(crate) fn current_room_kind(&self) -> Option<RoomKind> {
        self.active_dungeon(0)?.current_room_kind()
    }

    pub(crate) fn current_room_seed(&self) -> Option<u64> {
        self.active_dungeon(0)?.current_room_seed()
    }

    pub(crate) fn select_node(
        &mut self,
        node_id: usize,
        debug_mode: bool,
    ) -> Option<NodeSelection> {
        let mut dungeon = self.active_dungeon(0)?;
        let selection = if debug_mode && !dungeon.is_available(node_id) {
            dungeon.debug_select_node(node_id)
        } else {
            dungeon.select_node(node_id)
        };
        if selection.is_some() {
            self.shared = Self::shared_from_dungeon(&dungeon);
        }
        selection
    }

    pub(crate) fn current_encounter_setup(&self, slot: usize) -> Option<EncounterSetup> {
        self.active_dungeon(slot)?.current_encounter_setup()
    }

    pub(crate) fn rest_heal_amount(&self, slot: usize) -> i32 {
        self.active_dungeon(slot)
            .map(|dungeon| dungeon.rest_heal_amount())
            .unwrap_or(0)
    }

    pub(crate) fn upgradable_card_indices(&self, slot: usize) -> Vec<usize> {
        self.active_dungeon(slot)
            .map(|dungeon| dungeon.upgradable_card_indices())
            .unwrap_or_default()
    }

    pub(crate) fn has_module(&self, slot: usize, module: ModuleId) -> bool {
        self.active_dungeon(slot)
            .map(|dungeon| dungeon.has_module(module))
            .unwrap_or(false)
    }

    pub(crate) fn can_afford_shop_price(&self, slot: usize, price: u32) -> bool {
        self.active_dungeon(slot)
            .map(|dungeon| dungeon.can_afford_shop_price(price))
            .unwrap_or(false)
    }

    pub(crate) fn apply_rest_heal(&mut self, slot: usize) -> Option<i32> {
        let mut dungeon = self.active_dungeon(slot)?;
        if !matches!(dungeon.current_room_kind(), Some(RoomKind::Rest)) {
            return None;
        }
        let healed = dungeon.rest_heal_amount();
        dungeon.player_hp += healed;
        self.update_hero_from_dungeon(slot, &dungeon);
        Some(healed)
    }

    pub(crate) fn apply_rest_upgrade(
        &mut self,
        slot: usize,
        deck_index: usize,
    ) -> Option<(CardId, CardId)> {
        let mut dungeon = self.active_dungeon(slot)?;
        if !matches!(dungeon.current_room_kind(), Some(RoomKind::Rest)) {
            return None;
        }
        let from = *dungeon.deck.get(deck_index)?;
        let to = crate::content::upgraded_card(from)?;
        dungeon.deck[deck_index] = to;
        self.update_hero_from_dungeon(slot, &dungeon);
        Some((from, to))
    }

    pub(crate) fn apply_shop_purchase(&mut self, slot: usize, card: CardId, price: u32) -> bool {
        let mut dungeon = match self.active_dungeon(slot) {
            Some(dungeon) => dungeon,
            None => return false,
        };
        if !matches!(dungeon.current_room_kind(), Some(RoomKind::Shop))
            || !dungeon.can_afford_shop_price(price)
        {
            return false;
        }
        dungeon.credits -= price;
        dungeon.deck.push(card);
        self.update_hero_from_dungeon(slot, &dungeon);
        true
    }

    pub(crate) fn apply_event_choice(
        &mut self,
        slot: usize,
        event: EventId,
        choice_index: usize,
    ) -> Option<EventResolution> {
        let mut dungeon = self.active_dungeon(slot)?;
        if !matches!(dungeon.current_room_kind(), Some(RoomKind::Event)) {
            return None;
        }
        let effect =
            crate::content::event_choice_effect(event, choice_index, dungeon.current_level())?;
        let resolution = match effect {
            crate::content::EventChoiceEffect::GainCredits(credits) => {
                dungeon.credits = dungeon.credits.saturating_add(credits);
                EventResolution::Credits {
                    hp_lost: 0,
                    credits_gained: credits,
                }
            }
            crate::content::EventChoiceEffect::LoseHpGainCredits {
                lose_hp,
                gain_credits,
            } => {
                let hp_lost = lose_hp.min((dungeon.player_hp - 1).max(0));
                dungeon.player_hp -= hp_lost;
                dungeon.credits = dungeon.credits.saturating_add(gain_credits);
                EventResolution::Credits {
                    hp_lost,
                    credits_gained: gain_credits,
                }
            }
            crate::content::EventChoiceEffect::Heal(amount) => {
                let healed = dungeon.recover_hp(amount);
                EventResolution::Heal { healed }
            }
            crate::content::EventChoiceEffect::LoseHpGainMaxHp {
                lose_hp,
                gain_max_hp,
            } => {
                let hp_lost = lose_hp.min((dungeon.player_hp - 1).max(0));
                dungeon.player_hp -= hp_lost;
                dungeon.player_max_hp += gain_max_hp;
                dungeon.player_hp = (dungeon.player_hp + gain_max_hp).min(dungeon.player_max_hp);
                EventResolution::MaxHp {
                    hp_lost,
                    max_hp_gained: gain_max_hp,
                }
            }
            crate::content::EventChoiceEffect::AddCard(card) => {
                dungeon.deck.push(card);
                EventResolution::Card { hp_lost: 0, card }
            }
            crate::content::EventChoiceEffect::LoseHpAddCard { lose_hp, card } => {
                let hp_lost = lose_hp.min((dungeon.player_hp - 1).max(0));
                dungeon.player_hp -= hp_lost;
                dungeon.deck.push(card);
                EventResolution::Card { hp_lost, card }
            }
        };
        self.update_hero_from_dungeon(slot, &dungeon);
        Some(resolution)
    }

    pub(crate) fn add_card(&mut self, slot: usize, card: CardId) -> bool {
        let Some(hero) = self.heroes.get_mut(slot) else {
            return false;
        };
        hero.deck.push(card);
        true
    }

    pub(crate) fn add_module(&mut self, slot: usize, module: ModuleId) -> bool {
        let Some(hero) = self.heroes.get_mut(slot) else {
            return false;
        };
        if hero.modules.contains(&module) {
            return false;
        }
        hero.modules.push(module);
        true
    }

    pub(crate) fn resolve_combat_defeat(&mut self, slot: usize, player_hp: i32) -> bool {
        let Some(hero) = self.heroes.get_mut(slot) else {
            return false;
        };
        hero.player_hp = player_hp.max(0);
        true
    }

    pub(crate) fn apply_post_victory_modules(
        &mut self,
        slot: usize,
    ) -> Option<PostVictoryModuleEffects> {
        let mut dungeon = self.active_dungeon(slot)?;
        let effects = apply_post_victory_modules(&mut dungeon);
        self.update_hero_from_dungeon(slot, &dungeon);
        Some(effects)
    }

    pub(crate) fn resolve_combat_victory_all(
        &mut self,
        player_hps: &[i32],
    ) -> Option<(DungeonProgress, u32)> {
        if player_hps.len() != self.heroes.len() || self.heroes.is_empty() {
            return None;
        }
        let shared_before_victory = self.shared.clone();
        let mut progress = None;
        let mut credits_gained = 0;
        let mut resolved_shared = None;
        for (slot, &player_hp) in player_hps.iter().enumerate() {
            let hero = self.heroes.get(slot)?;
            let mut dungeon = DungeonRun {
                seed: shared_before_victory.seed,
                current_level: shared_before_victory.current_level,
                nodes: shared_before_victory.nodes.clone(),
                current_node: shared_before_victory.current_node,
                available_nodes: shared_before_victory.available_nodes.clone(),
                visited_nodes: shared_before_victory.visited_nodes.clone(),
                deck: hero.deck.clone(),
                modules: hero.modules.clone(),
                player_hp: hero.player_hp,
                player_max_hp: hero.player_max_hp,
                credits: hero.credits,
                combats_cleared: shared_before_victory.combats_cleared,
                elites_cleared: shared_before_victory.elites_cleared,
                rests_completed: shared_before_victory.rests_completed,
                bosses_cleared: shared_before_victory.bosses_cleared,
            };
            let result = dungeon.resolve_combat_victory(player_hp)?;
            credits_gained = result.1;
            if progress.is_none() {
                progress = Some(result.0);
                resolved_shared = Some(Self::shared_from_dungeon(&dungeon));
            }
            self.update_hero_from_dungeon(slot, &dungeon);
        }
        self.shared = resolved_shared?;
        Some((progress?, credits_gained))
    }

    pub(crate) fn complete_current_node_shared(&mut self) -> Option<DungeonProgress> {
        let mut dungeon = self.active_dungeon(0)?;
        let progress = dungeon.complete_current_room_shared()?;
        self.shared = Self::shared_from_dungeon(&dungeon);
        Some(progress)
    }

    fn update_hero_from_dungeon(&mut self, slot: usize, dungeon: &DungeonRun) {
        if let Some(hero) = self.heroes.get_mut(slot) {
            hero.deck = dungeon.deck.clone();
            hero.modules = dungeon.modules.clone();
            hero.player_hp = dungeon.player_hp;
            hero.player_max_hp = dungeon.player_max_hp;
            hero.credits = dungeon.credits;
        }
    }

    fn shared_from_dungeon(dungeon: &DungeonRun) -> SharedDungeonState {
        SharedDungeonState {
            seed: dungeon.seed,
            current_level: dungeon.current_level,
            nodes: dungeon.nodes.clone(),
            current_node: dungeon.current_node,
            available_nodes: dungeon.available_nodes.clone(),
            visited_nodes: dungeon.visited_nodes.clone(),
            combats_cleared: dungeon.combats_cleared,
            elites_cleared: dungeon.elites_cleared,
            rests_completed: dungeon.rests_completed,
            bosses_cleared: dungeon.bosses_cleared,
        }
    }
}

impl PartyCombatState {
    pub(crate) fn new(
        seed: u64,
        setups: &[EncounterSetup],
        source_decks: &[Vec<CardId>],
        modules: &[Vec<ModuleId>],
    ) -> Option<Self> {
        if setups.is_empty() || source_decks.len() != setups.len() || modules.len() != setups.len()
        {
            return None;
        }
        let mut enemies = Vec::new();
        let mut heroes = Vec::with_capacity(setups.len());
        let mut turn = 1;
        for (slot, setup) in setups.iter().cloned().enumerate() {
            let deck = source_decks
                .get(slot)
                .cloned()
                .filter(|deck| !deck.is_empty())
                .unwrap_or_else(starter_deck);
            let (combat, _) = CombatState::new_with_deck(
                seed ^ ((slot as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15)),
                setup,
                deck,
            );
            if enemies.is_empty() {
                enemies = combat.enemies.clone();
                turn = combat.turn;
            }
            heroes.push(HeroCombatState {
                player: combat.player,
                deck: combat.deck,
                ready: false,
            });
        }
        let mut state = Self {
            inactive_slots: vec![false; heroes.len()],
            heroes,
            enemies,
            phase: TurnPhase::PlayerTurn,
            turn,
            rng_state: seed ^ 0xBF58_476D_1CE4_E5B9,
        };
        for (slot, hero_modules) in modules.iter().enumerate() {
            state.apply_start_of_combat_modules(slot, hero_modules);
        }
        Some(state)
    }

    pub(crate) fn hero_count(&self) -> usize {
        self.heroes.len()
    }

    pub(crate) fn from_views(views: Vec<CombatState>, ready: Vec<bool>) -> Option<Self> {
        let first = views.first()?;
        if ready.len() != views.len() {
            return None;
        }
        let enemies = first.enemies.clone();
        let phase = first.phase;
        let turn = first.turn;
        let rng_state = first.rng_state();
        if views.iter().skip(1).any(|combat| {
            combat.enemies != enemies
                || combat.phase != phase
                || combat.turn != turn
                || combat.rng_state() != rng_state
        }) {
            return None;
        }
        let heroes: Vec<_> = views
            .into_iter()
            .zip(ready)
            .map(|(combat, ready)| HeroCombatState {
                player: combat.player,
                deck: combat.deck,
                ready,
            })
            .collect();
        Some(Self {
            inactive_slots: vec![false; heroes.len()],
            heroes,
            enemies,
            phase,
            turn,
            rng_state,
        })
    }

    pub(crate) fn hero_is_alive(&self, slot: usize) -> bool {
        self.heroes
            .get(slot)
            .map(|hero| hero.player.fighter.hp > 0)
            .unwrap_or(false)
    }

    pub(crate) fn hero_is_inactive(&self, slot: usize) -> bool {
        self.inactive_slots.get(slot).copied().unwrap_or(false)
    }

    pub(crate) fn set_hero_inactive(&mut self, slot: usize, inactive: bool) -> bool {
        if slot >= self.heroes.len() {
            return false;
        }
        self.normalize_inactive_slots();
        let changed = self.inactive_slots[slot] != inactive;
        self.inactive_slots[slot] = inactive;
        let was_ready = self.heroes[slot].ready;
        if inactive {
            self.heroes[slot].ready = true;
        }
        changed || (inactive && !was_ready)
    }

    pub(crate) fn resolve_if_all_living_active_heroes_ready(
        &mut self,
        playback_slot: usize,
    ) -> Option<Vec<CombatEvent>> {
        if !matches!(self.phase, TurnPhase::PlayerTurn) || !self.all_living_active_heroes_ready() {
            return None;
        }
        Some(self.resolve_enemy_round_events_for_slot(playback_slot))
    }

    pub(crate) fn all_living_active_heroes_ready(&self) -> bool {
        self.heroes
            .iter()
            .enumerate()
            .filter(|(slot, hero)| hero.player.fighter.hp > 0 && !self.hero_is_inactive(*slot))
            .all(|(_, hero)| hero.ready)
    }

    fn normalize_inactive_slots(&mut self) {
        if self.inactive_slots.len() < self.heroes.len() {
            self.inactive_slots.resize(self.heroes.len(), false);
        } else if self.inactive_slots.len() > self.heroes.len() {
            self.inactive_slots.truncate(self.heroes.len());
        }
    }

    fn hero_is_active(&self, slot: usize) -> bool {
        self.hero_is_alive(slot) && !self.hero_is_inactive(slot)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn can_play_card(&self, slot: usize, index: usize) -> bool {
        self.view_for_slot(slot)
            .map(|combat| combat.can_play_card(index))
            .unwrap_or(false)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn card_requires_enemy(&self, slot: usize, index: usize) -> bool {
        self.view_for_slot(slot)
            .map(|combat| combat.card_requires_enemy(index))
            .unwrap_or(false)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn card_targets_all_enemies(&self, slot: usize, index: usize) -> bool {
        self.view_for_slot(slot)
            .map(|combat| combat.card_targets_all_enemies(index))
            .unwrap_or(false)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn hand_card(&self, slot: usize, index: usize) -> Option<CardId> {
        self.heroes.get(slot)?.deck.hand.get(index).copied()
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn hand_len(&self, slot: usize) -> usize {
        self.heroes
            .get(slot)
            .map(|hero| hero.deck.hand.len())
            .unwrap_or(0)
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn enemy_count(&self) -> usize {
        self.enemies.len()
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn phase(&self) -> TurnPhase {
        self.phase
    }

    pub(crate) fn outcome(&self) -> Option<CombatOutcome> {
        match self.phase {
            TurnPhase::Ended(outcome) => Some(outcome),
            _ => None,
        }
    }

    pub(crate) fn view_for_slot(&self, slot: usize) -> Option<CombatState> {
        let hero = self.heroes.get(slot)?;
        Some(CombatState::from_persisted_parts(
            hero.player.clone(),
            self.enemies.clone(),
            hero.deck.clone(),
            self.phase,
            self.turn,
            self.rng_state,
        ))
    }

    pub(crate) fn play_card_with_events(
        &mut self,
        slot: usize,
        hand_index: usize,
        target_enemy: Option<usize>,
    ) -> Option<Vec<CombatEvent>> {
        if !matches!(self.phase, TurnPhase::PlayerTurn) || !self.hero_is_active(slot) {
            return None;
        }
        let hero = self.heroes.get(slot)?;
        if hero.ready {
            return None;
        }
        let mut combat = self.view_for_slot(slot)?;
        let target = target_enemy.map(crate::combat::Actor::Enemy);
        let before = combat.clone();
        let events = combat.dispatch(CombatAction::PlayCard { hand_index, target });
        if combat == before {
            return None;
        }
        self.apply_combat_view(slot, &combat);
        self.check_outcome();
        Some(events)
    }

    pub(crate) fn play_card(
        &mut self,
        slot: usize,
        hand_index: usize,
        target_enemy: Option<usize>,
    ) -> bool {
        self.play_card_with_events(slot, hand_index, target_enemy)
            .is_some()
    }

    pub(crate) fn ready_hero_with_events(&mut self, slot: usize) -> Option<Vec<CombatEvent>> {
        if !matches!(self.phase, TurnPhase::PlayerTurn) || !self.hero_is_active(slot) {
            return None;
        }
        let hero = self.heroes.get(slot)?;
        if hero.ready {
            return None;
        }
        self.heroes[slot].ready = true;
        let mut events = Vec::new();
        if let Some(mut combat) = self.view_for_slot(slot) {
            events.extend(combat.apply_player_end_turn_only());
            self.apply_combat_view(slot, &combat);
        }
        if self.all_living_active_heroes_ready() {
            events.extend(self.resolve_enemy_round_events_for_slot(slot));
        }
        Some(events)
    }

    pub(crate) fn ready_hero(&mut self, slot: usize) -> bool {
        self.ready_hero_with_events(slot).is_some()
    }

    pub(crate) fn apply_start_of_combat_modules(
        &mut self,
        slot: usize,
        modules: &[ModuleId],
    ) -> Vec<ModuleId> {
        let mut combat = match self.view_for_slot(slot) {
            Some(combat) => combat,
            None => return Vec::new(),
        };
        let applied = combat.apply_start_of_combat_modules(modules);
        self.apply_combat_view(slot, &combat);
        applied
    }

    fn resolve_enemy_round_events_for_slot(&mut self, playback_slot: usize) -> Vec<CombatEvent> {
        let mut events = Vec::new();
        self.phase = TurnPhase::EnemyTurn;
        let playback_slot = if self.hero_is_active(playback_slot) {
            Some(playback_slot)
        } else {
            self.first_active_slot()
        };
        let playback_view_slot = playback_slot.or_else(|| self.first_active_slot());
        if let Some(mut combat) = playback_view_slot.and_then(|slot| self.view_for_slot(slot)) {
            events.extend(combat.start_enemy_turn_only());
            self.enemies = combat.enemies.clone();
        }
        for enemy_index in 0..self.enemies.len() {
            if self.enemies[enemy_index].fighter.hp <= 0 {
                continue;
            }
            let Some(first_active_slot) = self.first_active_slot() else {
                break;
            };
            let Some(resolved) = self
                .view_for_slot(first_active_slot)
                .and_then(|combat| combat.resolved_enemy_intent(enemy_index))
            else {
                continue;
            };
            let consumed_on_hit_bleed = resolved.on_hit_bleed > 0
                && resolved.damage > 0
                && self.first_active_slot().is_some();
            for slot in 0..self.heroes.len() {
                if !self.hero_is_active(slot) {
                    continue;
                }
                let mut combat = match self.view_for_slot(slot) {
                    Some(combat) => combat,
                    None => continue,
                };
                let slot_events =
                    combat.apply_multiplayer_enemy_target_effects(enemy_index, resolved);
                self.apply_combat_view(slot, &combat);
                if Some(slot) == playback_slot {
                    events.extend(slot_events);
                }
            }
            if self.all_active_heroes_dead() {
                self.phase = TurnPhase::Ended(CombatOutcome::Defeat);
                return events;
            }
            let self_effect_slot = self.first_active_slot().unwrap_or(0);
            let Some(mut combat) = self.view_for_slot(self_effect_slot) else {
                self.phase = TurnPhase::Ended(CombatOutcome::Defeat);
                return events;
            };
            events.extend(combat.apply_multiplayer_enemy_self_effects(
                enemy_index,
                resolved,
                consumed_on_hit_bleed,
            ));
            self.apply_combat_view(self_effect_slot, &combat);
        }
        if self.all_active_heroes_dead() {
            self.phase = TurnPhase::Ended(CombatOutcome::Defeat);
            return events;
        }
        if let Some(mut combat) = playback_view_slot.and_then(|slot| self.view_for_slot(slot)) {
            events.extend(combat.apply_enemy_end_turn_only());
            self.enemies = combat.enemies.clone();
        }
        if self.all_enemies_dead() {
            self.phase = TurnPhase::Ended(CombatOutcome::Victory);
            return events;
        }
        let next_turn = self.turn.wrapping_add(1);
        self.phase = TurnPhase::PlayerTurn;
        for slot in 0..self.heroes.len() {
            if !self.hero_is_active(slot) {
                if let Some(hero) = self.heroes.get_mut(slot) {
                    hero.ready = true;
                }
                continue;
            }
            let mut combat = match self.view_for_slot(slot) {
                Some(combat) => combat,
                None => continue,
            };
            let slot_events = combat.start_player_turn_only();
            self.apply_combat_view(slot, &combat);
            if Some(slot) == playback_slot {
                events.extend(slot_events);
            }
            if let Some(hero) = self.heroes.get_mut(slot) {
                hero.ready = false;
            }
        }
        self.turn = next_turn;
        self.check_outcome();
        events
    }

    fn apply_combat_view(&mut self, slot: usize, combat: &CombatState) {
        let inactive = self.hero_is_inactive(slot);
        if let Some(hero) = self.heroes.get_mut(slot) {
            hero.player = combat.player.clone();
            hero.deck = combat.deck.clone();
            hero.ready = if inactive {
                true
            } else {
                hero.ready && combat.player.fighter.hp > 0
            };
        }
        self.enemies = combat.enemies.clone();
        self.rng_state = combat.rng_state();
    }

    fn all_active_heroes_dead(&self) -> bool {
        self.heroes
            .iter()
            .enumerate()
            .all(|(slot, hero)| hero.player.fighter.hp <= 0 || self.hero_is_inactive(slot))
    }

    fn all_enemies_dead(&self) -> bool {
        self.enemies.iter().all(|enemy| enemy.fighter.hp <= 0)
    }

    fn first_active_slot(&self) -> Option<usize> {
        self.heroes
            .iter()
            .enumerate()
            .find(|(slot, hero)| hero.player.fighter.hp > 0 && !self.hero_is_inactive(*slot))
            .map(|(index, _)| index)
    }

    fn check_outcome(&mut self) {
        if self.all_enemies_dead() {
            self.phase = TurnPhase::Ended(CombatOutcome::Victory);
        } else if self.all_active_heroes_dead() {
            self.phase = TurnPhase::Ended(CombatOutcome::Defeat);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::{Actor, CombatEvent, CombatState, EncounterEnemySetup, EncounterSetup};
    use crate::content::{CardId, EnemyProfileId, ModuleId};

    const TEST_SEED: u64 = 0x51A7_C0DE;

    fn setup_with_enemy() -> EncounterSetup {
        EncounterSetup {
            player_hp: 24,
            player_max_hp: 24,
            player_max_energy: 3,
            enemies: vec![EncounterEnemySetup {
                hp: 18,
                max_hp: 18,
                block: 0,
                profile: EnemyProfileId::ScoutDrone,
                intent_index: 0,
                on_hit_bleed: 0,
            }],
        }
    }

    fn setup_with_enemy_profile(profile: EnemyProfileId, intent_index: usize) -> EncounterSetup {
        EncounterSetup {
            player_hp: 24,
            player_max_hp: 24,
            player_max_energy: 3,
            enemies: vec![EncounterEnemySetup {
                hp: 18,
                max_hp: 18,
                block: 0,
                profile,
                intent_index,
                on_hit_bleed: 0,
            }],
        }
    }

    #[test]
    fn party_run_initializes_requested_party_size() {
        let party = PartyRunState::new(TEST_SEED, 3);
        assert_eq!(party.party_size(), 3);
        assert_eq!(party.hero(2).unwrap().player_hp, 32);
    }

    #[test]
    fn from_dungeons_rejects_mismatched_shared_progression() {
        let first = DungeonRun::new(TEST_SEED);
        let mut second = first.clone();
        second.current_level += 1;

        assert!(PartyRunState::from_dungeons(vec![first, second]).is_none());
    }

    #[test]
    fn from_views_rejects_mismatched_shared_combat_state() {
        let (base, _) = CombatState::new_with_setup(TEST_SEED, setup_with_enemy());
        let ready = vec![false, false];
        let mut mismatched_enemies = base.clone();
        mismatched_enemies.enemies[0].fighter.hp -= 1;
        assert!(
            PartyCombatState::from_views(vec![base.clone(), mismatched_enemies], ready.clone())
                .is_none()
        );

        let mut mismatched_phase = base.clone();
        mismatched_phase.phase = TurnPhase::EnemyTurn;
        assert!(
            PartyCombatState::from_views(vec![base.clone(), mismatched_phase], ready.clone())
                .is_none()
        );

        let mut mismatched_turn = base.clone();
        mismatched_turn.turn += 1;
        assert!(
            PartyCombatState::from_views(vec![base.clone(), mismatched_turn], ready.clone())
                .is_none()
        );

        let mismatched_rng = CombatState::from_persisted_parts(
            base.player.clone(),
            base.enemies.clone(),
            base.deck.clone(),
            base.phase,
            base.turn,
            base.rng_state().wrapping_add(1),
        );
        assert!(PartyCombatState::from_views(vec![base, mismatched_rng], ready).is_none());
    }

    #[test]
    fn rest_upgrade_only_applies_in_rest_room() {
        let mut party = PartyRunState::new(TEST_SEED, 1);
        let deck_index = party
            .upgradable_card_indices(0)
            .into_iter()
            .next()
            .expect("starter deck should have an upgrade");

        assert_eq!(party.apply_rest_upgrade(0, deck_index), None);

        party.shared.current_node = Some(0);
        if let Some(node) = party.shared.nodes.get_mut(0) {
            node.kind = RoomKind::Rest;
        }

        assert!(party.apply_rest_upgrade(0, deck_index).is_some());
    }

    #[test]
    fn rest_heal_only_applies_in_rest_room() {
        let mut party = PartyRunState::new(TEST_SEED, 1);
        party.shared.current_node = Some(0);
        if let Some(node) = party.shared.nodes.get_mut(0) {
            node.kind = RoomKind::Combat;
        }
        if let Some(hero) = party.hero_mut(0) {
            hero.player_hp = 20;
        }

        assert_eq!(party.apply_rest_heal(0), None);
        assert_eq!(party.hero(0).unwrap().player_hp, 20);

        if let Some(node) = party.shared.nodes.get_mut(0) {
            node.kind = RoomKind::Rest;
        }

        assert_eq!(party.apply_rest_heal(0), Some(6));
        assert_eq!(party.hero(0).unwrap().player_hp, 26);
    }

    #[test]
    fn rest_and_shop_apply_per_hero_without_touching_other_heroes() {
        let mut party = PartyRunState::new(TEST_SEED, 2);
        party.shared.current_node = Some(5);
        if let Some(hero) = party.hero_mut(0) {
            hero.player_hp = 20;
            hero.credits = 24;
        }
        if let Some(hero) = party.hero_mut(1) {
            hero.player_hp = 18;
            hero.credits = 0;
        }
        if let Some(node) = party.shared.nodes.get_mut(5) {
            node.kind = RoomKind::Rest;
        }
        assert_eq!(party.apply_rest_heal(0), Some(6));
        assert_eq!(party.hero(0).unwrap().player_hp, 26);
        assert_eq!(party.hero(1).unwrap().player_hp, 18);

        if let Some(node) = party.shared.nodes.get_mut(5) {
            node.kind = RoomKind::Shop;
        }
        assert!(party.apply_shop_purchase(0, CardId::QuickStrike, 24));
        assert!(!party.apply_shop_purchase(1, CardId::QuickStrike, 24));
        assert_eq!(party.hero(0).unwrap().credits, 0);
        assert_eq!(party.hero(1).unwrap().credits, 0);
    }

    #[test]
    fn combat_waits_for_all_living_heroes_before_enemy_round() {
        let mut combat = PartyCombatState::new(
            TEST_SEED,
            &[setup_with_enemy(), setup_with_enemy()],
            &[starter_deck(), starter_deck()],
            &[vec![ModuleId::AegisDrive], vec![]],
        )
        .unwrap();

        assert!(matches!(combat.phase(), TurnPhase::PlayerTurn));
        assert!(combat.ready_hero(0));
        assert!(matches!(combat.phase(), TurnPhase::PlayerTurn));
        assert!(!combat.heroes[0].ready || !combat.heroes[1].ready);
        assert!(combat.ready_hero(1));
        assert!(matches!(
            combat.phase(),
            TurnPhase::PlayerTurn | TurnPhase::Ended(_)
        ));
        assert!(!combat.heroes[0].ready);
        assert!(!combat.heroes[1].ready);
    }

    #[test]
    fn combat_defeat_requires_all_heroes_to_fall() {
        let mut combat = PartyCombatState::new(
            TEST_SEED,
            &[setup_with_enemy(), setup_with_enemy()],
            &[starter_deck(), starter_deck()],
            &[vec![], vec![]],
        )
        .unwrap();
        combat.heroes[0].player.fighter.hp = 0;
        combat.check_outcome();
        assert_eq!(combat.outcome(), None);
        combat.heroes[1].player.fighter.hp = 0;
        combat.check_outcome();
        assert_eq!(combat.outcome(), Some(CombatOutcome::Defeat));
    }

    #[test]
    fn enemy_round_damages_all_living_heroes_equally() {
        let setup = setup_with_enemy_profile(EnemyProfileId::ScoutDrone, 0);
        let mut combat = PartyCombatState::new(
            TEST_SEED,
            &[setup.clone(), setup],
            &[starter_deck(), starter_deck()],
            &[vec![], vec![]],
        )
        .unwrap();
        let starting_hp = combat.heroes[0].player.fighter.hp;
        let starting_turn = combat.turn;

        assert!(combat.ready_hero(0));
        let events = combat
            .ready_hero_with_events(1)
            .expect("second ready resolves the enemy round");

        assert_eq!(combat.heroes[0].player.fighter.hp, starting_hp - 5);
        assert_eq!(combat.heroes[1].player.fighter.hp, starting_hp - 5);
        assert_eq!(
            combat.heroes[0].player.fighter.hp,
            combat.heroes[1].player.fighter.hp
        );
        assert_eq!(combat.turn, starting_turn + 1);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(
                    event,
                    CombatEvent::TurnStarted {
                        actor: Actor::Player,
                        turn,
                    } if *turn == starting_turn + 1
                ))
                .count(),
            1
        );
    }

    #[test]
    fn inactive_hero_does_not_block_or_take_enemy_turn() {
        let setup = setup_with_enemy_profile(EnemyProfileId::ScoutDrone, 0);
        let mut combat = PartyCombatState::new(
            TEST_SEED,
            &[setup.clone(), setup],
            &[starter_deck(), starter_deck()],
            &[vec![], vec![]],
        )
        .unwrap();
        let active_starting_hp = combat.heroes[0].player.fighter.hp;
        let inactive_starting_hp = combat.heroes[1].player.fighter.hp;

        assert!(combat.set_hero_inactive(1, true));
        assert!(combat.ready_hero(0));

        assert_eq!(combat.heroes[0].player.fighter.hp, active_starting_hp - 5);
        assert_eq!(combat.heroes[1].player.fighter.hp, inactive_starting_hp);
        assert!(!combat.heroes[0].ready);
        assert!(combat.heroes[1].ready);
        assert!(matches!(combat.phase(), TurnPhase::PlayerTurn));
    }

    #[test]
    fn enemy_target_debuffs_apply_to_all_living_heroes() {
        let setup = setup_with_enemy_profile(EnemyProfileId::RampartDrone, 1);
        let mut combat = PartyCombatState::new(
            TEST_SEED,
            &[setup.clone(), setup],
            &[starter_deck(), starter_deck()],
            &[vec![], vec![]],
        )
        .unwrap();

        assert!(combat.ready_hero(0));
        assert!(combat.ready_hero(1));

        assert_eq!(combat.heroes[0].player.fighter.statuses.focus, -1);
        assert_eq!(combat.heroes[1].player.fighter.statuses.focus, -1);
    }
}
