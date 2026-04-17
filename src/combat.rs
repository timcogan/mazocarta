use std::collections::VecDeque;

use crate::content::{
    CardId, CardTarget, EnemyIntent, EnemyProfileId, card_def, enemy_intent, starter_deck,
};
use crate::rng::XorShift64;

pub(crate) const DEFAULT_PLAYER_HP: i32 = 32;
pub(crate) const MAX_ENEMIES_PER_ENCOUNTER: usize = 2;

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
    Expose,
    Weak,
    Frail,
    Strength,
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
    pub(crate) expose: u8,
    pub(crate) weak: u8,
    pub(crate) frail: u8,
    pub(crate) strength: u8,
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
    CardsDiscarded {
        count: usize,
    },
    Reshuffled,
    EnergySpent {
        amount: u8,
        remaining: u8,
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
        amount: u8,
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

    pub(crate) fn rng_state(&self) -> u64 {
        self.rng.state
    }

    pub(crate) fn first_enemy_index(&self) -> Option<usize> {
        self.enemies
            .iter()
            .enumerate()
            .find(|(_, enemy)| enemy.fighter.hp > 0)
            .map(|(index, _)| index)
            .or_else(|| (!self.enemies.is_empty()).then_some(0))
    }

    pub(crate) fn enemy_is_alive(&self, index: usize) -> bool {
        self.enemy(index)
            .map(|enemy| enemy.fighter.hp > 0)
            .unwrap_or(false)
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
        self.player.energy >= card_def(card).cost
    }

    pub(crate) fn card_requires_enemy(&self, index: usize) -> bool {
        self.hand_card(index)
            .map(|card| matches!(card_def(card).target, CardTarget::Enemy))
            .unwrap_or(false)
    }

    fn status_amount(&self, actor: Actor, status: StatusKind) -> u8 {
        let statuses = self.fighter(actor).statuses;
        match status {
            StatusKind::Bleed => statuses.bleed,
            StatusKind::Expose => statuses.expose,
            StatusKind::Weak => statuses.weak,
            StatusKind::Frail => statuses.frail,
            StatusKind::Strength => statuses.strength,
        }
    }

    fn has_status(&self, actor: Actor, status: StatusKind) -> bool {
        self.status_amount(actor, status) > 0
    }

    fn gain_energy(&mut self, amount: u8) {
        self.player.energy = self.player.energy.saturating_add(amount);
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
                self.player.energy = self.player.max_energy;
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

        if self.player.energy < def.cost {
            events.push(CombatEvent::NotEnoughEnergy {
                needed: def.cost,
                available: self.player.energy,
            });
            return;
        }

        let target_enemy = if matches!(def.target, CardTarget::Enemy) {
            match target.and_then(Actor::enemy_index) {
                Some(enemy_index) if self.enemy_is_alive(enemy_index) => Some(enemy_index),
                _ => {
                    events.push(CombatEvent::InvalidAction {
                        reason: "Select a living enemy target.",
                    });
                    return;
                }
            }
        } else {
            None
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
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 6, events);
            }
            CardId::FlareSlashPlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 9, events);
            }
            CardId::GuardStep => {
                self.gain_block(Actor::Player, 5, events);
            }
            CardId::GuardStepPlus => {
                self.gain_block(Actor::Player, 8, events);
            }
            CardId::Slipstream => {
                self.gain_block(Actor::Player, 2, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::SlipstreamPlus => {
                self.gain_block(Actor::Player, 4, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::SunderingArc => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 12, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Expose, 1, events);
            }
            CardId::SunderingArcPlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 16, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Expose, 1, events);
            }
            CardId::QuickStrike => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 5, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::QuickStrikePlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 7, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::PinpointJab => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 5, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Bleed, 1, events);
            }
            CardId::PinpointJabPlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 7, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Bleed, 1, events);
            }
            CardId::SignalTap => {
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Expose, 1, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::SignalTapPlus => {
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Expose, 1, events);
                self.gain_block(Actor::Player, 3, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::Reinforce => {
                self.gain_block(Actor::Player, 8, events);
            }
            CardId::ReinforcePlus => {
                self.gain_block(Actor::Player, 11, events);
            }
            CardId::PressurePoint => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 4, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Weak, 1, events);
            }
            CardId::PressurePointPlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 6, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Weak, 2, events);
            }
            CardId::BurstArray => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 3, events);
                if self.enemy_is_alive(target_enemy.unwrap()) {
                    self.damage_enemy(Actor::Player, target_enemy.unwrap(), 3, events);
                }
                if self.enemy_is_alive(target_enemy.unwrap()) {
                    self.damage_enemy(Actor::Player, target_enemy.unwrap(), 3, events);
                }
            }
            CardId::BurstArrayPlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 4, events);
                if self.enemy_is_alive(target_enemy.unwrap()) {
                    self.damage_enemy(Actor::Player, target_enemy.unwrap(), 4, events);
                }
                if self.enemy_is_alive(target_enemy.unwrap()) {
                    self.damage_enemy(Actor::Player, target_enemy.unwrap(), 4, events);
                }
            }
            CardId::CoverPulse => {
                self.gain_block(Actor::Player, 6, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::CoverPulsePlus => {
                self.gain_block(Actor::Player, 8, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::TwinStrike => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 4, events);
                if self.enemy_is_alive(target_enemy.unwrap()) {
                    self.damage_enemy(Actor::Player, target_enemy.unwrap(), 4, events);
                }
            }
            CardId::TwinStrikePlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 5, events);
                if self.enemy_is_alive(target_enemy.unwrap()) {
                    self.damage_enemy(Actor::Player, target_enemy.unwrap(), 5, events);
                }
            }
            CardId::BarrierField => {
                self.gain_block(Actor::Player, 10, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Frail, 1, events);
            }
            CardId::BarrierFieldPlus => {
                self.gain_block(Actor::Player, 13, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Frail, 2, events);
            }
            CardId::TacticalBurst => {
                self.apply_status(Actor::Player, StatusKind::Strength, 1, events);
                queue.push_front(CombatCommand::DrawCards(2));
            }
            CardId::TacticalBurstPlus => {
                self.apply_status(Actor::Player, StatusKind::Strength, 2, events);
                queue.push_front(CombatCommand::DrawCards(2));
            }
            CardId::RazorNet => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 4, events);
                if self.enemy_is_alive(target_enemy.unwrap()) {
                    self.damage_enemy(Actor::Player, target_enemy.unwrap(), 4, events);
                }
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Bleed, 2, events);
            }
            CardId::RazorNetPlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 5, events);
                if self.enemy_is_alive(target_enemy.unwrap()) {
                    self.damage_enemy(Actor::Player, target_enemy.unwrap(), 5, events);
                }
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Bleed, 2, events);
            }
            CardId::FracturePulse => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 9, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Bleed, 3, events);
            }
            CardId::FracturePulsePlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 12, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Bleed, 3, events);
            }
            CardId::VectorLock => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 6, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Expose, 2, events);
                self.gain_block(Actor::Player, 5, events);
            }
            CardId::VectorLockPlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 8, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Expose, 2, events);
                self.gain_block(Actor::Player, 6, events);
            }
            CardId::BreachSignal => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 7, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Expose, 2, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::BreachSignalPlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 9, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Expose, 2, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::AnchorLoop => {
                self.gain_block(Actor::Player, 14, events);
                queue.push_front(CombatCommand::DrawCards(2));
            }
            CardId::AnchorLoopPlus => {
                self.gain_block(Actor::Player, 17, events);
                queue.push_front(CombatCommand::DrawCards(2));
            }
            CardId::ExecutionBeam => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 20, events);
            }
            CardId::ExecutionBeamPlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 26, events);
            }
            CardId::ChainBarrage => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 8, events);
                if self.enemy_is_alive(target_enemy.unwrap()) {
                    self.damage_enemy(Actor::Player, target_enemy.unwrap(), 8, events);
                }
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Bleed, 2, events);
            }
            CardId::ChainBarragePlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 10, events);
                if self.enemy_is_alive(target_enemy.unwrap()) {
                    self.damage_enemy(Actor::Player, target_enemy.unwrap(), 10, events);
                }
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Bleed, 2, events);
            }
            CardId::FortressMatrix => {
                self.gain_block(Actor::Player, 16, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::FortressMatrixPlus => {
                self.gain_block(Actor::Player, 20, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::OverwatchGrid => {
                self.gain_block(Actor::Player, 18, events);
                queue.push_front(CombatCommand::DrawCards(2));
            }
            CardId::OverwatchGridPlus => {
                self.gain_block(Actor::Player, 22, events);
                queue.push_front(CombatCommand::DrawCards(2));
            }
            CardId::RiftDart => {
                let enemy = target_enemy.unwrap();
                let exposed = self.has_status(Actor::Enemy(enemy), StatusKind::Expose);
                self.damage_enemy(Actor::Player, enemy, 4, events);
                self.apply_enemy_status(enemy, StatusKind::Bleed, 1, events);
                if exposed {
                    queue.push_front(CombatCommand::DrawCards(1));
                }
            }
            CardId::RiftDartPlus => {
                let enemy = target_enemy.unwrap();
                let exposed = self.has_status(Actor::Enemy(enemy), StatusKind::Expose);
                self.damage_enemy(Actor::Player, enemy, 6, events);
                self.apply_enemy_status(enemy, StatusKind::Bleed, 1, events);
                if exposed {
                    queue.push_front(CombatCommand::DrawCards(1));
                }
            }
            CardId::MarkPulse => {
                let enemy = target_enemy.unwrap();
                let bleeding = self.has_status(Actor::Enemy(enemy), StatusKind::Bleed);
                self.apply_enemy_status(enemy, StatusKind::Expose, 1, events);
                if bleeding {
                    self.gain_block(Actor::Player, 4, events);
                }
            }
            CardId::MarkPulsePlus => {
                let enemy = target_enemy.unwrap();
                let bleeding = self.has_status(Actor::Enemy(enemy), StatusKind::Bleed);
                self.apply_enemy_status(enemy, StatusKind::Expose, 1, events);
                if bleeding {
                    self.gain_block(Actor::Player, 6, events);
                }
            }
            CardId::BraceCircuit => {
                let had_block = self.player.fighter.block > 0;
                self.gain_block(Actor::Player, 6, events);
                if had_block {
                    queue.push_front(CombatCommand::DrawCards(1));
                }
            }
            CardId::BraceCircuitPlus => {
                let had_block = self.player.fighter.block > 0;
                self.gain_block(Actor::Player, 8, events);
                if had_block {
                    queue.push_front(CombatCommand::DrawCards(1));
                }
            }
            CardId::FaultShot => {
                let enemy = target_enemy.unwrap();
                let primed = self.has_status(Actor::Enemy(enemy), StatusKind::Weak)
                    || self.has_status(Actor::Enemy(enemy), StatusKind::Frail);
                self.damage_enemy(Actor::Player, enemy, 5, events);
                if primed {
                    self.apply_status(Actor::Player, StatusKind::Strength, 1, events);
                }
            }
            CardId::FaultShotPlus => {
                let enemy = target_enemy.unwrap();
                let primed = self.has_status(Actor::Enemy(enemy), StatusKind::Weak)
                    || self.has_status(Actor::Enemy(enemy), StatusKind::Frail);
                self.damage_enemy(Actor::Player, enemy, 7, events);
                if primed {
                    self.apply_status(Actor::Player, StatusKind::Strength, 1, events);
                }
            }
            CardId::SeverArc => {
                let enemy = target_enemy.unwrap();
                let bleeding = self.has_status(Actor::Enemy(enemy), StatusKind::Bleed);
                self.damage_enemy(Actor::Player, enemy, 8, events);
                if bleeding && self.enemy_is_alive(enemy) {
                    self.damage_enemy(Actor::Player, enemy, 8, events);
                }
            }
            CardId::SeverArcPlus => {
                let enemy = target_enemy.unwrap();
                let bleeding = self.has_status(Actor::Enemy(enemy), StatusKind::Bleed);
                self.damage_enemy(Actor::Player, enemy, 10, events);
                if bleeding && self.enemy_is_alive(enemy) {
                    self.damage_enemy(Actor::Player, enemy, 10, events);
                }
            }
            CardId::Lockbreaker => {
                let enemy = target_enemy.unwrap();
                let exposed = self.has_status(Actor::Enemy(enemy), StatusKind::Expose);
                self.damage_enemy(Actor::Player, enemy, 6, events);
                if exposed {
                    self.apply_enemy_status(enemy, StatusKind::Weak, 1, events);
                    self.gain_block(Actor::Player, 6, events);
                }
            }
            CardId::LockbreakerPlus => {
                let enemy = target_enemy.unwrap();
                let exposed = self.has_status(Actor::Enemy(enemy), StatusKind::Expose);
                self.damage_enemy(Actor::Player, enemy, 8, events);
                if exposed {
                    self.apply_enemy_status(enemy, StatusKind::Weak, 1, events);
                    self.gain_block(Actor::Player, 8, events);
                }
            }
            CardId::CounterLattice => {
                let enemy = target_enemy.unwrap();
                let weakened = self.has_status(Actor::Enemy(enemy), StatusKind::Weak);
                self.damage_enemy(Actor::Player, enemy, 6, events);
                if weakened {
                    self.gain_energy(1);
                }
            }
            CardId::CounterLatticePlus => {
                let enemy = target_enemy.unwrap();
                let weakened = self.has_status(Actor::Enemy(enemy), StatusKind::Weak);
                self.damage_enemy(Actor::Player, enemy, 8, events);
                if weakened {
                    self.gain_energy(1);
                }
            }
            CardId::TerminalLoop => {
                let enemy = target_enemy.unwrap();
                let bleeding = self.has_status(Actor::Enemy(enemy), StatusKind::Bleed);
                let exposed = self.has_status(Actor::Enemy(enemy), StatusKind::Expose);
                self.damage_enemy(Actor::Player, enemy, 12, events);
                if bleeding {
                    queue.push_front(CombatCommand::DrawCards(1));
                }
                if exposed {
                    self.apply_status(Actor::Player, StatusKind::Strength, 1, events);
                }
            }
            CardId::TerminalLoopPlus => {
                let enemy = target_enemy.unwrap();
                let bleeding = self.has_status(Actor::Enemy(enemy), StatusKind::Bleed);
                let exposed = self.has_status(Actor::Enemy(enemy), StatusKind::Expose);
                self.damage_enemy(Actor::Player, enemy, 15, events);
                if bleeding {
                    queue.push_front(CombatCommand::DrawCards(1));
                }
                if exposed {
                    self.apply_status(Actor::Player, StatusKind::Strength, 2, events);
                }
            }
            CardId::ZeroPoint => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 10, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Expose, 2, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
            CardId::ZeroPointPlus => {
                self.damage_enemy(Actor::Player, target_enemy.unwrap(), 14, events);
                self.apply_enemy_status(target_enemy.unwrap(), StatusKind::Expose, 2, events);
                queue.push_front(CombatCommand::DrawCards(1));
            }
        }
    }

    fn end_turn(&mut self, actor: Actor, events: &mut Vec<CombatEvent>) {
        match actor {
            Actor::Player => {
                let discard_count = self.deck.hand.len();
                self.deck.discard_pile.append(&mut self.deck.hand);
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

        let expose = self.fighter(actor).statuses.expose;
        if expose > 0 {
            self.fighter_mut(actor).statuses.expose = expose.saturating_sub(1);
            events.push(CombatEvent::StatusTicked {
                actor,
                status: StatusKind::Expose,
                amount: self.fighter(actor).statuses.expose,
            });
        }

        let weak = self.fighter(actor).statuses.weak;
        if weak > 0 {
            self.fighter_mut(actor).statuses.weak = weak.saturating_sub(1);
            events.push(CombatEvent::StatusTicked {
                actor,
                status: StatusKind::Weak,
                amount: self.fighter(actor).statuses.weak,
            });
        }

        let frail = self.fighter(actor).statuses.frail;
        if frail > 0 {
            self.fighter_mut(actor).statuses.frail = frail.saturating_sub(1);
            events.push(CombatEvent::StatusTicked {
                actor,
                status: StatusKind::Frail,
                amount: self.fighter(actor).statuses.frail,
            });
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
            let Some(intent) = self.current_intent(enemy_index) else {
                continue;
            };
            let enemy_actor = Actor::Enemy(enemy_index);

            for hit_index in 0..intent.hits {
                if intent.damage <= 0 {
                    break;
                }
                self.enemy_attack(enemy_index, intent.damage, events);
                if hit_index + 1 < intent.hits && self.player.fighter.hp <= 0 {
                    break;
                }
            }

            if self.player.fighter.hp <= 0 {
                break;
            }

            if intent.gain_block > 0 {
                self.gain_block(enemy_actor, intent.gain_block, events);
            }

            if intent.gain_strength > 0 {
                self.apply_status(
                    enemy_actor,
                    StatusKind::Strength,
                    intent.gain_strength,
                    events,
                );
            }

            if intent.prime_bleed > 0 {
                if let Some(enemy) = self.enemy_mut(enemy_index) {
                    enemy.on_hit_bleed = intent.prime_bleed;
                }
                events.push(CombatEvent::EnemyPrimedBleed {
                    enemy_index,
                    amount: intent.prime_bleed,
                });
            }

            if intent.apply_expose > 0 {
                self.apply_status(
                    Actor::Player,
                    StatusKind::Expose,
                    intent.apply_expose,
                    events,
                );
            }

            if intent.apply_weak > 0 {
                self.apply_status(Actor::Player, StatusKind::Weak, intent.apply_weak, events);
            }

            if intent.apply_frail > 0 {
                self.apply_status(Actor::Player, StatusKind::Frail, intent.apply_frail, events);
            }

            if intent.apply_bleed > 0 {
                self.apply_status(Actor::Player, StatusKind::Bleed, intent.apply_bleed, events);
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

            if self.player.fighter.hp <= 0 {
                break;
            }
        }

        self.end_turn(Actor::Enemy(0), events);
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
            self.apply_status(Actor::Player, StatusKind::Bleed, bleed, events);
        }
    }

    fn draw_cards(&mut self, count: u8, events: &mut Vec<CombatEvent>) {
        for _ in 0..count {
            let Some(card) = self.draw_one(events) else {
                break;
            };

            if self.deck.hand.len() >= 10 {
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
        let adjusted_amount = scale_down_for_frail(amount, self.fighter(actor).statuses.frail);
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
        amount: u8,
        events: &mut Vec<CombatEvent>,
    ) {
        let statuses = &mut self.fighter_mut(actor).statuses;
        match status {
            StatusKind::Bleed => {
                statuses.bleed = statuses.bleed.saturating_add(amount);
            }
            StatusKind::Expose => {
                statuses.expose = statuses.expose.saturating_add(amount);
            }
            StatusKind::Weak => {
                statuses.weak = statuses.weak.saturating_add(amount);
            }
            StatusKind::Frail => {
                statuses.frail = statuses.frail.saturating_add(amount);
            }
            StatusKind::Strength => {
                statuses.strength = statuses.strength.saturating_add(amount);
            }
        }

        events.push(CombatEvent::StatusApplied {
            target: actor,
            status,
            amount,
        });
    }

    fn apply_enemy_status(
        &mut self,
        enemy_index: usize,
        status: StatusKind,
        amount: u8,
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
    let with_strength = base_amount.saturating_add(attacker.strength as i32);
    if with_strength <= 0 {
        return 0;
    }
    let weakened = scale_down_for_weak(with_strength, attacker.weak);
    if weakened <= 0 {
        return 0;
    }
    let expose_multiplier = 100 + (defender.expose as i32 * 50);
    weakened.saturating_mul(expose_multiplier).max(0) / 100
}

fn scale_down_for_weak(amount: i32, weak: u8) -> i32 {
    let multiplier = (100 - weak as i32 * 25).max(0);
    amount.saturating_mul(multiplier) / 100
}

fn scale_down_for_frail(amount: i32, frail: u8) -> i32 {
    let multiplier = (100 - frail as i32 * 25).max(0);
    amount.saturating_mul(multiplier) / 100
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
    fn reshuffles_discard_when_draw_pile_is_empty() {
        let mut state = blank_state();
        state.deck.draw_pile = vec![CardId::FlareSlash];
        state.deck.discard_pile = vec![CardId::GuardStep];

        let events = state.process_commands([CombatCommand::DrawCards(2)]);

        assert_eq!(state.deck.hand.len(), 2);
        assert!(events.contains(&CombatEvent::Reshuffled));
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
    fn expose_increases_damage_taken() {
        let mut state = blank_state();
        primary_enemy_mut(&mut state).fighter.statuses.expose = 1;

        let mut events = Vec::new();
        state.damage(Actor::Player, PRIMARY_ENEMY, 6, &mut events);

        assert_eq!(primary_enemy(&state).fighter.hp, 31);
        assert!(events.contains(&CombatEvent::DamageDealt {
            source: Actor::Player,
            target: PRIMARY_ENEMY,
            amount: 9,
        }));
    }

    #[test]
    fn weak_reduces_outgoing_damage_and_decays_at_end_of_turn() {
        let mut state = blank_state();
        state.player.fighter.statuses.weak = 1;

        let mut damage_events = Vec::new();
        state.damage(Actor::Player, PRIMARY_ENEMY, 8, &mut damage_events);

        assert_eq!(primary_enemy(&state).fighter.hp, 34);
        assert!(damage_events.contains(&CombatEvent::DamageDealt {
            source: Actor::Player,
            target: PRIMARY_ENEMY,
            amount: 6,
        }));

        let tick_events = state.process_commands([CombatCommand::ApplyEndOfTurn(Actor::Player)]);
        assert_eq!(state.player.fighter.statuses.weak, 0);
        assert!(tick_events.contains(&CombatEvent::StatusTicked {
            actor: Actor::Player,
            status: StatusKind::Weak,
            amount: 0,
        }));
    }

    #[test]
    fn strength_adds_damage_per_hit_and_persists() {
        let mut state = blank_state();
        state.player.fighter.statuses.strength = 2;
        state.deck.hand = vec![CardId::BurstArray];

        state.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(primary_enemy(&state).fighter.hp, 25);

        let tick_events = state.process_commands([CombatCommand::ApplyEndOfTurn(Actor::Player)]);
        assert_eq!(state.player.fighter.statuses.strength, 2);
        assert!(!tick_events.iter().any(|event| matches!(
            event,
            CombatEvent::StatusTicked {
                status: StatusKind::Strength,
                ..
            }
        )));
    }

    #[test]
    fn frail_reduces_block_gain_from_cards_and_enemy_intents() {
        let mut state = blank_state();
        state.player.fighter.statuses.frail = 1;
        primary_enemy_mut(&mut state).fighter.statuses.frail = 1;
        state.deck.hand = vec![CardId::GuardStep];

        state.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(Actor::Player),
        });

        assert_eq!(state.player.fighter.block, 3);

        let mut events = Vec::new();
        state.gain_block(PRIMARY_ENEMY, 8, &mut events);
        assert_eq!(primary_enemy(&state).fighter.block, 6);
        assert!(events.contains(&CombatEvent::BlockGained {
            actor: PRIMARY_ENEMY,
            amount: 6,
        }));
    }

    #[test]
    fn strength_weak_and_expose_apply_in_the_defined_order() {
        let mut state = blank_state();
        state.player.fighter.statuses.strength = 2;
        state.player.fighter.statuses.weak = 1;
        primary_enemy_mut(&mut state).fighter.statuses.expose = 1;

        let mut events = Vec::new();
        state.damage(Actor::Player, PRIMARY_ENEMY, 6, &mut events);

        assert_eq!(primary_enemy(&state).fighter.hp, 31);
        assert!(events.contains(&CombatEvent::DamageDealt {
            source: Actor::Player,
            target: PRIMARY_ENEMY,
            amount: 9,
        }));
    }

    #[test]
    fn bleed_still_ignores_strength_weak_and_expose() {
        let mut state = blank_state();
        state.player.fighter.statuses.bleed = 3;
        state.player.fighter.statuses.weak = 2;
        state.player.fighter.statuses.strength = 4;
        primary_enemy_mut(&mut state).fighter.statuses.expose = 2;

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

        assert_eq!(primary_enemy(&state).fighter.hp, 35);
        assert_eq!(primary_enemy(&state).fighter.statuses.bleed, 1);
    }

    #[test]
    fn signal_tap_plus_applies_expose_draws_and_gains_block() {
        let mut state = blank_state();
        state.deck.hand = vec![CardId::SignalTapPlus];
        state.deck.draw_pile = vec![CardId::FlareSlash];

        state.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(primary_enemy(&state).fighter.statuses.expose, 1);
        assert_eq!(state.player.fighter.block, 3);
        assert_eq!(state.deck.hand, vec![CardId::FlareSlash]);
    }

    #[test]
    fn pressure_point_applies_weak() {
        let mut state = blank_state();
        state.deck.hand = vec![CardId::PressurePoint];

        state.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(primary_enemy(&state).fighter.hp, 36);
        assert_eq!(primary_enemy(&state).fighter.statuses.weak, 1);
    }

    #[test]
    fn burst_array_hits_three_times() {
        let mut state = blank_state();
        state.deck.hand = vec![CardId::BurstArray];

        state.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(primary_enemy(&state).fighter.hp, 31);
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

        assert_eq!(state.player.fighter.block, 6);
        assert_eq!(state.deck.hand, vec![CardId::GuardStep]);
    }

    #[test]
    fn barrier_field_applies_frail_while_gaining_block() {
        let mut state = blank_state();
        state.deck.hand = vec![CardId::BarrierField];

        state.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(state.player.fighter.block, 10);
        assert_eq!(primary_enemy(&state).fighter.statuses.frail, 1);
    }

    #[test]
    fn tactical_burst_draws_and_gains_strength() {
        let mut state = blank_state();
        state.deck.hand = vec![CardId::TacticalBurst];
        state.deck.draw_pile = vec![CardId::GuardStep, CardId::FlareSlash];

        state.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: None,
        });

        assert_eq!(state.player.fighter.statuses.strength, 1);
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

        assert_eq!(primary_enemy(&state).fighter.hp, 32);
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

        assert_eq!(primary_enemy(&state).fighter.hp, 31);
        assert_eq!(primary_enemy(&state).fighter.statuses.bleed, 3);
    }

    #[test]
    fn vector_lock_combines_damage_expose_and_block() {
        let mut state = blank_state();
        state.deck.hand = vec![CardId::VectorLock];

        state.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(primary_enemy(&state).fighter.hp, 34);
        assert_eq!(primary_enemy(&state).fighter.statuses.expose, 2);
        assert_eq!(state.player.fighter.block, 5);
    }

    #[test]
    fn breach_signal_draws_and_applies_expose() {
        let mut state = blank_state();
        state.deck.hand = vec![CardId::BreachSignal];
        state.deck.draw_pile = vec![CardId::FlareSlash];

        state.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(primary_enemy(&state).fighter.hp, 33);
        assert_eq!(primary_enemy(&state).fighter.statuses.expose, 2);
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

        assert_eq!(primary_enemy(&state).fighter.hp, 24);
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

        assert_eq!(state.player.fighter.block, 18);
        assert_eq!(state.deck.hand.len(), 2);
        assert!(state.deck.hand.contains(&CardId::GuardStep));
        assert!(state.deck.hand.contains(&CardId::FlareSlash));
    }

    #[test]
    fn rift_dart_variants_respect_the_expose_draw_condition() {
        let mut primed = blank_state();
        primed.deck.hand = vec![CardId::RiftDart];
        primed.deck.draw_pile = vec![CardId::FlareSlash];
        primary_enemy_mut(&mut primed).fighter.statuses.expose = 1;

        primed.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(primary_enemy(&primed).fighter.statuses.bleed, 1);
        assert_eq!(primed.deck.hand, vec![CardId::FlareSlash]);

        let mut unprimed = blank_state();
        unprimed.deck.hand = vec![CardId::RiftDartPlus];
        unprimed.deck.draw_pile = vec![CardId::FlareSlash];

        unprimed.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(primary_enemy(&unprimed).fighter.statuses.bleed, 1);
        assert!(unprimed.deck.hand.is_empty());
    }

    #[test]
    fn mark_pulse_variants_grant_block_only_against_bleeding_targets() {
        let mut primed = blank_state();
        primed.deck.hand = vec![CardId::MarkPulsePlus];
        primary_enemy_mut(&mut primed).fighter.statuses.bleed = 1;

        primed.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(primed.player.fighter.block, 6);
        assert_eq!(primary_enemy(&primed).fighter.statuses.expose, 1);

        let mut unprimed = blank_state();
        unprimed.deck.hand = vec![CardId::MarkPulse];

        unprimed.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(unprimed.player.fighter.block, 0);
        assert_eq!(primary_enemy(&unprimed).fighter.statuses.expose, 1);
    }

    #[test]
    fn brace_circuit_variants_draw_only_when_block_already_exists() {
        let mut primed = blank_state();
        primed.deck.hand = vec![CardId::BraceCircuitPlus];
        primed.deck.draw_pile = vec![CardId::GuardStep];
        primed.player.fighter.block = 2;

        primed.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: None,
        });

        assert_eq!(primed.player.fighter.block, 10);
        assert_eq!(primed.deck.hand, vec![CardId::GuardStep]);

        let mut unprimed = blank_state();
        unprimed.deck.hand = vec![CardId::BraceCircuit];
        unprimed.deck.draw_pile = vec![CardId::GuardStep];

        unprimed.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: None,
        });

        assert_eq!(unprimed.player.fighter.block, 6);
        assert!(unprimed.deck.hand.is_empty());
    }

    #[test]
    fn fault_shot_variants_gain_strength_only_against_debuffed_targets() {
        let mut primed = blank_state();
        primed.deck.hand = vec![CardId::FaultShotPlus];
        primary_enemy_mut(&mut primed).fighter.statuses.weak = 1;

        primed.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(primed.player.fighter.statuses.strength, 1);

        let mut unprimed = blank_state();
        unprimed.deck.hand = vec![CardId::FaultShot];

        unprimed.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(unprimed.player.fighter.statuses.strength, 0);
    }

    #[test]
    fn sever_arc_variants_hit_twice_only_against_bleeding_targets() {
        let mut primed = blank_state();
        primed.deck.hand = vec![CardId::SeverArcPlus];
        primary_enemy_mut(&mut primed).fighter.statuses.bleed = 1;

        primed.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(primary_enemy(&primed).fighter.hp, 20);

        let mut unprimed = blank_state();
        unprimed.deck.hand = vec![CardId::SeverArc];

        unprimed.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(primary_enemy(&unprimed).fighter.hp, 32);
    }

    #[test]
    fn lockbreaker_variants_convert_expose_into_weak_and_block() {
        let mut primed = blank_state();
        primed.deck.hand = vec![CardId::LockbreakerPlus];
        primary_enemy_mut(&mut primed).fighter.statuses.expose = 1;

        primed.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(primed.player.fighter.block, 8);
        assert_eq!(primary_enemy(&primed).fighter.statuses.weak, 1);

        let mut unprimed = blank_state();
        unprimed.deck.hand = vec![CardId::Lockbreaker];

        unprimed.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(unprimed.player.fighter.block, 0);
        assert_eq!(primary_enemy(&unprimed).fighter.statuses.weak, 0);
    }

    #[test]
    fn counter_lattice_variants_refund_energy_only_against_weakened_targets() {
        let mut primed = blank_state();
        primed.deck.hand = vec![CardId::CounterLatticePlus];
        primary_enemy_mut(&mut primed).fighter.statuses.weak = 1;

        primed.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(primed.player.energy, 3);
        assert_eq!(primary_enemy(&primed).fighter.hp, 32);

        let mut unprimed = blank_state();
        unprimed.deck.hand = vec![CardId::CounterLattice];

        unprimed.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(unprimed.player.energy, 2);
        assert_eq!(primary_enemy(&unprimed).fighter.hp, 34);
    }

    #[test]
    fn terminal_loop_variants_reward_bleed_and_expose_setup() {
        let mut primed = blank_state();
        primed.deck.hand = vec![CardId::TerminalLoopPlus];
        primed.deck.draw_pile = vec![CardId::GuardStep];
        primary_enemy_mut(&mut primed).fighter.statuses.bleed = 1;
        primary_enemy_mut(&mut primed).fighter.statuses.expose = 1;

        primed.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(primed.player.fighter.statuses.strength, 2);
        assert_eq!(primed.deck.hand, vec![CardId::GuardStep]);

        let mut unprimed = blank_state();
        unprimed.deck.hand = vec![CardId::TerminalLoop];
        unprimed.deck.draw_pile = vec![CardId::GuardStep];

        unprimed.dispatch(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(PRIMARY_ENEMY),
        });

        assert_eq!(unprimed.player.fighter.statuses.strength, 0);
        assert!(unprimed.deck.hand.is_empty());
    }

    #[test]
    fn scout_drone_brace_cycle_grants_strength_and_block() {
        let mut state = blank_state();
        set_primary_enemy_intent(&mut state, EnemyProfileId::ScoutDrone, 2);

        let mut events = Vec::new();
        state.resolve_enemy_intent(&mut events);

        assert_eq!(primary_enemy(&state).fighter.block, 4);
        assert_eq!(primary_enemy(&state).fighter.statuses.strength, 1);
        assert!(events.contains(&CombatEvent::StatusApplied {
            target: PRIMARY_ENEMY,
            status: StatusKind::Strength,
            amount: 1,
        }));
    }

    #[test]
    fn rampart_drone_pressure_clamp_applies_weak() {
        let mut state = blank_state();
        set_primary_enemy_intent(&mut state, EnemyProfileId::RampartDrone, 1);

        state.resolve_enemy_intent(&mut Vec::new());

        assert_eq!(state.player.fighter.hp, 35);
        assert_eq!(state.player.fighter.statuses.weak, 1);
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
    fn shard_weaver_refocus_applies_frail() {
        let mut state = blank_state();
        set_primary_enemy_intent(&mut state, EnemyProfileId::ShardWeaver, 2);

        state.resolve_enemy_intent(&mut Vec::new());

        assert_eq!(primary_enemy(&state).fighter.block, 8);
        assert_eq!(state.player.fighter.statuses.frail, 1);
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

        assert_eq!(state.player.fighter.block, 14);
        assert_eq!(state.deck.hand.len(), 2);
        assert!(state.deck.hand.contains(&CardId::GuardStep));
        assert!(state.deck.hand.contains(&CardId::FlareSlash));
    }
}
