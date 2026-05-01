use std::collections::VecDeque;

use crate::content::{
    AxisKind, CardId, CardTarget, EnemyIntent, EnemyProfileId, ModuleId, card_def,
    card_requirement, enemy_intent, starter_deck,
};
use crate::rng::XorShift64;

pub(crate) const DEFAULT_PLAYER_HP: i32 = 64;
pub(crate) const MAX_ENEMIES_PER_ENCOUNTER: usize = 2;
pub(crate) const MAX_HAND_CARDS: usize = 9;
const MAX_AXIS_ABS_VALUE: i8 = 9;
const AXIS_STEP_UP_BASIS_POINTS: i32 = 11_000;
const AXIS_STEP_DOWN_BASIS_POINTS: i32 = 9_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Actor {
    Player,
    Enemy(usize),
}

impl Actor {
    pub(crate) fn enemy_index(self) -> Option<usize> {
        match self {
            Self::Enemy(index) => Some(index),
            Self::Player => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum StatusKind {
    Bleed,
    Focus,
    Rhythm,
    Momentum,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CombatOutcome {
    Victory,
    Defeat,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TurnPhase {
    PlayerTurn,
    EnemyTurn,
    Ended(CombatOutcome),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct EncounterEnemySetup {
    pub(crate) hp: i32,
    pub(crate) max_hp: i32,
    pub(crate) block: i32,
    pub(crate) profile: EnemyProfileId,
    pub(crate) intent_index: usize,
    pub(crate) on_hit_bleed: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EncounterSetup {
    pub(crate) player_hp: i32,
    pub(crate) player_max_hp: i32,
    pub(crate) player_max_energy: u8,
    pub(crate) enemies: Vec<EncounterEnemySetup>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ResolvedEnemyIntent {
    pub(crate) damage: i32,
    pub(crate) hits: u8,
    pub(crate) on_hit_bleed: u8,
    pub(crate) gain_block: i32,
    pub(crate) prime_bleed: u8,
    pub(crate) self_focus: i8,
    pub(crate) self_rhythm: i8,
    pub(crate) self_momentum: i8,
    pub(crate) target_focus: i8,
    pub(crate) target_rhythm: i8,
    pub(crate) target_momentum: i8,
    pub(crate) apply_bleed: u8,
}

impl Default for EncounterSetup {
    fn default() -> Self {
        Self {
            player_hp: DEFAULT_PLAYER_HP,
            player_max_hp: DEFAULT_PLAYER_HP,
            player_max_energy: 3,
            enemies: vec![EncounterEnemySetup {
                hp: 40,
                max_hp: 40,
                block: 0,
                profile: EnemyProfileId::ScoutDrone,
                intent_index: 0,
                on_hit_bleed: 0,
            }],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct StatusSet {
    pub(crate) bleed: u8,
    pub(crate) focus: i8,
    pub(crate) rhythm: i8,
    pub(crate) momentum: i8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FighterState {
    pub(crate) hp: i32,
    pub(crate) max_hp: i32,
    pub(crate) block: i32,
    pub(crate) statuses: StatusSet,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PlayerState {
    pub(crate) fighter: FighterState,
    pub(crate) energy: u8,
    pub(crate) max_energy: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EnemyState {
    pub(crate) fighter: FighterState,
    pub(crate) profile: EnemyProfileId,
    pub(crate) intent_index: usize,
    pub(crate) on_hit_bleed: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DeckState {
    pub(crate) draw_pile: Vec<CardId>,
    pub(crate) hand: Vec<CardId>,
    pub(crate) discard_pile: Vec<CardId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CombatAction {
    PlayCard {
        hand_index: usize,
        target: Option<Actor>,
    },
    EndTurn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CombatEvent {
    CombatStarted,
    TurnStarted {
        actor: Actor,
        turn: u32,
    },
    TurnEnded {
        actor: Actor,
    },
    CardDrawn {
        card: CardId,
    },
    CardPlayed {
        card: CardId,
    },
    CardBurned {
        card: CardId,
    },
    CardCreated {
        card: CardId,
    },
    CardsDiscarded {
        count: usize,
    },
    Reshuffled,
    EnergySpent {
        amount: u8,
        remaining: u8,
    },
    RequirementNotMet {
        axis: AxisKind,
        threshold: i8,
        actual: i8,
    },
    DamageDealt {
        source: Actor,
        target: Actor,
        amount: i32,
    },
    BlockGained {
        actor: Actor,
        amount: i32,
    },
    BlockSpent {
        actor: Actor,
        amount: i32,
    },
    BlockCleared {
        actor: Actor,
        amount: i32,
    },
    StatusApplied {
        target: Actor,
        status: StatusKind,
        amount: i8,
    },
    StatusTicked {
        actor: Actor,
        status: StatusKind,
        amount: u8,
    },
    ActorDefeated {
        actor: Actor,
    },
    EnemyPrimedBleed {
        enemy_index: usize,
        amount: u8,
    },
    IntentAdvanced {
        enemy_index: usize,
        intent: EnemyIntent,
    },
    NotEnoughEnergy {
        needed: u8,
        available: u8,
    },
    InvalidAction {
        reason: &'static str,
    },
    CombatWon,
    CombatLost,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CombatCommand {
    StartCombat,
    StartTurn(Actor),
    DrawCards(u8),
    PlayCard {
        hand_index: usize,
        target: Option<Actor>,
    },
    EndTurn(Actor),
    ApplyEndOfTurn(Actor),
    ResolveEnemyIntent,
    CheckOutcome,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CombatState {
    pub(crate) player: PlayerState,
    pub(crate) enemies: Vec<EnemyState>,
    pub(crate) deck: DeckState,
    pub(crate) phase: TurnPhase,
    pub(crate) turn: u32,
    announced_enemy_defeats: Vec<bool>,
    rng: XorShift64,
}

impl CombatState {
    pub(crate) fn new(seed: u64) -> (Self, Vec<CombatEvent>) {
        Self::new_with_setup(seed, EncounterSetup::default())
    }

    pub(crate) fn new_with_setup(seed: u64, setup: EncounterSetup) -> (Self, Vec<CombatEvent>) {
        Self::new_with_deck(seed, setup, starter_deck())
    }

    pub(crate) fn new_with_deck(
        seed: u64,
        mut setup: EncounterSetup,
        mut source_deck: Vec<CardId>,
    ) -> (Self, Vec<CombatEvent>) {
        let mut rng = XorShift64::new(seed);
        if source_deck.is_empty() {
            source_deck = starter_deck();
        }
        let mut draw_pile = source_deck;
        shuffle(&mut draw_pile, &mut rng);

        if setup.enemies.is_empty() {
            setup.enemies = EncounterSetup::default().enemies;
        }
        setup.enemies.truncate(MAX_ENEMIES_PER_ENCOUNTER);

        let enemies: Vec<_> = setup
            .enemies
            .into_iter()
            .map(|enemy| EnemyState {
                fighter: FighterState {
                    hp: enemy.hp.clamp(1, enemy.max_hp.max(1)),
                    max_hp: enemy.max_hp.max(1),
                    block: enemy.block.max(0),
                    statuses: StatusSet::default(),
                },
                profile: enemy.profile,
                intent_index: enemy.intent_index % 3,
                on_hit_bleed: enemy.on_hit_bleed,
            })
            .collect();

        let mut state = Self {
            player: PlayerState {
                fighter: FighterState {
                    hp: setup.player_hp.clamp(1, setup.player_max_hp.max(1)),
                    max_hp: setup.player_max_hp.max(1),
                    block: 0,
                    statuses: StatusSet::default(),
                },
                energy: 0,
                max_energy: setup.player_max_energy.max(1),
            },
            announced_enemy_defeats: vec![false; enemies.len()],
            enemies,
            deck: DeckState {
                draw_pile,
                hand: Vec::new(),
                discard_pile: Vec::new(),
            },
            phase: TurnPhase::PlayerTurn,
            turn: 0,
            rng,
        };

        let events = state.process_commands([CombatCommand::StartCombat]);
        (state, events)
    }

    pub(crate) fn dispatch(&mut self, action: CombatAction) -> Vec<CombatEvent> {
        if matches!(self.phase, TurnPhase::Ended(_)) {
            return Vec::new();
        }

        let mut queue = VecDeque::new();

        match action {
            CombatAction::PlayCard { hand_index, target } => {
                queue.push_back(CombatCommand::PlayCard { hand_index, target });
                queue.push_back(CombatCommand::CheckOutcome);
            }
            CombatAction::EndTurn => {
                if !matches!(self.phase, TurnPhase::PlayerTurn) {
                    return vec![CombatEvent::InvalidAction {
                        reason: "Wait for the player turn.",
                    }];
                }

                queue.push_back(CombatCommand::EndTurn(Actor::Player));
                queue.push_back(CombatCommand::ApplyEndOfTurn(Actor::Player));
                queue.push_back(CombatCommand::CheckOutcome);
                queue.push_back(CombatCommand::StartTurn(Actor::Enemy(0)));
                queue.push_back(CombatCommand::ResolveEnemyIntent);
                queue.push_back(CombatCommand::ApplyEndOfTurn(Actor::Enemy(0)));
                queue.push_back(CombatCommand::CheckOutcome);
                queue.push_back(CombatCommand::StartTurn(Actor::Player));
            }
        }

        self.process_commands(queue)
    }

    pub(crate) fn start_player_turn_only(&mut self) -> Vec<CombatEvent> {
        self.process_commands([CombatCommand::StartTurn(Actor::Player)])
    }

    pub(crate) fn start_enemy_turn_only(&mut self) -> Vec<CombatEvent> {
        self.process_commands([CombatCommand::StartTurn(Actor::Enemy(0))])
    }

    pub(crate) fn apply_player_end_turn_only(&mut self) -> Vec<CombatEvent> {
        self.process_commands([
            CombatCommand::EndTurn(Actor::Player),
            CombatCommand::ApplyEndOfTurn(Actor::Player),
            CombatCommand::CheckOutcome,
        ])
    }

    pub(crate) fn apply_enemy_end_turn_only(&mut self) -> Vec<CombatEvent> {
        self.process_commands([
            CombatCommand::ApplyEndOfTurn(Actor::Enemy(0)),
            CombatCommand::EndTurn(Actor::Enemy(0)),
            CombatCommand::CheckOutcome,
        ])
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) fn resolve_enemy_intent_for_current_player(
        &mut self,
        enemy_index: usize,
    ) -> Vec<CombatEvent> {
        let mut events = Vec::new();
        self.resolve_enemy_intent_at(enemy_index, &mut events);
        self.check_outcome(&mut events);
        events
    }

    pub(crate) fn is_player_turn(&self) -> bool {
        matches!(self.phase, TurnPhase::PlayerTurn)
    }

    pub(crate) fn outcome(&self) -> Option<CombatOutcome> {
        match self.phase {
            TurnPhase::Ended(outcome) => Some(outcome),
            _ => None,
        }
    }

    pub(crate) fn hand_len(&self) -> usize {
        self.deck.hand.len()
    }

    pub(crate) fn enemy_count(&self) -> usize {
        self.enemies.len()
    }

    pub(crate) fn enemy(&self, index: usize) -> Option<&EnemyState> {
        self.enemies.get(index)
    }

    pub(crate) fn enemy_mut(&mut self, index: usize) -> Option<&mut EnemyState> {
        self.enemies.get_mut(index)
    }

    pub(crate) fn enemy_profile(&self, index: usize) -> Option<EnemyProfileId> {
        self.enemy(index).map(|enemy| enemy.profile)
    }

    pub(crate) fn current_intent(&self, index: usize) -> Option<EnemyIntent> {
        let enemy = self.enemy(index)?;
        Some(enemy_intent(enemy.profile, enemy.intent_index))
    }

    pub(crate) fn resolved_enemy_intent(&self, index: usize) -> Option<ResolvedEnemyIntent> {
        let intent = self.current_intent(index)?;
        let enemy_actor = Actor::Enemy(index);
        let enemy_momentum = self.axis_value(enemy_actor, AxisKind::Momentum);
        Some(ResolvedEnemyIntent {
            damage: scale_axis_value(intent.damage, enemy_momentum),
            hits: intent.hits,
            on_hit_bleed: self
                .enemy(index)
                .map(|enemy| enemy.on_hit_bleed)
                .unwrap_or(0),
            gain_block: scale_axis_value(intent.gain_block, enemy_momentum),
            prime_bleed: scale_axis_value(intent.prime_bleed as i32, enemy_momentum).max(0) as u8,
            self_focus: scale_intent_axis_delta(intent.self_focus, enemy_momentum),
            self_rhythm: scale_intent_axis_delta(intent.self_rhythm, enemy_momentum),
            self_momentum: scale_intent_axis_delta(intent.self_momentum, enemy_momentum),
            target_focus: scale_intent_axis_delta(intent.target_focus, enemy_momentum),
            target_rhythm: scale_intent_axis_delta(intent.target_rhythm, enemy_momentum),
            target_momentum: scale_intent_axis_delta(intent.target_momentum, enemy_momentum),
            apply_bleed: scale_axis_value(intent.apply_bleed as i32, enemy_momentum).max(0) as u8,
        })
    }

    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    pub(crate) fn expected_enemy_threat(&self) -> i32 {
        let mut total = 0;
        for enemy_index in 0..self.enemy_count() {
            let Some(enemy) = self.enemy(enemy_index) else {
                continue;
            };
            if enemy.fighter.hp <= 0 {
                continue;
            }
            let Some(resolved) = self.resolved_enemy_intent(enemy_index) else {
                continue;
            };
            let damage = scale_axis_value(resolved.damage, enemy.fighter.statuses.focus);
            total += damage * resolved.hits as i32;
            total += resolved.apply_bleed as i32 * 3;
            total += resolved.on_hit_bleed as i32 * 3;
        }
        total
    }

    pub(crate) fn apply_multiplayer_enemy_target_effects(
        &mut self,
        enemy_index: usize,
        resolved: ResolvedEnemyIntent,
    ) -> Vec<CombatEvent> {
        let mut events = Vec::new();
        let mut on_hit_bleed = resolved.on_hit_bleed;

        for hit_index in 0..resolved.hits {
            if resolved.damage <= 0 {
                break;
            }
            self.damage(
                Actor::Enemy(enemy_index),
                Actor::Player,
                resolved.damage,
                &mut events,
            );
            if on_hit_bleed > 0 {
                self.apply_status(
                    Actor::Player,
                    StatusKind::Bleed,
                    on_hit_bleed as i8,
                    &mut events,
                );
                on_hit_bleed = 0;
            }
            if hit_index + 1 < resolved.hits && self.player.fighter.hp <= 0 {
                break;
            }
        }

        if self.player.fighter.hp > 0 {
            if resolved.target_focus != 0 {
                self.apply_status(
                    Actor::Player,
                    StatusKind::Focus,
                    resolved.target_focus,
                    &mut events,
                );
            }
            if resolved.target_rhythm != 0 {
                self.apply_status(
                    Actor::Player,
                    StatusKind::Rhythm,
                    resolved.target_rhythm,
                    &mut events,
                );
            }
            if resolved.target_momentum != 0 {
                self.apply_status(
                    Actor::Player,
                    StatusKind::Momentum,
                    resolved.target_momentum,
                    &mut events,
                );
            }
            if resolved.apply_bleed > 0 {
                self.apply_status(
                    Actor::Player,
                    StatusKind::Bleed,
                    resolved.apply_bleed as i8,
                    &mut events,
                );
            }
        }

        self.check_outcome(&mut events);
        events
    }

    pub(crate) fn apply_multiplayer_enemy_self_effects(
        &mut self,
        enemy_index: usize,
        resolved: ResolvedEnemyIntent,
        consumed_on_hit_bleed: bool,
    ) -> Vec<CombatEvent> {
        let mut events = Vec::new();
        if let Some(enemy) = self.enemy_mut(enemy_index) {
            if consumed_on_hit_bleed {
                enemy.on_hit_bleed = 0;
            }
        }

        let enemy_actor = Actor::Enemy(enemy_index);
        if resolved.gain_block > 0 {
            self.gain_block(enemy_actor, resolved.gain_block, &mut events);
        }
        if resolved.self_focus != 0 {
            self.apply_status(
                enemy_actor,
                StatusKind::Focus,
                resolved.self_focus,
                &mut events,
            );
        }
        if resolved.self_rhythm != 0 {
            self.apply_status(
                enemy_actor,
                StatusKind::Rhythm,
                resolved.self_rhythm,
                &mut events,
            );
        }
        if resolved.self_momentum != 0 {
            self.apply_status(
                enemy_actor,
                StatusKind::Momentum,
                resolved.self_momentum,
                &mut events,
            );
        }
        if resolved.prime_bleed > 0 {
            if let Some(enemy) = self.enemy_mut(enemy_index) {
                enemy.on_hit_bleed = resolved.prime_bleed;
            }
            events.push(CombatEvent::EnemyPrimedBleed {
                enemy_index,
                amount: resolved.prime_bleed,
            });
        }
        if let Some(enemy) = self.enemy_mut(enemy_index) {
            enemy.intent_index = (enemy.intent_index + 1) % 3;
        }
        if let Some(next_intent) = self.current_intent(enemy_index) {
            events.push(CombatEvent::IntentAdvanced {
                enemy_index,
                intent: next_intent,
            });
        }
        self.check_outcome(&mut events);
        events
    }

    pub(crate) fn rng_state(&self) -> u64 {
        self.rng.state
    }

    pub(crate) fn enemy_is_alive(&self, index: usize) -> bool {
        self.enemy(index)
            .map(|enemy| enemy.fighter.hp > 0)
            .unwrap_or(false)
    }

    pub(crate) fn apply_module_start_of_combat(&mut self, module: ModuleId) -> bool {
        match module {
            ModuleId::AegisDrive => {
                self.player.fighter.block = self.player.fighter.block.saturating_add(5);
                true
            }
            ModuleId::TargetingRelay => {
                self.player.fighter.statuses.focus =
                    self.player.fighter.statuses.focus.saturating_add(1);
                self.player.fighter.statuses.focus =
                    clamp_axis_value(self.player.fighter.statuses.focus);
                true
            }
            ModuleId::Nanoforge => false,
            ModuleId::CapacitorBank => {
                self.player.fighter.statuses.momentum =
                    self.player.fighter.statuses.momentum.saturating_add(1);
                self.player.fighter.statuses.momentum =
                    clamp_axis_value(self.player.fighter.statuses.momentum);
                true
            }
            ModuleId::PrismScope => {
                let mut changed = false;
                for enemy in self.enemies.iter_mut().filter(|enemy| enemy.fighter.hp > 0) {
                    enemy.fighter.statuses.rhythm =
                        clamp_axis_value(enemy.fighter.statuses.rhythm.saturating_sub(1));
                    changed = true;
                }
                changed
            }
            ModuleId::SalvageLedger => false,
            ModuleId::OverclockCore => {
                self.player.max_energy = self.player.max_energy.saturating_add(1);
                self.player.energy = self
                    .player
                    .energy
                    .saturating_add(1)
                    .min(self.player.max_energy);
                true
            }
            ModuleId::SuppressionField => {
                let mut changed = false;
                for enemy in self.enemies.iter_mut().filter(|enemy| enemy.fighter.hp > 0) {
                    enemy.fighter.statuses.focus =
                        clamp_axis_value(enemy.fighter.statuses.focus.saturating_sub(1));
                    changed = true;
                }
                changed
            }
            ModuleId::RecoveryMatrix => false,
        }
    }

    pub(crate) fn apply_start_of_combat_modules(&mut self, modules: &[ModuleId]) -> Vec<ModuleId> {
        let mut applied = Vec::new();
        for &module in modules {
            if self.apply_module_start_of_combat(module) {
                applied.push(module);
            }
        }
        applied
    }

    pub(crate) fn from_persisted_parts(
        player: PlayerState,
        enemies: Vec<EnemyState>,
        deck: DeckState,
        phase: TurnPhase,
        turn: u32,
        rng_state: u64,
    ) -> Self {
        let announced_enemy_defeats = enemies.iter().map(|enemy| enemy.fighter.hp <= 0).collect();
        Self {
            player,
            enemies,
            deck,
            phase,
            turn,
            announced_enemy_defeats,
            rng: XorShift64::new(rng_state),
        }
    }

    pub(crate) fn hand_card(&self, index: usize) -> Option<CardId> {
        self.deck.hand.get(index).copied()
    }

    pub(crate) fn can_play_card(&self, index: usize) -> bool {
        if !self.is_player_turn() {
            return false;
        }
        let Some(card) = self.hand_card(index) else {
            return false;
        };
        self.player.energy >= card_def(card).cost && self.meets_card_requirement(card)
    }

    pub(crate) fn card_requires_enemy(&self, index: usize) -> bool {
        self.hand_card(index)
            .map(|card| matches!(card_def(card).target, CardTarget::Enemy))
            .unwrap_or(false)
    }

    pub(crate) fn card_targets_all_enemies(&self, index: usize) -> bool {
        self.hand_card(index)
            .map(|card| matches!(card_def(card).target, CardTarget::AllEnemies))
            .unwrap_or(false)
    }

    fn axis_value(&self, actor: Actor, axis: AxisKind) -> i8 {
        let statuses = self.fighter(actor).statuses;
        match axis {
            AxisKind::Focus => statuses.focus,
            AxisKind::Rhythm => statuses.rhythm,
            AxisKind::Momentum => statuses.momentum,
        }
    }

    fn has_status(&self, actor: Actor, status: StatusKind) -> bool {
        let statuses = self.fighter(actor).statuses;
        match status {
            StatusKind::Bleed => statuses.bleed > 0,
            StatusKind::Focus => statuses.focus != 0,
            StatusKind::Rhythm => statuses.rhythm != 0,
            StatusKind::Momentum => statuses.momentum != 0,
        }
    }

    fn gain_energy(&mut self, amount: u8) {
        if amount == 0 {
            return;
        }
        // Energy refunds intentionally share momentum scaling with other gains,
        // so a gain of 1 can still round down to 0 under sufficiently negative momentum.
        let scaled = scale_axis_value(amount as i32, self.player.fighter.statuses.momentum);
        self.player.energy = self.player.energy.saturating_add(scaled.max(0) as u8);
    }

    fn meets_card_requirement(&self, card: CardId) -> bool {
        let Some(requirement) = card_requirement(card) else {
            return true;
        };
        self.axis_value(Actor::Player, requirement.axis) > requirement.threshold
    }

    fn process_commands<I>(&mut self, commands: I) -> Vec<CombatEvent>
    where
        I: IntoIterator<Item = CombatCommand>,
    {
        let mut queue: VecDeque<CombatCommand> = commands.into_iter().collect();
        let mut events = Vec::new();

        while let Some(command) = queue.pop_front() {
            if matches!(self.phase, TurnPhase::Ended(_)) {
                break;
            }

            match command {
                CombatCommand::StartCombat => {
                    events.push(CombatEvent::CombatStarted);
                    queue.push_back(CombatCommand::StartTurn(Actor::Player));
                }
                CombatCommand::StartTurn(actor) => {
                    self.start_turn(actor, &mut events, &mut queue);
                }
                CombatCommand::DrawCards(count) => {
                    self.draw_cards(count, &mut events);
                }
                CombatCommand::PlayCard { hand_index, target } => {
                    self.play_card(hand_index, target, &mut events, &mut queue);
                }
                CombatCommand::EndTurn(actor) => {
                    self.end_turn(actor, &mut events);
                }
                CombatCommand::ApplyEndOfTurn(actor) => {
                    self.apply_end_of_turn(actor, &mut events);
                }
                CombatCommand::ResolveEnemyIntent => {
                    self.resolve_enemy_intent(&mut events);
                }
                CombatCommand::CheckOutcome => {
                    self.check_outcome(&mut events);
                }
            }
        }

        events
    }

    fn start_turn(
        &mut self,
        actor: Actor,
        events: &mut Vec<CombatEvent>,
        queue: &mut VecDeque<CombatCommand>,
    ) {
        match actor {
            Actor::Player => {
                self.clear_block(actor, events);
                self.turn += 1;
                self.phase = TurnPhase::PlayerTurn;
                let scaled_energy = scale_axis_value(
                    self.player.max_energy as i32,
                    self.player.fighter.statuses.momentum,
                )
                .max(1);
                self.player.energy = scaled_energy as u8;
                events.push(CombatEvent::TurnStarted {
                    actor,
                    turn: self.turn,
                });
                queue.push_front(CombatCommand::DrawCards(5));
            }
            Actor::Enemy(_) => {
                for enemy_index in 0..self.enemy_count() {
                    self.clear_block(Actor::Enemy(enemy_index), events);
                }
                self.phase = TurnPhase::EnemyTurn;
                events.push(CombatEvent::TurnStarted {
                    actor: Actor::Enemy(0),
                    turn: self.turn,
                });
            }
        }
    }

    fn play_card(
        &mut self,
        hand_index: usize,
        target: Option<Actor>,
        events: &mut Vec<CombatEvent>,
        queue: &mut VecDeque<CombatCommand>,
    ) {
        if !matches!(self.phase, TurnPhase::PlayerTurn) {
            events.push(CombatEvent::InvalidAction {
                reason: "Cards can only be played on the player turn.",
            });
            return;
        }

        let Some(card) = self.deck.hand.get(hand_index).copied() else {
            events.push(CombatEvent::InvalidAction {
                reason: "That card is no longer in hand.",
            });
            return;
        };

        let def = card_def(card);

        if let Some(requirement) = card_requirement(card) {
            let actual = self.axis_value(Actor::Player, requirement.axis);
            if actual <= requirement.threshold {
                events.push(CombatEvent::RequirementNotMet {
                    axis: requirement.axis,
                    threshold: requirement.threshold,
                    actual,
                });
                return;
            }
        }

        if self.player.energy < def.cost {
            events.push(CombatEvent::NotEnoughEnergy {
                needed: def.cost,
                available: self.player.energy,
            });
            return;
        }

        let target_enemy = match def.target {
            CardTarget::Enemy => match target.and_then(Actor::enemy_index) {
                Some(enemy_index) if self.enemy_is_alive(enemy_index) => Some(enemy_index),
                _ => {
                    events.push(CombatEvent::InvalidAction {
                        reason: "Select a living enemy target.",
                    });
                    return;
                }
            },
            CardTarget::SelfOnly | CardTarget::AllEnemies => None,
        };
        self.player.energy -= def.cost;
        self.deck.hand.remove(hand_index);
        self.deck.discard_pile.push(card);

        events.push(CombatEvent::EnergySpent {
            amount: def.cost,
            remaining: self.player.energy,
        });
        events.push(CombatEvent::CardPlayed { card });

        match def.id {
            CardId::FlareSlash => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 8, events);
            }
            CardId::FlareSlashPlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 12, events);
            }
            CardId::GuardStep => {
                self.gain_block(Actor::Player, 8, events);
            }
            CardId::GuardStepPlus => {
                self.gain_block(Actor::Player, 12, events);
            }
            CardId::Slipstream => {
                self.apply_status(Actor::Player, StatusKind::Rhythm, 1, events);
                self.gain_block(Actor::Player, 3, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::SlipstreamPlus => {
                self.apply_status(Actor::Player, StatusKind::Rhythm, 1, events);
                self.gain_block(Actor::Player, 6, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::SunderingArc => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 16, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Momentum, -1, events);
            }
            CardId::SunderingArcPlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 21, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Momentum, -1, events);
            }
            CardId::QuickStrike => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 7, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::QuickStrikePlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 9, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::PinpointJab => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 7, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Bleed, 1, events);
            }
            CardId::PinpointJabPlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 9, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Bleed, 1, events);
            }
            CardId::SignalTap => {
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Momentum, -1, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::SignalTapPlus => {
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Momentum, -1, events);
                self.gain_block(Actor::Player, 5, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::Reinforce => {
                self.apply_status(Actor::Player, StatusKind::Rhythm, 1, events);
                self.gain_block(Actor::Player, 13, events);
            }
            CardId::ReinforcePlus => {
                self.apply_status(Actor::Player, StatusKind::Rhythm, 2, events);
                self.gain_block(Actor::Player, 18, events);
            }
            CardId::PressurePoint => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 5, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Focus, -1, events);
            }
            CardId::PressurePointPlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 8, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Focus, -2, events);
            }
            CardId::BurstArray => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 4, events);
                if self.enemy_is_alive(target_enemy.unwrap()) {
                    self.damage_enemy(Actor::Player, target_enemy.unwrap(), 4, events);
                }
                if self.enemy_is_alive(target_enemy.unwrap()) {
                    self.damage_enemy(Actor::Player, target_enemy.unwrap(), 4, events);
                }
            }
            CardId::BurstArrayPlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 5, events);
                if self.enemy_is_alive(target_enemy.unwrap()) {
                    self.damage_enemy(Actor::Player, target_enemy.unwrap(), 5, events);
                }
                if self.enemy_is_alive(target_enemy.unwrap()) {
                    self.damage_enemy(Actor::Player, target_enemy.unwrap(), 5, events);
                }
            }
            CardId::CoverPulse => {
                self.gain_block(Actor::Player, 10, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::CoverPulsePlus => {
                self.gain_block(Actor::Player, 13, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::TwinStrike => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 5, events);
                if self.enemy_is_alive(target_enemy.unwrap()) {
                    self.damage_enemy(Actor::Player, target_enemy.unwrap(), 5, events);
                }
            }
            CardId::TwinStrikePlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 7, events);
                if self.enemy_is_alive(target_enemy.unwrap()) {
                    self.damage_enemy(Actor::Player, target_enemy.unwrap(), 7, events);
                }
            }
            CardId::BarrierField => {
                self.gain_block(Actor::Player, 16, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Focus, -1, events);
            }
            CardId::BarrierFieldPlus => {
                self.gain_block(Actor::Player, 21, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Focus, -2, events);
            }
            CardId::TacticalBurst => {
                self.apply_status(Actor::Player, StatusKind::Focus, 1, events);
                queue.push_front(CombatCommand::DrawCards(2));
            }
            CardId::TacticalBurstPlus => {
                self.apply_status(Actor::Player, StatusKind::Focus, 2, events);
                queue.push_front(CombatCommand::DrawCards(2));
            }
            CardId::RazorNet => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 5, events);
                if self.enemy_is_alive(target_enemy.unwrap()) {
                    self.damage_enemy(Actor::Player, target_enemy.unwrap(), 5, events);
                }
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Bleed, 2, events);
            }
            CardId::RazorNetPlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 7, events);
                if self.enemy_is_alive(target_enemy.unwrap()) {
                    self.damage_enemy(Actor::Player, target_enemy.unwrap(), 7, events);
                }
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Bleed, 2, events);
            }
            CardId::FracturePulse => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 12, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Bleed, 3, events);
            }
            CardId::FracturePulsePlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 16, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Bleed, 3, events);
            }
            CardId::VectorLock => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 8, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Momentum, -2, events);
                self.gain_block(Actor::Player, 8, events);
            }
            CardId::VectorLockPlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 11, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Momentum, -2, events);
                self.gain_block(Actor::Player, 10, events);
            }
            CardId::BreachSignal => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 9, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Momentum, -2, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::BreachSignalPlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 12, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Momentum, -2, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::AnchorLoop => {
                self.gain_block(Actor::Player, 22, events);
                queue.push_front(CombatCommand::DrawCards(2));
            }
            CardId::AnchorLoopPlus => {
                self.gain_block(Actor::Player, 27, events);
                queue.push_front(CombatCommand::DrawCards(2));
            }
            CardId::ExecutionBeam => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 27, events);
            }
            CardId::ExecutionBeamPlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 35, events);
            }
            CardId::ChainBarrage => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 11, events);
                if self.enemy_is_alive(target_enemy.unwrap()) {
                    self.damage_enemy(Actor::Player, target_enemy.unwrap(), 11, events);
                }
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Bleed, 2, events);
            }
            CardId::ChainBarragePlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 13, events);
                if self.enemy_is_alive(target_enemy.unwrap()) {
                    self.damage_enemy(Actor::Player, target_enemy.unwrap(), 13, events);
                }
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Bleed, 2, events);
            }
            CardId::FortressMatrix => {
                self.gain_block(Actor::Player, 26, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::FortressMatrixPlus => {
                self.gain_block(Actor::Player, 32, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::OverwatchGrid => {
                self.gain_block(Actor::Player, 29, events);
                queue.push_front(CombatCommand::DrawCards(2));
            }
            CardId::OverwatchGridPlus => {
                self.gain_block(Actor::Player, 35, events);
                queue.push_front(CombatCommand::DrawCards(2));
            }
            CardId::RiftDart => {
                let enemy = target_enemy.unwrap();
                let disrupted = self.axis_value(Actor::Enemy(enemy), AxisKind::Momentum) < 0;
                self.damage_enemy(Actor::Player, enemy, 5, events);
                self.apply_enemy_status(enemy, StatusKind::Bleed, 1, events);
                if disrupted {
                    queue.push_front(CombatCommand::DrawCards(1));
                }
            }
            CardId::RiftDartPlus => {
                let enemy = target_enemy.unwrap();
                let disrupted = self.axis_value(Actor::Enemy(enemy), AxisKind::Momentum) < 0;
                self.damage_enemy(Actor::Player, enemy, 8, events);
                self.apply_enemy_status(enemy, StatusKind::Bleed, 1, events);
                if disrupted {
                    queue.push_front(CombatCommand::DrawCards(1));
                }
            }
            CardId::MarkPulse => {
                let enemy = target_enemy.unwrap();
                let bleeding = self.has_status(Actor::Enemy(enemy), StatusKind::Bleed);
                self.apply_enemy_status(enemy, StatusKind::Momentum, -1, events);
                if bleeding {
                    self.gain_block(Actor::Player, 6, events);
                }
            }
            CardId::MarkPulsePlus => {
                let enemy = target_enemy.unwrap();
                let bleeding = self.has_status(Actor::Enemy(enemy), StatusKind::Bleed);
                self.apply_enemy_status(enemy, StatusKind::Momentum, -1, events);
                if bleeding {
                    self.gain_block(Actor::Player, 10, events);
                }
            }
            CardId::BraceCircuit => {
                let had_block = self.player.fighter.block > 0;
                self.gain_block(Actor::Player, 10, events);
                if had_block {
                    queue.push_front(CombatCommand::DrawCards(1));
                }
            }
            CardId::BraceCircuitPlus => {
                let had_block = self.player.fighter.block > 0;
                self.gain_block(Actor::Player, 13, events);
                if had_block {
                    queue.push_front(CombatCommand::DrawCards(1));
                }
            }
            CardId::FaultShot => {
                let enemy = target_enemy.unwrap();
                let primed = self.axis_value(Actor::Enemy(enemy), AxisKind::Focus) < 0;
                self.damage_enemy(Actor::Player, enemy, 7, events);
                if primed {
                    self.apply_status(Actor::Player, StatusKind::Focus, 1, events);
                }
            }
            CardId::FaultShotPlus => {
                let enemy = target_enemy.unwrap();
                let primed = self.axis_value(Actor::Enemy(enemy), AxisKind::Focus) < 0;
                self.damage_enemy(Actor::Player, enemy, 9, events);
                if primed {
                    self.apply_status(Actor::Player, StatusKind::Focus, 1, events);
                }
            }
            CardId::SeverArc => {
                let enemy = target_enemy.unwrap();
                let bleeding = self.has_status(Actor::Enemy(enemy), StatusKind::Bleed);
                self.damage_enemy(Actor::Player, enemy, 11, events);
                if bleeding && self.enemy_is_alive(enemy) {
                    self.damage_enemy(Actor::Player, enemy, 11, events);
                }
            }
            CardId::SeverArcPlus => {
                let enemy = target_enemy.unwrap();
                let bleeding = self.has_status(Actor::Enemy(enemy), StatusKind::Bleed);
                self.damage_enemy(Actor::Player, enemy, 13, events);
                if bleeding && self.enemy_is_alive(enemy) {
                    self.damage_enemy(Actor::Player, enemy, 13, events);
                }
            }
            CardId::Lockbreaker => {
                let enemy = target_enemy.unwrap();
                let disrupted = self.axis_value(Actor::Enemy(enemy), AxisKind::Focus) < 0;
                self.damage_enemy(Actor::Player, enemy, 8, events);
                if disrupted {
                    self.apply_enemy_status(enemy, StatusKind::Focus, -1, events);
                    self.gain_block(Actor::Player, 10, events);
                }
            }
            CardId::LockbreakerPlus => {
                let enemy = target_enemy.unwrap();
                let disrupted = self.axis_value(Actor::Enemy(enemy), AxisKind::Focus) < 0;
                self.damage_enemy(Actor::Player, enemy, 11, events);
                if disrupted {
                    self.apply_enemy_status(enemy, StatusKind::Focus, -1, events);
                    self.gain_block(Actor::Player, 13, events);
                }
            }
            CardId::CounterLattice => {
                let enemy = target_enemy.unwrap();
                let weakened = self.axis_value(Actor::Enemy(enemy), AxisKind::Focus) < 0;
                self.damage_enemy(Actor::Player, enemy, 8, events);
                if weakened {
                    self.gain_energy(1);
                }
            }
            CardId::CounterLatticePlus => {
                let enemy = target_enemy.unwrap();
                let weakened = self.axis_value(Actor::Enemy(enemy), AxisKind::Focus) < 0;
                self.damage_enemy(Actor::Player, enemy, 11, events);
                if weakened {
                    self.gain_energy(1);
                }
            }
            CardId::TerminalLoop => {
                let enemy = target_enemy.unwrap();
                let bleeding = self.has_status(Actor::Enemy(enemy), StatusKind::Bleed);
                let disrupted = self.axis_value(Actor::Enemy(enemy), AxisKind::Momentum) < 0;
                self.damage_enemy(Actor::Player, enemy, 16, events);
                if bleeding {
                    queue.push_front(CombatCommand::DrawCards(1));
                }
                if disrupted {
                    self.apply_status(Actor::Player, StatusKind::Focus, 1, events);
                }
            }
            CardId::TerminalLoopPlus => {
                let enemy = target_enemy.unwrap();
                let bleeding = self.has_status(Actor::Enemy(enemy), StatusKind::Bleed);
                let disrupted = self.axis_value(Actor::Enemy(enemy), AxisKind::Momentum) < 0;
                self.damage_enemy(Actor::Player, enemy, 20, events);
                if bleeding {
                    queue.push_front(CombatCommand::DrawCards(1));
                }
                if disrupted {
                    self.apply_status(Actor::Player, StatusKind::Focus, 2, events);
                }
            }
            CardId::ZeroPoint => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 13, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Momentum, -2, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::ZeroPointPlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 19, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Momentum, -2, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::ArcSpark => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 5, events);
                self.apply_status(Actor::Player, StatusKind::Momentum, 2, events);
            }
            CardId::ArcSparkPlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 8, events);
                self.apply_status(Actor::Player, StatusKind::Momentum, 3, events);
            }
            CardId::CapacitiveShell => {
                self.gain_block(Actor::Player, 8, events);
                self.apply_status(Actor::Player, StatusKind::Momentum, 2, events);
            }
            CardId::CapacitiveShellPlus => {
                self.gain_block(Actor::Player, 13, events);
                self.apply_status(Actor::Player, StatusKind::Momentum, 3, events);
            }
            CardId::PrimeRoutine => {
                self.apply_status(Actor::Player, StatusKind::Momentum, 2, events);
                queue.push_front(CombatCommand::DrawCards(2));
            }
            CardId::PrimeRoutinePlus => {
                self.apply_status(Actor::Player, StatusKind::Momentum, 3, events);
                queue.push_front(CombatCommand::DrawCards(3));
            }
            CardId::Stockpile => {
                self.apply_status(Actor::Player, StatusKind::Momentum, 3, events);
            }
            CardId::StockpilePlus => {
                self.apply_status(Actor::Player, StatusKind::Momentum, 4, events);
            }
            CardId::PulseConverter => {
                self.set_axis(
                    Actor::Enemy(target_enemy.unwrap()),
                    AxisKind::Focus,
                    0,
                    events,
                );
            }
            CardId::PulseConverterPlus => {
                self.set_axis(
                    Actor::Enemy(target_enemy.unwrap()),
                    AxisKind::Focus,
                    0,
                    events,
                );
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::ReservoirGuard => {
                self.gain_block(Actor::Player, 16, events);
                if self.axis_value(Actor::Player, AxisKind::Momentum) > 1 {
                    self.gain_energy(1);
                }
            }
            CardId::ReservoirGuardPlus => {
                self.gain_block(Actor::Player, 21, events);
                if self.axis_value(Actor::Player, AxisKind::Momentum) > 1 {
                    self.gain_energy(1);
                }
            }
            CardId::VoltaicDrive => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 15, events);
                if self.axis_value(Actor::Player, AxisKind::Momentum) > 1 {
                    queue.push_front(CombatCommand::DrawCards(2));
                }
            }
            CardId::VoltaicDrivePlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 19, events);
                if self.axis_value(Actor::Player, AxisKind::Momentum) > 1 {
                    queue.push_front(CombatCommand::DrawCards(2));
                }
            }
            CardId::StormVault => {
                self.set_all_momentum(0, events);
            }
            CardId::StormVaultPlus => {
                self.set_all_momentum(0, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::SparkSmith => {
                self.create_card_in_hand(CardId::Spark, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::SparkSmithPlus => {
                self.create_card_in_hand(CardId::Spark, events);
                queue.push_front(CombatCommand::DrawCards(2));
            }
            CardId::PatchBay => {
                self.gain_block(Actor::Player, 10, events);
                self.create_card_in_hand(CardId::Patch, events);
            }
            CardId::PatchBayPlus => {
                self.gain_block(Actor::Player, 13, events);
                self.create_card_in_hand(CardId::Patch, events);
            }
            CardId::TracerWeave => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 5, events);
                self.create_card_in_hand(CardId::Tracer, events);
            }
            CardId::TracerWeavePlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 8, events);
                self.create_card_in_hand(CardId::Tracer, events);
            }
            CardId::NeedleNest => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 4, events);
                self.create_card_in_hand(CardId::Needler, events);
            }
            CardId::NeedleNestPlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 7, events);
                self.create_card_in_hand(CardId::Needler, events);
            }
            CardId::AssemblyLine => {
                self.create_card_in_hand(CardId::Spark, events);
                self.create_card_in_hand(CardId::Patch, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::AssemblyLinePlus => {
                self.create_card_in_hand(CardId::Spark, events);
                self.create_card_in_hand(CardId::Patch, events);
                queue.push_front(CombatCommand::DrawCards(2));
            }
            CardId::ToolCache => {
                self.create_card_in_hand(CardId::Tracer, events);
                self.create_card_in_hand(CardId::Needler, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::ToolCachePlus => {
                self.create_card_in_hand(CardId::Tracer, events);
                self.create_card_in_hand(CardId::Needler, events);
                queue.push_front(CombatCommand::DrawCards(2));
            }
            CardId::ImprovisedArsenal => {
                self.create_card_in_discard(CardId::Spark, events);
                self.create_card_in_discard(CardId::Patch, events);
                self.create_card_in_discard(CardId::Tracer, events);
                self.create_card_in_discard(CardId::Needler, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::ImprovisedArsenalPlus => {
                self.create_card_in_discard(CardId::Spark, events);
                self.create_card_in_discard(CardId::Patch, events);
                self.create_card_in_discard(CardId::Tracer, events);
                self.create_card_in_discard(CardId::Needler, events);
                queue.push_front(CombatCommand::DrawCards(2));
            }
            CardId::ForgeStorm => {
                self.create_card_in_hand(CardId::Spark, events);
                self.create_card_in_hand(CardId::Patch, events);
                self.create_card_in_hand(CardId::Tracer, events);
                self.create_card_in_hand(CardId::Needler, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::ForgeStormPlus => {
                self.create_card_in_hand(CardId::Spark, events);
                self.create_card_in_hand(CardId::Patch, events);
                self.create_card_in_hand(CardId::Tracer, events);
                self.create_card_in_hand(CardId::Needler, events);
                queue.push_front(CombatCommand::DrawCards(2));
            }
            CardId::SweepPulse => {
                self.damage_all_enemies(Actor::Player, 5, events, false);
            }
            CardId::SweepPulsePlus => {
                self.damage_all_enemies(Actor::Player, 8, events, false);
            }
            CardId::DimmingWave => {
                self.apply_status_to_all_enemies(StatusKind::Focus, -1, events);
            }
            CardId::DimmingWavePlus => {
                self.apply_status_to_all_enemies(StatusKind::Focus, -2, events);
            }
            CardId::ShrapnelVeil => {
                self.damage_all_enemies(Actor::Player, 3, events, false);
                self.gain_block(Actor::Player, 6, events);
            }
            CardId::ShrapnelVeilPlus => {
                self.damage_all_enemies(Actor::Player, 4, events, false);
                self.gain_block(Actor::Player, 10, events);
            }
            CardId::ScouringWind => {
                self.damage_all_enemies(Actor::Player, 8, events, false);
                self.apply_status_to_all_enemies(StatusKind::Bleed, 1, events);
            }
            CardId::ScouringWindPlus => {
                self.damage_all_enemies(Actor::Player, 11, events, false);
                self.apply_status_to_all_enemies(StatusKind::Bleed, 2, events);
            }
            CardId::CollapsePattern => {
                self.apply_status_to_all_enemies(StatusKind::Momentum, -1, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::CollapsePatternPlus => {
                self.apply_status_to_all_enemies(StatusKind::Momentum, -2, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::Linebreaker => {
                self.damage_all_enemies(Actor::Player, 7, events, true);
            }
            CardId::LinebreakerPlus => {
                self.damage_all_enemies(Actor::Player, 9, events, true);
            }
            CardId::NovaCollapse => {
                self.damage_all_enemies(Actor::Player, 12, events, true);
            }
            CardId::NovaCollapsePlus => {
                self.damage_all_enemies(Actor::Player, 16, events, true);
            }
            CardId::SuppressionNet => {
                self.gain_block(Actor::Player, 13, events);
                self.apply_status_to_all_enemies(StatusKind::Focus, -1, events);
                self.apply_status_to_all_enemies(StatusKind::Momentum, -1, events);
            }
            CardId::SuppressionNetPlus => {
                self.gain_block(Actor::Player, 18, events);
                self.apply_status_to_all_enemies(StatusKind::Focus, -2, events);
                self.apply_status_to_all_enemies(StatusKind::Momentum, -1, events);
            }
            CardId::RazorRush => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 9, events);
            }
            CardId::RazorRushPlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 13, events);
            }
            CardId::HardReset => {
                queue.push_front(CombatCommand::DrawCards(2));
            }
            CardId::HardResetPlus => {
                queue.push_front(CombatCommand::DrawCards(3));
            }
            CardId::EmergencyPlating => {
                self.gain_block(Actor::Player, 19, events);
            }
            CardId::EmergencyPlatingPlus => {
                self.gain_block(Actor::Player, 26, events);
            }
            CardId::Cauterize => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 7, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Focus, -1, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Rhythm, -1, events);
            }
            CardId::CauterizePlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 9, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Focus, -2, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Rhythm, -1, events);
            }
            CardId::EmberBurst => {
                self.gain_energy(1);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::EmberBurstPlus => {
                self.gain_energy(1);
                queue.push_front(CombatCommand::DrawCards(2));
            }
            CardId::AshenVector => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 16, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::AshenVectorPlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 20, events);
                queue.push_front(CombatCommand::DrawCards(2));
            }
            CardId::PurgeArray => {
                self.damage_all_enemies(Actor::Player, 9, events, false);
                self.damage_all_enemies(Actor::Player, 9, events, false);
            }
            CardId::PurgeArrayPlus => {
                self.damage_all_enemies(Actor::Player, 12, events, false);
                self.damage_all_enemies(Actor::Player, 12, events, false);
            }
            CardId::LastProtocol => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 24, events);
                self.gain_energy(1);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::LastProtocolPlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 30, events);
                self.gain_energy(1);
                queue.push_front(CombatCommand::DrawCards(2));
            }
            CardId::Spark => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 5, events);
            }
            CardId::Patch => {
                self.gain_block(Actor::Player, 8, events);
            }
            CardId::Tracer => {
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Rhythm, -1, events);
            }
            CardId::Needler => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 3, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Bleed, 1, events);
            }
        }
    }

    fn end_turn(&mut self, actor: Actor, events: &mut Vec<CombatEvent>) {
        match actor {
            Actor::Player => {
                let discarded: Vec<_> = self.deck.hand.drain(..).collect();
                let discard_count = discarded.len();
                self.deck.discard_pile.extend(discarded);
                events.push(CombatEvent::TurnEnded { actor });
                if discard_count > 0 {
                    events.push(CombatEvent::CardsDiscarded {
                        count: discard_count,
                    });
                }
            }
            Actor::Enemy(_) => {
                events.push(CombatEvent::TurnEnded {
                    actor: Actor::Enemy(0),
                });
            }
        }
    }

    fn set_axis(&mut self, actor: Actor, axis: AxisKind, value: i8, events: &mut Vec<CombatEvent>) {
        let current = self.axis_value(actor, axis);
        let clamped = clamp_axis_value(value);
        let delta = clamped - current;
        if delta != 0 {
            self.apply_status(actor, axis_status(axis), delta, events);
        }
    }

    fn set_all_momentum(&mut self, value: i8, events: &mut Vec<CombatEvent>) {
        self.set_axis(Actor::Player, AxisKind::Momentum, value, events);
        for enemy_index in 0..self.enemy_count() {
            if self.enemy_is_alive(enemy_index) {
                self.set_axis(Actor::Enemy(enemy_index), AxisKind::Momentum, value, events);
            }
        }
    }

    fn create_card_in_hand(&mut self, card: CardId, events: &mut Vec<CombatEvent>) {
        if self.deck.hand.len() >= MAX_HAND_CARDS {
            self.deck.discard_pile.push(card);
        } else {
            self.deck.hand.push(card);
        }
        events.push(CombatEvent::CardCreated { card });
    }

    fn create_card_in_discard(&mut self, card: CardId, events: &mut Vec<CombatEvent>) {
        self.deck.discard_pile.push(card);
        events.push(CombatEvent::CardCreated { card });
    }

    fn apply_status_to_all_enemies(
        &mut self,
        status: StatusKind,
        amount: i8,
        events: &mut Vec<CombatEvent>,
    ) {
        for enemy_index in 0..self.enemy_count() {
            if self.enemy_is_alive(enemy_index) {
                self.apply_enemy_status(enemy_index, status, amount, events);
            }
        }
    }

    fn damage_all_enemies(
        &mut self,
        source: Actor,
        amount: i32,
        events: &mut Vec<CombatEvent>,
        piercing: bool,
    ) {
        for enemy_index in 0..self.enemy_count() {
            if !self.enemy_is_alive(enemy_index) {
                continue;
            }
            if piercing {
                self.damage_enemy_piercing(source, enemy_index, amount, events);
            } else {
                self.damage_enemy(source, enemy_index, amount, events);
            }
        }
    }

    fn apply_end_of_turn(&mut self, actor: Actor, events: &mut Vec<CombatEvent>) {
        match actor {
            Actor::Player => self.apply_end_of_turn_to_actor(actor, events),
            Actor::Enemy(_) => {
                let enemy_indices: Vec<_> = self
                    .enemies
                    .iter()
                    .enumerate()
                    .filter(|(_, enemy)| enemy.fighter.hp > 0)
                    .map(|(index, _)| index)
                    .collect();
                for enemy_index in enemy_indices {
                    self.apply_end_of_turn_to_actor(Actor::Enemy(enemy_index), events);
                }
            }
        }
    }

    fn apply_end_of_turn_to_actor(&mut self, actor: Actor, events: &mut Vec<CombatEvent>) {
        let bleed = self.fighter(actor).statuses.bleed;
        if bleed > 0 {
            self.direct_damage(actor, bleed as i32, events);
            self.fighter_mut(actor).statuses.bleed = bleed.saturating_sub(1);
            events.push(CombatEvent::StatusTicked {
                actor,
                status: StatusKind::Bleed,
                amount: bleed,
            });
        }
        self.decay_axis_toward_zero(actor, AxisKind::Focus);
        self.decay_axis_toward_zero(actor, AxisKind::Rhythm);
        self.decay_axis_toward_zero(actor, AxisKind::Momentum);
    }

    fn decay_axis_toward_zero(&mut self, actor: Actor, axis: AxisKind) {
        let statuses = &mut self.fighter_mut(actor).statuses;
        let value = match axis {
            AxisKind::Focus => &mut statuses.focus,
            AxisKind::Rhythm => &mut statuses.rhythm,
            AxisKind::Momentum => &mut statuses.momentum,
        };
        if *value > 0 {
            *value -= 1;
        } else if *value < 0 {
            *value += 1;
        }
    }

    fn resolve_enemy_intent(&mut self, events: &mut Vec<CombatEvent>) {
        let enemy_indices: Vec<_> = self
            .enemies
            .iter()
            .enumerate()
            .filter(|(_, enemy)| enemy.fighter.hp > 0)
            .map(|(index, _)| index)
            .collect();

        for enemy_index in enemy_indices {
            self.resolve_enemy_intent_at(enemy_index, events);
            if self.player.fighter.hp <= 0 {
                break;
            }
        }

        self.end_turn(Actor::Enemy(0), events);
    }

    fn resolve_enemy_intent_at(&mut self, enemy_index: usize, events: &mut Vec<CombatEvent>) {
        let Some(intent) = self.current_intent(enemy_index) else {
            return;
        };
        let enemy_actor = Actor::Enemy(enemy_index);
        let enemy_momentum = self.axis_value(enemy_actor, AxisKind::Momentum);
        let scaled_damage = scale_axis_value(intent.damage, enemy_momentum);
        let scaled_block = scale_axis_value(intent.gain_block, enemy_momentum);
        let scaled_prime_bleed =
            scale_axis_value(intent.prime_bleed as i32, enemy_momentum).max(0) as u8;
        let scaled_apply_bleed =
            scale_axis_value(intent.apply_bleed as i32, enemy_momentum).max(0) as u8;

        for hit_index in 0..intent.hits {
            if scaled_damage <= 0 {
                break;
            }
            self.enemy_attack(enemy_index, scaled_damage, events);
            if hit_index + 1 < intent.hits && self.player.fighter.hp <= 0 {
                break;
            }
        }

        if self.player.fighter.hp <= 0 {
            return;
        }

        if scaled_block > 0 {
            self.gain_block(enemy_actor, scaled_block, events);
        }

        let self_focus = scale_intent_axis_delta(intent.self_focus, enemy_momentum);
        if self_focus != 0 {
            self.apply_status(enemy_actor, StatusKind::Focus, self_focus, events);
        }

        let self_rhythm = scale_intent_axis_delta(intent.self_rhythm, enemy_momentum);
        if self_rhythm != 0 {
            self.apply_status(enemy_actor, StatusKind::Rhythm, self_rhythm, events);
        }

        let self_momentum = scale_intent_axis_delta(intent.self_momentum, enemy_momentum);
        if self_momentum != 0 {
            self.apply_status(enemy_actor, StatusKind::Momentum, self_momentum, events);
        }

        if scaled_prime_bleed > 0 {
            if let Some(enemy) = self.enemy_mut(enemy_index) {
                enemy.on_hit_bleed = scaled_prime_bleed;
            }
            events.push(CombatEvent::EnemyPrimedBleed {
                enemy_index,
                amount: scaled_prime_bleed,
            });
        }

        let target_focus = scale_intent_axis_delta(intent.target_focus, enemy_momentum);
        if target_focus != 0 {
            self.apply_status(Actor::Player, StatusKind::Focus, target_focus, events);
        }

        let target_rhythm = scale_intent_axis_delta(intent.target_rhythm, enemy_momentum);
        if target_rhythm != 0 {
            self.apply_status(Actor::Player, StatusKind::Rhythm, target_rhythm, events);
        }

        let target_momentum = scale_intent_axis_delta(intent.target_momentum, enemy_momentum);
        if target_momentum != 0 {
            self.apply_status(Actor::Player, StatusKind::Momentum, target_momentum, events);
        }

        if scaled_apply_bleed > 0 {
            self.apply_status(
                Actor::Player,
                StatusKind::Bleed,
                scaled_apply_bleed as i8,
                events,
            );
        }

        if let Some(enemy) = self.enemy_mut(enemy_index) {
            enemy.intent_index = (enemy.intent_index + 1) % 3;
        }
        if let Some(next_intent) = self.current_intent(enemy_index) {
            events.push(CombatEvent::IntentAdvanced {
                enemy_index,
                intent: next_intent,
            });
        }
    }

    fn enemy_attack(&mut self, enemy_index: usize, amount: i32, events: &mut Vec<CombatEvent>) {
        self.damage(Actor::Enemy(enemy_index), Actor::Player, amount, events);

        let bleed = self
            .enemy(enemy_index)
            .map(|enemy| enemy.on_hit_bleed)
            .unwrap_or(0);
        if bleed > 0 {
            if let Some(enemy) = self.enemy_mut(enemy_index) {
                enemy.on_hit_bleed = 0;
            }
            self.apply_status(Actor::Player, StatusKind::Bleed, bleed as i8, events);
        }
    }

    fn draw_cards(&mut self, count: u8, events: &mut Vec<CombatEvent>) {
        for _ in 0..count {
            let Some(card) = self.draw_one(events) else {
                break;
            };

            if self.deck.hand.len() >= MAX_HAND_CARDS {
                self.deck.discard_pile.push(card);
                events.push(CombatEvent::CardBurned { card });
                continue;
            }

            self.deck.hand.push(card);
            events.push(CombatEvent::CardDrawn { card });
        }
    }

    fn draw_one(&mut self, events: &mut Vec<CombatEvent>) -> Option<CardId> {
        if self.deck.draw_pile.is_empty() {
            if self.deck.discard_pile.is_empty() {
                return None;
            }

            self.deck.draw_pile.append(&mut self.deck.discard_pile);
            shuffle(&mut self.deck.draw_pile, &mut self.rng);
            events.push(CombatEvent::Reshuffled);
        }

        self.deck.draw_pile.pop()
    }

    fn gain_block(&mut self, actor: Actor, amount: i32, events: &mut Vec<CombatEvent>) {
        let adjusted_amount = scale_axis_value(amount, self.fighter(actor).statuses.rhythm);
        if adjusted_amount <= 0 {
            return;
        }
        self.fighter_mut(actor).block += adjusted_amount;
        events.push(CombatEvent::BlockGained {
            actor,
            amount: adjusted_amount,
        });
    }

    fn clear_block(&mut self, actor: Actor, events: &mut Vec<CombatEvent>) {
        let fighter = self.fighter_mut(actor);
        let amount = fighter.block;
        if amount > 0 {
            fighter.block = 0;
            events.push(CombatEvent::BlockCleared { actor, amount });
        }
    }

    fn apply_status(
        &mut self,
        actor: Actor,
        status: StatusKind,
        amount: i8,
        events: &mut Vec<CombatEvent>,
    ) {
        let statuses = &mut self.fighter_mut(actor).statuses;
        let applied_amount = match status {
            StatusKind::Bleed => {
                if amount <= 0 {
                    return;
                }
                statuses.bleed = statuses.bleed.saturating_add(amount as u8);
                amount
            }
            StatusKind::Focus => {
                let previous = statuses.focus;
                statuses.focus = clamp_axis_value(statuses.focus.saturating_add(amount));
                statuses.focus - previous
            }
            StatusKind::Rhythm => {
                let previous = statuses.rhythm;
                statuses.rhythm = clamp_axis_value(statuses.rhythm.saturating_add(amount));
                statuses.rhythm - previous
            }
            StatusKind::Momentum => {
                let previous = statuses.momentum;
                statuses.momentum = clamp_axis_value(statuses.momentum.saturating_add(amount));
                statuses.momentum - previous
            }
        };

        if applied_amount == 0 {
            return;
        }

        events.push(CombatEvent::StatusApplied {
            target: actor,
            status,
            amount: applied_amount,
        });
    }

    fn apply_enemy_status(
        &mut self,
        enemy_index: usize,
        status: StatusKind,
        amount: i8,
        events: &mut Vec<CombatEvent>,
    ) {
        self.apply_status(Actor::Enemy(enemy_index), status, amount, events);
    }

    fn damage_enemy(
        &mut self,
        source: Actor,
        enemy_index: usize,
        base_amount: i32,
        events: &mut Vec<CombatEvent>,
    ) {
        self.damage(source, Actor::Enemy(enemy_index), base_amount, events);
    }

    fn damage_enemy_piercing(
        &mut self,
        source: Actor,
        enemy_index: usize,
        base_amount: i32,
        events: &mut Vec<CombatEvent>,
    ) {
        self.damage_piercing(source, Actor::Enemy(enemy_index), base_amount, events);
    }

    fn damage(
        &mut self,
        source: Actor,
        target: Actor,
        base_amount: i32,
        events: &mut Vec<CombatEvent>,
    ) {
        let attacker_statuses = self.fighter(source).statuses;
        let defender_statuses = self.fighter(target).statuses;
        let scaled_amount = scale_damage_amount(base_amount, attacker_statuses, defender_statuses);
        if scaled_amount <= 0 {
            return;
        }
        let fighter = self.fighter_mut(target);

        let absorbed = fighter.block.min(scaled_amount);
        if absorbed > 0 {
            fighter.block -= absorbed;
            events.push(CombatEvent::BlockSpent {
                actor: target,
                amount: absorbed,
            });
        }

        let dealt = scaled_amount - absorbed;
        if dealt > 0 {
            fighter.hp = (fighter.hp - dealt).max(0);
            events.push(CombatEvent::DamageDealt {
                source,
                target,
                amount: dealt,
            });
        }
    }

    fn damage_piercing(
        &mut self,
        source: Actor,
        target: Actor,
        base_amount: i32,
        events: &mut Vec<CombatEvent>,
    ) {
        let attacker_statuses = self.fighter(source).statuses;
        let defender_statuses = self.fighter(target).statuses;
        let dealt = scale_damage_amount(base_amount, attacker_statuses, defender_statuses);
        if dealt <= 0 {
            return;
        }
        let fighter = self.fighter_mut(target);
        fighter.hp = (fighter.hp - dealt).max(0);
        events.push(CombatEvent::DamageDealt {
            source,
            target,
            amount: dealt,
        });
    }

    fn direct_damage(&mut self, actor: Actor, amount: i32, events: &mut Vec<CombatEvent>) {
        let fighter = self.fighter_mut(actor);
        fighter.hp = (fighter.hp - amount).max(0);
        events.push(CombatEvent::DamageDealt {
            source: actor,
            target: actor,
            amount,
        });
    }

    fn check_outcome(&mut self, events: &mut Vec<CombatEvent>) {
        for enemy_index in 0..self.enemy_count() {
            let is_dead = self
                .enemy(enemy_index)
                .map(|enemy| enemy.fighter.hp <= 0)
                .unwrap_or(false);
            let already_announced = self
                .announced_enemy_defeats
                .get(enemy_index)
                .copied()
                .unwrap_or(false);
            if is_dead && !already_announced {
                if let Some(announced) = self.announced_enemy_defeats.get_mut(enemy_index) {
                    *announced = true;
                }
                events.push(CombatEvent::ActorDefeated {
                    actor: Actor::Enemy(enemy_index),
                });
            }
        }

        if self.player.fighter.hp <= 0 {
            self.phase = TurnPhase::Ended(CombatOutcome::Defeat);
            events.push(CombatEvent::ActorDefeated {
                actor: Actor::Player,
            });
            events.push(CombatEvent::CombatLost);
            return;
        }

        if self.enemies.iter().all(|enemy| enemy.fighter.hp <= 0) {
            self.phase = TurnPhase::Ended(CombatOutcome::Victory);
            events.push(CombatEvent::CombatWon);
        }
    }

    fn fighter(&self, actor: Actor) -> &FighterState {
        match actor {
            Actor::Player => &self.player.fighter,
            Actor::Enemy(index) => &self.enemies[index].fighter,
        }
    }

    fn fighter_mut(&mut self, actor: Actor) -> &mut FighterState {
        match actor {
            Actor::Player => &mut self.player.fighter,
            Actor::Enemy(index) => &mut self.enemies[index].fighter,
        }
    }
}

fn scale_damage_amount(base_amount: i32, attacker: StatusSet, defender: StatusSet) -> i32 {
    let _ = defender;
    scale_axis_value(base_amount, attacker.focus)
}

pub(crate) fn preview_scaled_value(amount: i32, axis_value: i8) -> i32 {
    scale_axis_value(amount, axis_value)
}

fn axis_multiplier_basis_points(value: i8) -> i32 {
    let mut basis_points = 10_000i32;
    for _ in 0..value.unsigned_abs() {
        basis_points = if value >= 0 {
            basis_points.saturating_mul(AXIS_STEP_UP_BASIS_POINTS) / 10_000
        } else {
            basis_points.saturating_mul(AXIS_STEP_DOWN_BASIS_POINTS) / 10_000
        };
    }
    basis_points
}

pub(crate) fn scale_axis_value(amount: i32, axis_value: i8) -> i32 {
    if amount == 0 {
        return 0;
    }
    // The shared axis curve can round a small positive value down to 0 when
    // the axis is sufficiently negative.
    let basis_points = axis_multiplier_basis_points(axis_value);
    let magnitude = amount.saturating_abs();
    let scaled = (magnitude.saturating_mul(basis_points) + 5_000) / 10_000;
    amount.signum().saturating_mul(scaled)
}

fn scale_intent_axis_delta(delta: i8, momentum: i8) -> i8 {
    scale_axis_value(delta as i32, momentum) as i8
}

fn clamp_axis_value(value: i8) -> i8 {
    value.clamp(-MAX_AXIS_ABS_VALUE, MAX_AXIS_ABS_VALUE)
}

fn axis_status(axis: AxisKind) -> StatusKind {
    match axis {
        AxisKind::Focus => StatusKind::Focus,
        AxisKind::Rhythm => StatusKind::Rhythm,
        AxisKind::Momentum => StatusKind::Momentum,
    }
}

fn shuffle<T>(items: &mut [T], rng: &mut XorShift64) {
    for index in (1..items.len()).rev() {
        let swap_with = rng.next_index(index + 1);
        items.swap(index, swap_with);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRIMARY_ENEMY: Actor = Actor::Enemy(0);

    fn primary_enemy(state: &CombatState) -> &EnemyState {
        state
            .enemy(0)
            .expect("combat fixture should have a primary enemy")
    }

    fn primary_enemy_mut(state: &mut CombatState) -> &mut EnemyState {
        state
            .enemy_mut(0)
            .expect("combat fixture should have a primary enemy")
    }

    fn set_primary_enemy_intent(
        state: &mut CombatState,
        profile: EnemyProfileId,
        intent_index: usize,
    ) {
        let enemy = primary_enemy_mut(state);
        enemy.profile = profile;
        enemy.intent_index = intent_index % 3;
    }

    fn blank_state() -> CombatState {
        let (mut state, _) = CombatState::new(0xACE5);
        state.deck.draw_pile.clear();
        state.deck.hand.clear();
        state.deck.discard_pile.clear();
        state.turn = 1;
        state.phase = TurnPhase::PlayerTurn;
        state.player.energy = 3;
        state.player.fighter.block = 0;
        state.player.fighter.statuses = StatusSet::default();
        primary_enemy_mut(&mut state).fighter.block = 0;
        primary_enemy_mut(&mut state).fighter.statuses = StatusSet::default();
        primary_enemy_mut(&mut state).fighter.hp = 40;
        primary_enemy_mut(&mut state).fighter.max_hp = 40;
        state.player.fighter.hp = 40;
        state.player.fighter.max_hp = 40;
        state
    }

    #[test]
    fn enemy_end_turn_only_emits_turn_ended() {
        let mut state = blank_state();
        state.phase = TurnPhase::EnemyTurn;

        let events = state.apply_enemy_end_turn_only();

        assert!(events.contains(&CombatEvent::TurnEnded {
            actor: Actor::Enemy(0),
        }));
    }

    #[test]
    fn reshuffles_discard_when_draw_pile_is_empty() {
        let mut state = blank_state();
        state.deck.draw_pile = vec![CardId::FlareSlash];
        state.deck.discard_pile = vec![CardId::GuardStep];

        let events = state.process_commands([CombatCommand::DrawCards(2)]);

        assert_eq!(state.deck.hand.len(), 2);
        assert!(events.contains(&CombatEvent::Reshuffled));
    }

    #[test]
    fn burns_drawn_cards_once_the_hand_reaches_nine_cards() {
        let mut state = blank_state();
        state.deck.hand = vec![
            CardId::QuickStrike,
            CardId::GuardStep,
            CardId::FlareSlash,
            CardId::PinpointJab,
            CardId::BurstArray,
            CardId::CoverPulse,
            CardId::BarrierField,
            CardId::TacticalBurst,
            CardId::RazorNet,
        ];
        state.deck.draw_pile = vec![CardId::FracturePulse];

        let events = state.process_commands([CombatCommand::DrawCards(1)]);

        assert_eq!(state.deck.hand.len(), MAX_HAND_CARDS);
        assert_eq!(state.deck.discard_pile, vec![CardId::FracturePulse]);
        assert!(events.contains(&CombatEvent::CardBurned {
            card: CardId::FracturePulse,
        }));
    }

    #[test]
    fn start_of_combat_modules_use_shared_axis_clamp_limits() {
        let mut state = blank_state();
        state.player.fighter.statuses.focus = 8;
        state.player.fighter.statuses.momentum = 8;
        primary_enemy_mut(&mut state).fighter.statuses.rhythm = -8;
        primary_enemy_mut(&mut state).fighter.statuses.focus = -8;

        let applied = state.apply_start_of_combat_modules(&[
            ModuleId::TargetingRelay,
            ModuleId::CapacitorBank,
            ModuleId::PrismScope,
            ModuleId::SuppressionField,
        ]);

        assert_eq!(
            applied,
            vec![
                ModuleId::TargetingRelay,
                ModuleId::CapacitorBank,
                ModuleId::PrismScope,
                ModuleId::SuppressionField,
            ]
        );
        assert_eq!(state.player.fighter.statuses.focus, 9);
        assert_eq!(state.player.fighter.statuses.momentum, 9);
        assert_eq!(primary_enemy(&state).fighter.statuses.rhythm, -9);
        assert_eq!(primary_enemy(&state).fighter.statuses.focus, -9);
    }

    #[test]
    fn refuses_cards_when_energy_is_too_low() {
        let mut state = blank_state();
        state.deck.hand = vec![CardId::SunderingArc];
        state.player.energy = 1;

        let events = state.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(state.deck.hand, vec![CardId::SunderingArc]);
        assert_eq!(state.player.energy, 1);
        assert!(events.iter().any(|event| matches!(
            event,
            CombatEvent::NotEnoughEnergy {
                needed: 2,
                available: 1
            }
        )));
    }

    #[test]
    fn focus_increases_damage_dealt() {
        let mut state = blank_state();
        state.player.fighter.statuses.focus = 1;

        let mut events = Vec::new();
        state.damage(Actor::Player, PRIMARY_ENEMY, 6, &mut events);

        assert_eq!(primary_enemy(&state).fighter.hp, 33);
        assert!(events.contains(&CombatEvent::DamageDealt {
            source: Actor::Player,
            target: PRIMARY_ENEMY,
            amount: 7,
        }));
    }

    #[test]
    fn negative_focus_reduces_damage_and_decays_at_end_of_turn() {
        let mut state = blank_state();
        state.player.fighter.statuses.focus = -1;

        let mut damage_events = Vec::new();
        state.damage(Actor::Player, PRIMARY_ENEMY, 8, &mut damage_events);

        assert_eq!(primary_enemy(&state).fighter.hp, 33);
        assert!(damage_events.contains(&CombatEvent::DamageDealt {
            source: Actor::Player,
            target: PRIMARY_ENEMY,
            amount: 7,
        }));

        state.process_commands([CombatCommand::ApplyEndOfTurn(Actor::Player)]);
        assert_eq!(state.player.fighter.statuses.focus, 0);
    }

    #[test]
    fn focus_scales_each_hit_and_decays_toward_zero() {
        let mut state = blank_state();
        state.player.fighter.statuses.focus = 2;
        state.deck.hand = vec![CardId::BurstArray];

        state.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(primary_enemy(&state).fighter.hp, 25);

        state.process_commands([CombatCommand::ApplyEndOfTurn(Actor::Player)]);
        assert_eq!(state.player.fighter.statuses.focus, 1);
    }

    #[test]
    fn axis_values_clamp_to_plus_and_minus_nine() {
        let mut state = blank_state();
        let mut events = Vec::new();

        state.apply_status(Actor::Player, StatusKind::Focus, 12, &mut events);
        state.apply_status(Actor::Player, StatusKind::Rhythm, -12, &mut events);
        state.apply_status(Actor::Player, StatusKind::Momentum, 15, &mut events);

        assert_eq!(state.player.fighter.statuses.focus, 9);
        assert_eq!(state.player.fighter.statuses.rhythm, -9);
        assert_eq!(state.player.fighter.statuses.momentum, 9);
    }

    #[test]
    fn status_applied_event_uses_effective_axis_delta_after_clamp() {
        let mut state = blank_state();
        let mut events = Vec::new();
        state.player.fighter.statuses.focus = 8;

        state.apply_status(Actor::Player, StatusKind::Focus, 3, &mut events);

        assert_eq!(state.player.fighter.statuses.focus, 9);
        assert!(events.contains(&CombatEvent::StatusApplied {
            target: Actor::Player,
            status: StatusKind::Focus,
            amount: 1,
        }));
    }

    #[test]
    fn rhythm_scales_block_gain_from_cards_and_enemy_intents_with_new_curve() {
        let mut state = blank_state();
        state.player.fighter.statuses.rhythm = -1;
        primary_enemy_mut(&mut state).fighter.statuses.rhythm = -1;
        state.deck.hand = vec![CardId::CoverPulse];

        state.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(Actor::Player),
        });

        assert_eq!(state.player.fighter.block, 9);

        let mut events = Vec::new();
        state.gain_block(PRIMARY_ENEMY, 8, &mut events);
        assert_eq!(primary_enemy(&state).fighter.block, 7);
        assert!(events.contains(&CombatEvent::BlockGained {
            actor: PRIMARY_ENEMY,
            amount: 7,
        }));
    }

    #[test]
    fn focus_scales_damage_multiplicatively() {
        let mut state = blank_state();
        state.player.fighter.statuses.focus = 2;

        let mut events = Vec::new();
        state.damage(Actor::Player, PRIMARY_ENEMY, 6, &mut events);

        assert_eq!(primary_enemy(&state).fighter.hp, 33);
        assert!(events.contains(&CombatEvent::DamageDealt {
            source: Actor::Player,
            target: PRIMARY_ENEMY,
            amount: 7,
        }));
    }

    #[test]
    fn bleed_still_ignores_focus_rhythm_and_momentum() {
        let mut state = blank_state();
        state.player.fighter.statuses.bleed = 3;
        state.player.fighter.statuses.focus = -2;
        state.player.fighter.statuses.rhythm = 3;
        primary_enemy_mut(&mut state).fighter.statuses.momentum = -2;

        state.process_commands([CombatCommand::ApplyEndOfTurn(Actor::Player)]);

        assert_eq!(state.player.fighter.hp, 37);
        assert_eq!(state.player.fighter.statuses.bleed, 2);
    }

    #[test]
    fn bleed_ticks_and_decays_at_end_of_turn() {
        let mut state = blank_state();
        state.player.fighter.statuses.bleed = 2;

        let events = state.process_commands([CombatCommand::ApplyEndOfTurn(Actor::Player)]);

        assert_eq!(state.player.fighter.hp, 38);
        assert_eq!(state.player.fighter.statuses.bleed, 1);
        assert!(events.contains(&CombatEvent::StatusTicked {
            actor: Actor::Player,
            status: StatusKind::Bleed,
            amount: 2,
        }));
    }

    #[test]
    fn block_clears_when_the_owner_turn_starts() {
        let mut state = blank_state();
        state.player.fighter.block = 8;
        state.phase = TurnPhase::EnemyTurn;

        let events = state.process_commands([CombatCommand::StartTurn(Actor::Player)]);

        assert_eq!(state.player.fighter.block, 0);
        assert!(events.contains(&CombatEvent::BlockCleared {
            actor: Actor::Player,
            amount: 8,
        }));
    }

    #[test]
    fn guard_step_now_only_gains_block() {
        let mut state = blank_state();
        state.deck.hand = vec![CardId::GuardStep];

        state.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: None,
        });

        assert_eq!(state.player.fighter.statuses.rhythm, 0);
        assert_eq!(state.player.fighter.block, 8);
    }

    #[test]
    fn pulse_converter_only_needs_rhythm_above_one() {
        let mut state = blank_state();
        state.deck.hand = vec![CardId::PulseConverter];
        state.player.fighter.statuses.rhythm = 1;

        assert!(!state.can_play_card(0));

        state.player.fighter.statuses.rhythm = 2;
        assert!(state.can_play_card(0));
    }

    #[test]
    fn arc_spark_now_grants_two_momentum() {
        let mut state = blank_state();
        state.deck.hand = vec![CardId::ArcSpark];

        state.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(primary_enemy(&state).fighter.hp, 35);
        assert_eq!(state.player.fighter.statuses.momentum, 2);
    }

    #[test]
    fn stockpile_plus_accumulates_momentum_and_scales_next_turn_energy() {
        let mut state = blank_state();
        state.deck.hand = vec![CardId::StockpilePlus];

        state.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: None,
        });

        assert_eq!(state.player.fighter.statuses.momentum, 4);

        state.phase = TurnPhase::EnemyTurn;
        state.process_commands([CombatCommand::StartTurn(Actor::Player)]);

        assert_eq!(state.player.energy, 4);
    }

    #[test]
    fn enemy_intent_rotates_in_order() {
        let mut state = blank_state();

        set_primary_enemy_intent(&mut state, EnemyProfileId::ScoutDrone, 0);
        primary_enemy_mut(&mut state).intent_index = 1;
        assert_eq!(state.current_intent(0).unwrap().name, "Crossfire");

        primary_enemy_mut(&mut state).intent_index = 2;
        assert_eq!(state.current_intent(0).unwrap().name, "Brace Cycle");

        primary_enemy_mut(&mut state).intent_index = 0;
        assert_eq!(state.current_intent(0).unwrap().name, "Shock Needle");
    }

    #[test]
    fn winning_card_sets_victory() {
        let mut state = blank_state();
        state.deck.hand = vec![CardId::FlareSlash];
        primary_enemy_mut(&mut state).fighter.hp = 6;

        let events = state.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(state.outcome(), Some(CombatOutcome::Victory));
        assert!(events.contains(&CombatEvent::ActorDefeated {
            actor: PRIMARY_ENEMY,
        }));
        assert!(events.contains(&CombatEvent::CombatWon));
    }

    #[test]
    fn custom_deck_is_used_when_starting_encounter() {
        let deck = vec![
            CardId::ExecutionBeam,
            CardId::FortressMatrix,
            CardId::ZeroPoint,
        ];
        let (state, _) =
            CombatState::new_with_deck(0xDEC0, EncounterSetup::default(), deck.clone());

        let total_cards =
            state.deck.draw_pile.len() + state.deck.hand.len() + state.deck.discard_pile.len();
        assert_eq!(total_cards, deck.len());
    }

    #[test]
    fn pinpoint_jab_applies_bleed() {
        let mut state = blank_state();
        state.deck.hand = vec![CardId::PinpointJab];

        state.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(primary_enemy(&state).fighter.hp, 33);
        assert_eq!(primary_enemy(&state).fighter.statuses.bleed, 1);
    }

    #[test]
    fn signal_tap_plus_applies_momentum_draws_and_gains_block() {
        let mut state = blank_state();
        state.deck.hand = vec![CardId::SignalTapPlus];
        state.deck.draw_pile = vec![CardId::FlareSlash];

        state.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(primary_enemy(&state).fighter.statuses.momentum, -1);
        assert_eq!(state.player.fighter.block, 5);
        assert_eq!(state.deck.hand, vec![CardId::FlareSlash]);
    }

    #[test]
    fn pressure_point_applies_focus_loss() {
        let mut state = blank_state();
        state.deck.hand = vec![CardId::PressurePoint];

        state.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(primary_enemy(&state).fighter.hp, 35);
        assert_eq!(primary_enemy(&state).fighter.statuses.focus, -1);
    }

    #[test]
    fn burst_array_hits_three_times() {
        let mut state = blank_state();
        state.deck.hand = vec![CardId::BurstArray];

        state.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(primary_enemy(&state).fighter.hp, 28);
    }

    #[test]
    fn cover_pulse_gains_block_and_draws() {
        let mut state = blank_state();
        state.deck.hand = vec![CardId::CoverPulse];
        state.deck.draw_pile = vec![CardId::GuardStep];

        state.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: None,
        });

        assert_eq!(state.player.fighter.block, 10);
        assert_eq!(state.deck.hand, vec![CardId::GuardStep]);
    }

    #[test]
    fn barrier_field_applies_focus_loss_while_gaining_block() {
        let mut state = blank_state();
        state.deck.hand = vec![CardId::BarrierField];

        state.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(state.player.fighter.block, 16);
        assert_eq!(primary_enemy(&state).fighter.statuses.focus, -1);
    }

    #[test]
    fn tactical_burst_draws_and_gains_focus() {
        let mut state = blank_state();
        state.deck.hand = vec![CardId::TacticalBurst];
        state.deck.draw_pile = vec![CardId::GuardStep, CardId::FlareSlash];

        state.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: None,
        });

        assert_eq!(state.player.fighter.statuses.focus, 1);
        assert_eq!(state.deck.hand.len(), 2);
        assert!(state.deck.hand.contains(&CardId::GuardStep));
        assert!(state.deck.hand.contains(&CardId::FlareSlash));
    }

    #[test]
    fn razor_net_applies_two_hits_and_bleed() {
        let mut state = blank_state();
        state.deck.hand = vec![CardId::RazorNet];

        state.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(primary_enemy(&state).fighter.hp, 30);
        assert_eq!(primary_enemy(&state).fighter.statuses.bleed, 2);
    }

    #[test]
    fn fracture_pulse_combines_damage_and_bleed() {
        let mut state = blank_state();
        state.deck.hand = vec![CardId::FracturePulse];

        state.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(primary_enemy(&state).fighter.hp, 28);
        assert_eq!(primary_enemy(&state).fighter.statuses.bleed, 3);
    }

    #[test]
    fn vector_lock_combines_damage_momentum_loss_and_block() {
        let mut state = blank_state();
        state.deck.hand = vec![CardId::VectorLock];

        state.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(primary_enemy(&state).fighter.hp, 32);
        assert_eq!(primary_enemy(&state).fighter.statuses.momentum, -2);
        assert_eq!(state.player.fighter.block, 8);
    }

    #[test]
    fn breach_signal_draws_and_applies_momentum_loss() {
        let mut state = blank_state();
        state.deck.hand = vec![CardId::BreachSignal];
        state.deck.draw_pile = vec![CardId::FlareSlash];

        state.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(primary_enemy(&state).fighter.hp, 31);
        assert_eq!(primary_enemy(&state).fighter.statuses.momentum, -2);
        assert_eq!(state.deck.hand, vec![CardId::FlareSlash]);
    }

    #[test]
    fn chain_barrage_hits_twice_and_applies_bleed() {
        let mut state = blank_state();
        state.deck.hand = vec![CardId::ChainBarrage];

        state.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(primary_enemy(&state).fighter.hp, 18);
        assert_eq!(primary_enemy(&state).fighter.statuses.bleed, 2);
    }

    #[test]
    fn overwatch_grid_gains_block_and_draws_two() {
        let mut state = blank_state();
        state.deck.hand = vec![CardId::OverwatchGrid];
        state.deck.draw_pile = vec![CardId::GuardStep, CardId::FlareSlash];

        state.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: None,
        });

        assert_eq!(state.player.fighter.block, 29);
        assert_eq!(state.deck.hand.len(), 2);
        assert!(state.deck.hand.contains(&CardId::GuardStep));
        assert!(state.deck.hand.contains(&CardId::FlareSlash));
    }

    #[test]
    fn rift_dart_variants_respect_the_momentum_draw_condition() {
        for (card, primed, expected_hp, expect_draw) in [
            (CardId::RiftDart, false, 35, false),
            (CardId::RiftDart, true, 35, true),
            (CardId::RiftDartPlus, false, 32, false),
            (CardId::RiftDartPlus, true, 32, true),
        ] {
            let mut state = blank_state();
            state.deck.hand = vec![card];
            state.deck.draw_pile = vec![CardId::FlareSlash];
            if primed {
                primary_enemy_mut(&mut state).fighter.statuses.momentum = -1;
            }

            state.dispatch(CombatAction::PlayCard {
                hand_index: 0,
                target: Some(PRIMARY_ENEMY),
            });

            assert_eq!(
                primary_enemy(&state).fighter.hp,
                expected_hp,
                "{card:?} primed={primed}"
            );
            assert_eq!(
                primary_enemy(&state).fighter.statuses.bleed,
                1,
                "{card:?} primed={primed}"
            );
            assert_eq!(
                state.deck.hand,
                if expect_draw {
                    vec![CardId::FlareSlash]
                } else {
                    Vec::new()
                },
                "{card:?} primed={primed}"
            );
        }
    }

    #[test]
    fn mark_pulse_variants_grant_block_only_against_bleeding_targets() {
        for (card, primed, expected_block) in [
            (CardId::MarkPulse, false, 0),
            (CardId::MarkPulse, true, 6),
            (CardId::MarkPulsePlus, false, 0),
            (CardId::MarkPulsePlus, true, 10),
        ] {
            let mut state = blank_state();
            state.deck.hand = vec![card];
            if primed {
                primary_enemy_mut(&mut state).fighter.statuses.bleed = 1;
            }

            state.dispatch(CombatAction::PlayCard {
                hand_index: 0,
                target: Some(PRIMARY_ENEMY),
            });

            assert_eq!(
                state.player.fighter.block, expected_block,
                "{card:?} primed={primed}"
            );
            assert_eq!(
                primary_enemy(&state).fighter.statuses.momentum,
                -1,
                "{card:?} primed={primed}"
            );
        }
    }

    #[test]
    fn brace_circuit_variants_draw_only_when_block_already_exists() {
        for (card, primed, initial_block, expected_block, expect_draw) in [
            (CardId::BraceCircuit, false, 0, 10, false),
            (CardId::BraceCircuit, true, 2, 12, true),
            (CardId::BraceCircuitPlus, false, 0, 13, false),
            (CardId::BraceCircuitPlus, true, 2, 15, true),
        ] {
            let mut state = blank_state();
            state.deck.hand = vec![card];
            state.deck.draw_pile = vec![CardId::GuardStep];
            state.player.fighter.block = initial_block;

            state.dispatch(CombatAction::PlayCard {
                hand_index: 0,
                target: None,
            });

            assert_eq!(
                state.player.fighter.block, expected_block,
                "{card:?} primed={primed}"
            );
            assert_eq!(
                state.deck.hand,
                if expect_draw {
                    vec![CardId::GuardStep]
                } else {
                    Vec::new()
                },
                "{card:?} primed={primed}"
            );
        }
    }

    #[test]
    fn fault_shot_variants_gain_focus_only_against_disrupted_targets() {
        for (card, primed, expected_hp, expected_focus) in [
            (CardId::FaultShot, false, 33, 0),
            (CardId::FaultShot, true, 33, 1),
            (CardId::FaultShotPlus, false, 31, 0),
            (CardId::FaultShotPlus, true, 31, 1),
        ] {
            let mut state = blank_state();
            state.deck.hand = vec![card];
            if primed {
                primary_enemy_mut(&mut state).fighter.statuses.focus = -1;
            }

            state.dispatch(CombatAction::PlayCard {
                hand_index: 0,
                target: Some(PRIMARY_ENEMY),
            });

            assert_eq!(
                primary_enemy(&state).fighter.hp,
                expected_hp,
                "{card:?} primed={primed}"
            );
            assert_eq!(
                state.player.fighter.statuses.focus, expected_focus,
                "{card:?} primed={primed}"
            );
        }
    }

    #[test]
    fn sever_arc_variants_hit_twice_only_against_bleeding_targets() {
        for (card, primed, expected_hp) in [
            (CardId::SeverArc, false, 29),
            (CardId::SeverArc, true, 18),
            (CardId::SeverArcPlus, false, 27),
            (CardId::SeverArcPlus, true, 14),
        ] {
            let mut state = blank_state();
            state.deck.hand = vec![card];
            if primed {
                primary_enemy_mut(&mut state).fighter.statuses.bleed = 1;
            }

            state.dispatch(CombatAction::PlayCard {
                hand_index: 0,
                target: Some(PRIMARY_ENEMY),
            });

            assert_eq!(
                primary_enemy(&state).fighter.hp,
                expected_hp,
                "{card:?} primed={primed}"
            );
        }
    }

    #[test]
    fn lockbreaker_variants_convert_focus_disruption_into_more_focus_loss_and_block() {
        for (card, primed, expected_hp, expected_block, expected_focus) in [
            (CardId::Lockbreaker, false, 32, 0, 0),
            (CardId::Lockbreaker, true, 32, 10, -2),
            (CardId::LockbreakerPlus, false, 29, 0, 0),
            (CardId::LockbreakerPlus, true, 29, 13, -2),
        ] {
            let mut state = blank_state();
            state.deck.hand = vec![card];
            if primed {
                primary_enemy_mut(&mut state).fighter.statuses.focus = -1;
            }

            state.dispatch(CombatAction::PlayCard {
                hand_index: 0,
                target: Some(PRIMARY_ENEMY),
            });

            assert_eq!(
                primary_enemy(&state).fighter.hp,
                expected_hp,
                "{card:?} primed={primed}"
            );
            assert_eq!(
                state.player.fighter.block, expected_block,
                "{card:?} primed={primed}"
            );
            assert_eq!(
                primary_enemy(&state).fighter.statuses.focus,
                expected_focus,
                "{card:?} primed={primed}"
            );
        }
    }

    #[test]
    fn counter_lattice_variants_refund_energy_only_against_focus_broken_targets() {
        for (card, primed, expected_hp, expected_energy) in [
            (CardId::CounterLattice, false, 32, 2),
            (CardId::CounterLattice, true, 32, 3),
            (CardId::CounterLatticePlus, false, 29, 2),
            (CardId::CounterLatticePlus, true, 29, 3),
        ] {
            let mut state = blank_state();
            state.deck.hand = vec![card];
            if primed {
                primary_enemy_mut(&mut state).fighter.statuses.focus = -1;
            }

            state.dispatch(CombatAction::PlayCard {
                hand_index: 0,
                target: Some(PRIMARY_ENEMY),
            });

            assert_eq!(
                primary_enemy(&state).fighter.hp,
                expected_hp,
                "{card:?} primed={primed}"
            );
            assert_eq!(
                state.player.energy, expected_energy,
                "{card:?} primed={primed}"
            );
        }
    }

    #[test]
    fn gain_energy_can_round_small_refunds_down_to_zero_under_negative_momentum() {
        let mut state = blank_state();
        state.player.energy = 2;
        state.player.fighter.statuses.momentum = -7;

        state.gain_energy(1);

        assert_eq!(state.player.energy, 2);
    }

    #[test]
    fn terminal_loop_variants_reward_bleed_and_momentum_setup() {
        for (card, primed, expected_hp, expected_focus, expect_draw) in [
            (CardId::TerminalLoop, false, 24, 0, false),
            (CardId::TerminalLoop, true, 24, 1, true),
            (CardId::TerminalLoopPlus, false, 20, 0, false),
            (CardId::TerminalLoopPlus, true, 20, 2, true),
        ] {
            let mut state = blank_state();
            state.deck.hand = vec![card];
            state.deck.draw_pile = vec![CardId::GuardStep];
            if primed {
                primary_enemy_mut(&mut state).fighter.statuses.bleed = 1;
                primary_enemy_mut(&mut state).fighter.statuses.momentum = -1;
            }

            state.dispatch(CombatAction::PlayCard {
                hand_index: 0,
                target: Some(PRIMARY_ENEMY),
            });

            assert_eq!(
                primary_enemy(&state).fighter.hp,
                expected_hp,
                "{card:?} primed={primed}"
            );
            assert_eq!(
                state.player.fighter.statuses.focus, expected_focus,
                "{card:?} primed={primed}"
            );
            assert_eq!(
                state.deck.hand,
                if expect_draw {
                    vec![CardId::GuardStep]
                } else {
                    Vec::new()
                },
                "{card:?} primed={primed}"
            );
        }
    }

    #[test]
    fn scout_drone_brace_cycle_keeps_focus_gain_under_negative_momentum() {
        let mut state = blank_state();
        set_primary_enemy_intent(&mut state, EnemyProfileId::ScoutDrone, 2);
        primary_enemy_mut(&mut state).fighter.statuses.momentum = -1;

        let mut events = Vec::new();
        state.resolve_enemy_intent(&mut events);

        assert_eq!(primary_enemy(&state).fighter.block, 4);
        assert_eq!(primary_enemy(&state).fighter.statuses.focus, 1);
        assert!(events.contains(&CombatEvent::StatusApplied {
            target: PRIMARY_ENEMY,
            status: StatusKind::Focus,
            amount: 1,
        }));
    }

    #[test]
    fn rampart_drone_pressure_clamp_keeps_focus_loss_under_negative_momentum() {
        let mut state = blank_state();
        set_primary_enemy_intent(&mut state, EnemyProfileId::RampartDrone, 1);
        primary_enemy_mut(&mut state).fighter.statuses.momentum = -1;

        state.resolve_enemy_intent(&mut Vec::new());

        assert_eq!(state.player.fighter.hp, 32);
        assert_eq!(state.player.fighter.statuses.focus, -1);
    }

    #[test]
    fn lethal_multi_hit_intent_stops_before_follow_up_effects() {
        let mut state = blank_state();
        state.player.fighter.hp = 6;
        set_primary_enemy_intent(&mut state, EnemyProfileId::ShardWeaver, 1);

        let mut events = Vec::new();
        state.resolve_enemy_intent(&mut events);

        assert_eq!(state.player.fighter.hp, 0);
        assert_eq!(primary_enemy(&state).fighter.block, 0);
        assert_eq!(primary_enemy(&state).intent_index, 1);
        assert!(
            !events
                .iter()
                .any(|event| matches!(event, CombatEvent::IntentAdvanced { enemy_index: 0, .. }))
        );
    }

    #[test]
    fn shard_weaver_refocus_applies_rhythm_loss() {
        let mut state = blank_state();
        set_primary_enemy_intent(&mut state, EnemyProfileId::ShardWeaver, 2);

        state.resolve_enemy_intent(&mut Vec::new());

        assert_eq!(primary_enemy(&state).fighter.block, 8);
        assert_eq!(state.player.fighter.statuses.rhythm, -1);
    }

    #[test]
    fn one_hero_enemy_resolution_parity_test() {
        let mut single_path = blank_state();
        set_primary_enemy_intent(&mut single_path, EnemyProfileId::ShardWeaver, 1);
        single_path.player.fighter.block = 2;
        primary_enemy_mut(&mut single_path).fighter.statuses.focus = 1;
        primary_enemy_mut(&mut single_path)
            .fighter
            .statuses
            .momentum = 1;
        primary_enemy_mut(&mut single_path).on_hit_bleed = 2;

        let mut split_path = single_path.clone();
        let single_events = single_path.resolve_enemy_intent_for_current_player(0);
        let resolved = split_path
            .resolved_enemy_intent(0)
            .expect("enemy intent should resolve");
        let consumed_on_hit_bleed = resolved.on_hit_bleed > 0 && resolved.damage > 0;
        let mut split_events = split_path.apply_multiplayer_enemy_target_effects(0, resolved);
        split_events.extend(split_path.apply_multiplayer_enemy_self_effects(
            0,
            resolved,
            consumed_on_hit_bleed,
        ));

        assert_eq!(split_events, single_events);
        assert_eq!(split_path, single_path);
    }

    #[test]
    fn anchor_loop_gains_block_and_draws_two() {
        let mut state = blank_state();
        state.deck.hand = vec![CardId::AnchorLoop];
        state.deck.draw_pile = vec![CardId::GuardStep, CardId::FlareSlash];

        state.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: None,
        });

        assert_eq!(state.player.fighter.block, 22);
        assert_eq!(state.deck.hand.len(), 2);
        assert!(state.deck.hand.contains(&CardId::GuardStep));
        assert!(state.deck.hand.contains(&CardId::FlareSlash));
    }
}
