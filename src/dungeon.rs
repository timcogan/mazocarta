use crate::combat::{DEFAULT_PLAYER_HP, EncounterEnemySetup, EncounterSetup};
use crate::content::CardId;
use crate::content::EnemyProfileId;
use crate::content::EventChoiceEffect;
use crate::content::EventId;
use crate::content::Language;
use crate::content::ModuleId;
use crate::content::localized_text;
use crate::content::ordered_events_for_level;
use crate::content::starter_deck;
use crate::content::upgraded_card;
use crate::rng::XorShift64;
use std::collections::{BTreeMap, BTreeSet};

const START_HP: i32 = DEFAULT_PLAYER_HP;
const REST_HEAL: i32 = 6;
const DUNGEON_LEVEL_COUNT: usize = 3;
const MAP_COLUMNS: usize = 7;
const MAP_REGULAR_DEPTHS: usize = 9;
const MAP_MAX_SEGMENT_MERGES: usize = 2;
const MAP_MAX_SEGMENTS_WITHOUT_BRIDGE: usize = 1;
const MAP_MAX_BRIDGE_SPAN: usize = 2;
const COMBAT_CREDITS_REWARD: u32 = 6;
const ELITE_CREDITS_REWARD: u32 = 14;
const BOSS_CREDITS_REWARD: u32 = 28;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RoomKind {
    Start,
    Combat,
    Elite,
    Rest,
    Shop,
    Event,
    Boss,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonNode {
    pub(crate) id: usize,
    pub(crate) depth: usize,
    pub(crate) lane: usize,
    pub(crate) kind: RoomKind,
    pub(crate) next: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct DungeonRun {
    pub(crate) seed: u64,
    pub(crate) current_level: usize,
    pub(crate) nodes: Vec<DungeonNode>,
    pub(crate) current_node: Option<usize>,
    pub(crate) available_nodes: Vec<usize>,
    pub(crate) visited_nodes: Vec<usize>,
    pub(crate) deck: Vec<CardId>,
    pub(crate) modules: Vec<ModuleId>,
    pub(crate) player_hp: i32,
    pub(crate) player_max_hp: i32,
    pub(crate) credits: u32,
    pub(crate) combats_cleared: usize,
    pub(crate) elites_cleared: usize,
    pub(crate) rests_completed: usize,
    pub(crate) bosses_cleared: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DungeonProgress {
    Continue,
    Completed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum NodeSelection {
    Encounter(EncounterSetup),
    Rest,
    Shop,
    Event(EventId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EventResolution {
    Credits { hp_lost: i32, credits_gained: u32 },
    Heal { healed: i32 },
    MaxHp { hp_lost: i32, max_hp_gained: i32 },
    Card { hp_lost: i32, card: CardId },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LevelBriefing {
    pub(crate) codename: &'static str,
    pub(crate) summary: &'static str,
    pub(crate) combat_enemy: EnemyProfileId,
    pub(crate) elite_enemy: EnemyProfileId,
    pub(crate) boss_enemy: EnemyProfileId,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LevelProfile {
    briefing: LevelBriefing,
    combat_enemy_table: &'static [EnemyProfileId],
    elite_enemy_table: &'static [EnemyProfileId],
    path_count_min: usize,
    path_count_max: usize,
    start_lane_window: usize,
    safe_combat_depths: usize,
    pre_boss_rest_roll: u64,
    elite_roll: u64,
    rest_roll: u64,
    force_elite_min_depth: usize,
    force_rest_min_depth: usize,
    combat_hp_base: i32,
    combat_hp_per_depth: i32,
    combat_block: i32,
    combat_intent_shift: usize,
    elite_hp_base: i32,
    elite_hp_per_depth: i32,
    elite_block: i32,
    boss_hp: i32,
    boss_block: i32,
}

impl LevelProfile {
    fn combat_hp(self, depth: usize) -> i32 {
        self.combat_hp_base + depth as i32 * self.combat_hp_per_depth
    }

    fn elite_hp(self, depth: usize) -> i32 {
        self.elite_hp_base + depth as i32 * self.elite_hp_per_depth
    }
}

fn level_profile(level: usize) -> LevelProfile {
    match level.clamp(1, DUNGEON_LEVEL_COUNT) {
        1 => LevelProfile {
            briefing: LevelBriefing {
                codename: "Relay Fringe",
                summary: "Fast scouts probe the outer defenses before the Penta Core punishes greedy turns.",
                combat_enemy: EnemyProfileId::ScoutDrone,
                elite_enemy: EnemyProfileId::RampartDrone,
                boss_enemy: EnemyProfileId::PentaCore,
            },
            combat_enemy_table: &[EnemyProfileId::ScoutDrone, EnemyProfileId::NeedlerDrone],
            elite_enemy_table: &[EnemyProfileId::RampartDrone, EnemyProfileId::SpineSentry],
            path_count_min: 2,
            path_count_max: 2,
            start_lane_window: 3,
            safe_combat_depths: 3,
            pre_boss_rest_roll: 680,
            elite_roll: 105,
            rest_roll: 340,
            force_elite_min_depth: 4,
            force_rest_min_depth: 5,
            combat_hp_base: 28,
            combat_hp_per_depth: 4,
            combat_block: 0,
            combat_intent_shift: 0,
            elite_hp_base: 44,
            elite_hp_per_depth: 5,
            elite_block: 3,
            boss_hp: 72,
            boss_block: 6,
        },
        2 => LevelProfile {
            briefing: LevelBriefing {
                codename: "Fracture Span",
                summary: "Sharper pressure and exposed openings set up the Hexarch Core's burst turns.",
                combat_enemy: EnemyProfileId::VoltMantis,
                elite_enemy: EnemyProfileId::PrismArray,
                boss_enemy: EnemyProfileId::HexarchCore,
            },
            combat_enemy_table: &[EnemyProfileId::VoltMantis, EnemyProfileId::ShardWeaver],
            elite_enemy_table: &[EnemyProfileId::PrismArray, EnemyProfileId::GlassBishop],
            path_count_min: 2,
            path_count_max: 3,
            start_lane_window: 4,
            safe_combat_depths: 2,
            pre_boss_rest_roll: 520,
            elite_roll: 155,
            rest_roll: 255,
            force_elite_min_depth: 4,
            force_rest_min_depth: 5,
            combat_hp_base: 34,
            combat_hp_per_depth: 5,
            combat_block: 1,
            combat_intent_shift: 1,
            elite_hp_base: 52,
            elite_hp_per_depth: 6,
            elite_block: 5,
            boss_hp: 88,
            boss_block: 10,
        },
        _ => LevelProfile {
            briefing: LevelBriefing {
                codename: "Null Vault",
                summary: "The final descent leans on sustained pressure before the Heptarch Core closes hard.",
                combat_enemy: EnemyProfileId::NullRaider,
                elite_enemy: EnemyProfileId::SiegeSpider,
                boss_enemy: EnemyProfileId::HeptarchCore,
            },
            combat_enemy_table: &[EnemyProfileId::NullRaider, EnemyProfileId::RiftStalker],
            elite_enemy_table: &[EnemyProfileId::SiegeSpider, EnemyProfileId::RiftBastion],
            path_count_min: 3,
            path_count_max: 3,
            start_lane_window: 5,
            safe_combat_depths: 1,
            pre_boss_rest_roll: 360,
            elite_roll: 220,
            rest_roll: 180,
            force_elite_min_depth: 3,
            force_rest_min_depth: 5,
            combat_hp_base: 42,
            combat_hp_per_depth: 6,
            combat_block: 2,
            combat_intent_shift: 2,
            elite_hp_base: 64,
            elite_hp_per_depth: 7,
            elite_block: 7,
            boss_hp: 108,
            boss_block: 12,
        },
    }
}

pub(crate) fn level_briefing(level: usize) -> LevelBriefing {
    level_profile(level).briefing
}

pub(crate) fn localized_level_codename(level: usize, language: Language) -> &'static str {
    match level.clamp(1, DUNGEON_LEVEL_COUNT) {
        1 => localized_text(language, "Relay Fringe", "Frontera de Relés"),
        2 => localized_text(language, "Fracture Span", "Paso de Fractura"),
        _ => localized_text(language, "Null Vault", "Bóveda Null"),
    }
}

pub(crate) fn localized_level_summary(level: usize, language: Language) -> &'static str {
    match level.clamp(1, DUNGEON_LEVEL_COUNT) {
        1 => localized_text(
            language,
            "Fast scouts probe the outer defenses before the Penta Core punishes greedy turns.",
            "Exploradores veloces tantean las defensas exteriores antes de que el Núcleo Penta castigue los turnos codiciosos.",
        ),
        2 => localized_text(
            language,
            "Sharper pressure and exposed openings set up the Hexarch Core's burst turns.",
            "Más presión y aperturas marcadas preparan los turnos explosivos del Núcleo Hexarch.",
        ),
        _ => localized_text(
            language,
            "The final descent leans on sustained pressure before the Heptarch Core closes hard.",
            "El descenso final se apoya en una presión sostenida antes de que el Núcleo Heptarch remate con fuerza.",
        ),
    }
}

impl DungeonRun {
    pub(crate) fn new(seed: u64) -> Self {
        let current_level = 1;
        let nodes = generate_nodes(level_seed(seed, current_level), current_level);
        let available_nodes = nodes
            .first()
            .map(|node| node.next.clone())
            .unwrap_or_default();

        Self {
            seed,
            current_level,
            nodes,
            current_node: None,
            available_nodes,
            visited_nodes: vec![0],
            deck: starter_deck(),
            modules: Vec::new(),
            player_hp: START_HP,
            player_max_hp: START_HP,
            credits: 0,
            combats_cleared: 0,
            elites_cleared: 0,
            rests_completed: 0,
            bosses_cleared: 0,
        }
    }

    pub(crate) fn current_level(&self) -> usize {
        self.current_level
    }

    pub(crate) fn total_levels(&self) -> usize {
        DUNGEON_LEVEL_COUNT
    }

    pub(crate) fn current_level_briefing(&self) -> LevelBriefing {
        level_briefing(self.current_level)
    }

    pub(crate) fn debug_set_level(&mut self, level: usize) -> bool {
        let clamped_level = level.clamp(1, DUNGEON_LEVEL_COUNT);
        if clamped_level == self.current_level {
            return false;
        }

        self.current_level = clamped_level;
        self.rebuild_current_level();
        true
    }

    fn current_level_profile(&self) -> LevelProfile {
        level_profile(self.current_level)
    }

    pub(crate) fn boss_symbol_sides(&self) -> usize {
        (self.current_level + 4).clamp(5, 7)
    }

    pub(crate) fn max_depth(&self) -> usize {
        self.nodes.iter().map(|node| node.depth).max().unwrap_or(0)
    }

    pub(crate) fn lane_count(&self) -> usize {
        self.nodes.iter().map(|node| node.lane).max().unwrap_or(0) + 1
    }

    pub(crate) fn is_available(&self, node_id: usize) -> bool {
        self.available_nodes.contains(&node_id)
    }

    pub(crate) fn is_visited(&self, node_id: usize) -> bool {
        self.visited_nodes.contains(&node_id)
    }

    pub(crate) fn node(&self, node_id: usize) -> Option<&DungeonNode> {
        self.nodes.get(node_id)
    }

    pub(crate) fn current_room_kind(&self) -> Option<RoomKind> {
        self.current_node
            .and_then(|node_id| self.node(node_id))
            .map(|node| node.kind)
    }

    pub(crate) fn current_room_seed(&self) -> Option<u64> {
        self.current_node.map(|node_id| {
            level_seed(self.seed, self.current_level)
                .wrapping_add((node_id as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15))
        })
    }

    pub(crate) fn current_encounter_setup(&self) -> Option<EncounterSetup> {
        let node = self.current_node.and_then(|node_id| self.node(node_id))?;
        match node.kind {
            RoomKind::Combat | RoomKind::Elite | RoomKind::Boss => {
                Some(self.encounter_setup_for(node))
            }
            RoomKind::Start | RoomKind::Rest | RoomKind::Shop | RoomKind::Event => None,
        }
    }

    fn ordered_event_node_ids(&self) -> Vec<usize> {
        let mut node_ids: Vec<_> = self
            .nodes
            .iter()
            .filter(|node| matches!(node.kind, RoomKind::Event))
            .map(|node| (node.depth, node.lane, node.id))
            .collect();
        node_ids.sort_unstable();
        node_ids.into_iter().map(|(_, _, id)| id).collect()
    }

    pub(crate) fn event_for_node(&self, node_id: usize) -> Option<EventId> {
        let event_index = self
            .ordered_event_node_ids()
            .into_iter()
            .position(|event_node_id| event_node_id == node_id)?;
        ordered_events_for_level(self.seed, self.current_level)
            .get(event_index)
            .copied()
    }

    pub(crate) fn add_card(&mut self, card: CardId) {
        self.deck.push(card);
    }

    pub(crate) fn add_module(&mut self, module: ModuleId) {
        if !self.modules.contains(&module) {
            self.modules.push(module);
        }
    }

    pub(crate) fn has_module(&self, module: ModuleId) -> bool {
        self.modules.contains(&module)
    }

    pub(crate) fn recover_hp(&mut self, amount: i32) -> i32 {
        if amount <= 0 {
            return 0;
        }

        let healed = (self.player_max_hp - self.player_hp).max(0).min(amount);
        self.player_hp += healed;
        healed
    }

    pub(crate) fn rest_heal_amount(&self) -> i32 {
        (self.player_max_hp - self.player_hp).clamp(0, REST_HEAL)
    }

    pub(crate) fn upgradable_card_indices(&self) -> Vec<usize> {
        let mut seen = Vec::new();
        let mut indices = Vec::new();
        for (index, &card) in self.deck.iter().enumerate() {
            if upgraded_card(card).is_some() && !seen.contains(&card) {
                seen.push(card);
                indices.push(index);
            }
        }
        indices
    }

    pub(crate) fn select_node(&mut self, node_id: usize) -> Option<NodeSelection> {
        if !self.is_available(node_id) {
            return None;
        }

        let node = self.node(node_id)?.clone();
        let selection = match node.kind {
            RoomKind::Start => None,
            RoomKind::Rest => Some(NodeSelection::Rest),
            RoomKind::Shop => Some(NodeSelection::Shop),
            RoomKind::Event => self.event_for_node(node.id).map(NodeSelection::Event),
            RoomKind::Combat | RoomKind::Elite | RoomKind::Boss => {
                Some(NodeSelection::Encounter(self.encounter_setup_for(&node)))
            }
        };
        if selection.is_some() {
            self.current_node = Some(node_id);
        }
        selection
    }

    pub(crate) fn debug_select_node(&mut self, node_id: usize) -> Option<NodeSelection> {
        let node = self.node(node_id)?.clone();
        let selection = match node.kind {
            RoomKind::Start => None,
            RoomKind::Rest => Some(NodeSelection::Rest),
            RoomKind::Shop => Some(NodeSelection::Shop),
            RoomKind::Event => self.event_for_node(node.id).map(NodeSelection::Event),
            RoomKind::Combat | RoomKind::Elite | RoomKind::Boss => {
                Some(NodeSelection::Encounter(self.encounter_setup_for(&node)))
            }
        };
        if selection.is_some() {
            self.current_node = Some(node_id);
        }
        selection
    }

    pub(crate) fn resolve_rest_heal(&mut self) -> Option<(i32, DungeonProgress)> {
        let healed = self.rest_heal_amount();
        self.player_hp += healed;
        let progress = self.complete_current_node()?;
        Some((healed, progress))
    }

    pub(crate) fn resolve_rest_upgrade(
        &mut self,
        deck_index: usize,
    ) -> Option<(CardId, CardId, DungeonProgress)> {
        let from = *self.deck.get(deck_index)?;
        let to = upgraded_card(from)?;
        self.deck[deck_index] = to;
        let progress = self.complete_current_node()?;
        Some((from, to, progress))
    }

    pub(crate) fn can_afford_shop_price(&self, price: u32) -> bool {
        self.credits >= price
    }

    pub(crate) fn resolve_shop_purchase(
        &mut self,
        card: CardId,
        price: u32,
    ) -> Option<DungeonProgress> {
        if !matches!(self.current_room_kind(), Some(RoomKind::Shop))
            || !self.can_afford_shop_price(price)
        {
            return None;
        }

        self.credits -= price;
        self.deck.push(card);
        self.complete_current_node()
    }

    pub(crate) fn resolve_shop_leave(&mut self) -> Option<DungeonProgress> {
        if !matches!(self.current_room_kind(), Some(RoomKind::Shop)) {
            return None;
        }

        self.complete_current_node()
    }

    pub(crate) fn resolve_event_choice(
        &mut self,
        event: EventId,
        choice_index: usize,
    ) -> Option<(EventResolution, DungeonProgress)> {
        if !matches!(self.current_room_kind(), Some(RoomKind::Event)) {
            return None;
        }

        let current_event = self
            .current_node
            .and_then(|node_id| self.event_for_node(node_id))?;
        if current_event != event {
            return None;
        }

        let effect =
            crate::content::event_choice_effect(current_event, choice_index, self.current_level)?;
        let resolution = match effect {
            EventChoiceEffect::GainCredits(credits) => {
                self.credits = self.credits.saturating_add(credits);
                EventResolution::Credits {
                    hp_lost: 0,
                    credits_gained: credits,
                }
            }
            EventChoiceEffect::LoseHpGainCredits {
                lose_hp,
                gain_credits,
            } => {
                let hp_lost = self.lose_hp_preserving_one(lose_hp);
                self.credits = self.credits.saturating_add(gain_credits);
                EventResolution::Credits {
                    hp_lost,
                    credits_gained: gain_credits,
                }
            }
            EventChoiceEffect::Heal(amount) => {
                let healed = self.recover_hp(amount);
                EventResolution::Heal { healed }
            }
            EventChoiceEffect::LoseHpGainMaxHp {
                lose_hp,
                gain_max_hp,
            } => {
                let hp_lost = self.lose_hp_preserving_one(lose_hp);
                self.player_max_hp += gain_max_hp;
                self.player_hp = (self.player_hp + gain_max_hp).min(self.player_max_hp);
                EventResolution::MaxHp {
                    hp_lost,
                    max_hp_gained: gain_max_hp,
                }
            }
            EventChoiceEffect::AddCard(card) => {
                self.deck.push(card);
                EventResolution::Card { hp_lost: 0, card }
            }
            EventChoiceEffect::LoseHpAddCard { lose_hp, card } => {
                let hp_lost = self.lose_hp_preserving_one(lose_hp);
                self.deck.push(card);
                EventResolution::Card { hp_lost, card }
            }
        };

        let progress = self.complete_current_node()?;
        Some((resolution, progress))
    }

    pub(crate) fn resolve_combat_victory(
        &mut self,
        player_hp: i32,
    ) -> Option<(DungeonProgress, u32)> {
        let credits_gained = self
            .current_room_kind()
            .map(credits_reward_for_room)
            .unwrap_or(0);
        self.player_hp = player_hp.clamp(0, self.player_max_hp);
        self.credits = self.credits.saturating_add(credits_gained);
        let progress = self.complete_current_node()?;
        Some((progress, credits_gained))
    }

    pub(crate) fn resolve_combat_defeat(&mut self, player_hp: i32) {
        self.player_hp = player_hp.max(0);
    }

    fn lose_hp_preserving_one(&mut self, amount: i32) -> i32 {
        if amount <= 0 {
            return 0;
        }

        let lost = (self.player_hp - 1).max(0).min(amount);
        self.player_hp -= lost;
        lost
    }

    fn complete_current_node(&mut self) -> Option<DungeonProgress> {
        let node_id = self.current_node.take()?;
        let node = self.nodes.get(node_id)?.clone();
        if !self.is_visited(node_id) {
            self.visited_nodes.push(node_id);
            self.record_completed_room(node.kind);
        }
        let next = node.next.clone();
        if !next.is_empty() {
            self.available_nodes = next;
            return Some(DungeonProgress::Continue);
        }

        if matches!(node.kind, RoomKind::Boss) && self.current_level < DUNGEON_LEVEL_COUNT {
            self.advance_level();
            return Some(DungeonProgress::Continue);
        }

        self.available_nodes.clear();
        Some(DungeonProgress::Completed)
    }

    fn encounter_setup_for(&self, node: &DungeonNode) -> EncounterSetup {
        let profile = self.current_level_profile();
        let (enemy_hp, enemy_block, enemy_intent_index) = match node.kind {
            RoomKind::Start => (profile.combat_hp_base, 0, profile.combat_intent_shift % 3),
            RoomKind::Combat => (
                profile.combat_hp(node.depth),
                profile.combat_block,
                (node.lane + node.depth + profile.combat_intent_shift) % 3,
            ),
            RoomKind::Elite => (profile.elite_hp(node.depth), profile.elite_block, 1),
            RoomKind::Rest => (profile.combat_hp_base, 0, profile.combat_intent_shift % 3),
            RoomKind::Shop => (profile.combat_hp_base, 0, profile.combat_intent_shift % 3),
            RoomKind::Event => (profile.combat_hp_base, 0, profile.combat_intent_shift % 3),
            RoomKind::Boss => (profile.boss_hp, profile.boss_block, 0),
        };

        EncounterSetup {
            player_hp: self.player_hp.max(1),
            player_max_hp: self.player_max_hp,
            player_max_energy: 3,
            enemies: self.encounter_enemies_for(node, enemy_hp, enemy_block, enemy_intent_index),
        }
    }

    fn encounter_enemies_for(
        &self,
        node: &DungeonNode,
        enemy_hp: i32,
        enemy_block: i32,
        enemy_intent_index: usize,
    ) -> Vec<EncounterEnemySetup> {
        let primary_profile = self.encounter_enemy_profile(node);

        if matches!(node.kind, RoomKind::Combat) && self.should_spawn_dual_combat(node) {
            let secondary_profile = self.secondary_combat_enemy_profile(node, primary_profile);
            let primary_hp = ((enemy_hp * 7) / 10).max(16);
            let secondary_hp = ((enemy_hp * 11) / 20).max(14);
            return vec![
                EncounterEnemySetup {
                    hp: primary_hp,
                    max_hp: primary_hp,
                    block: enemy_block,
                    profile: primary_profile,
                    intent_index: enemy_intent_index % 3,
                    on_hit_bleed: 0,
                },
                EncounterEnemySetup {
                    hp: secondary_hp,
                    max_hp: secondary_hp,
                    block: enemy_block.saturating_sub(1),
                    profile: secondary_profile,
                    intent_index: (enemy_intent_index + 1) % 3,
                    on_hit_bleed: 0,
                },
            ];
        }

        vec![EncounterEnemySetup {
            hp: enemy_hp,
            max_hp: enemy_hp,
            block: enemy_block,
            profile: primary_profile,
            intent_index: enemy_intent_index % 3,
            on_hit_bleed: 0,
        }]
    }

    fn should_spawn_dual_combat(&self, node: &DungeonNode) -> bool {
        let profile = self.current_level_profile();
        node.depth >= profile.force_elite_min_depth && profile.combat_enemy_table.len() > 1
    }

    fn secondary_combat_enemy_profile(
        &self,
        node: &DungeonNode,
        primary_profile: EnemyProfileId,
    ) -> EnemyProfileId {
        let profile = self.current_level_profile();
        let unlocked_count = (1 + node.depth.saturating_sub(profile.safe_combat_depths))
            .min(profile.combat_enemy_table.len())
            .max(1);
        let available = &profile.combat_enemy_table[..unlocked_count];
        if available.len() <= 1 {
            return primary_profile;
        }

        let mut rng = XorShift64::new(
            level_seed(self.seed, self.current_level)
                ^ (node.id as u64 + 1).wrapping_mul(0xD2B7_4407_B1CE_6E93)
                ^ (node.depth as u64 + 1).wrapping_mul(0x94D0_49BB_1331_11EB)
                ^ (node.lane as u64 + 1).wrapping_mul(0x517C_C1B7_2722_0A95),
        );
        let mut candidates: Vec<_> = available
            .iter()
            .copied()
            .filter(|profile| *profile != primary_profile)
            .collect();
        if candidates.is_empty() {
            primary_profile
        } else {
            candidates.swap_remove(rng.next_index(candidates.len()))
        }
    }

    fn encounter_enemy_profile(&self, node: &DungeonNode) -> EnemyProfileId {
        let profile = self.current_level_profile();
        match node.kind {
            RoomKind::Combat => {
                let unlocked_count = (1 + node.depth.saturating_sub(profile.safe_combat_depths))
                    .min(profile.combat_enemy_table.len())
                    .max(1);
                let mut rng = XorShift64::new(
                    level_seed(self.seed, self.current_level)
                        ^ (node.id as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15)
                        ^ (node.depth as u64 + 1).wrapping_mul(0xBF58_476D_1CE4_E5B9)
                        ^ (node.lane as u64 + 1).wrapping_mul(0x94D0_49BB_1331_11EB),
                );
                profile.combat_enemy_table[rng.next_index(unlocked_count)]
            }
            RoomKind::Elite => {
                let mut rng = XorShift64::new(
                    level_seed(self.seed, self.current_level)
                        ^ (node.id as u64 + 1).wrapping_mul(0x517C_C1B7_2722_0A95)
                        ^ (node.depth as u64 + 1).wrapping_mul(0x94D0_49BB_1331_11EB)
                        ^ (node.lane as u64 + 1).wrapping_mul(0xBF58_476D_1CE4_E5B9),
                );
                profile.elite_enemy_table[rng.next_index(profile.elite_enemy_table.len())]
            }
            RoomKind::Start | RoomKind::Rest | RoomKind::Shop | RoomKind::Event => {
                profile.briefing.combat_enemy
            }
            RoomKind::Boss => profile.briefing.boss_enemy,
        }
    }

    fn record_completed_room(&mut self, room_kind: RoomKind) {
        match room_kind {
            RoomKind::Start => {}
            RoomKind::Combat => self.combats_cleared += 1,
            RoomKind::Elite => self.elites_cleared += 1,
            RoomKind::Rest => self.rests_completed += 1,
            RoomKind::Shop => {}
            RoomKind::Event => {}
            RoomKind::Boss => self.bosses_cleared += 1,
        }
    }
}

impl DungeonRun {
    pub(crate) fn is_structurally_valid(&self) -> bool {
        if self.current_level == 0
            || self.current_level > DUNGEON_LEVEL_COUNT
            || self.nodes.is_empty()
        {
            return false;
        }

        if !self.nodes.iter().enumerate().all(|(index, node)| {
            node.id == index && node.next.iter().all(|next| *next < self.nodes.len())
        }) {
            return false;
        }

        if self
            .current_node
            .is_some_and(|node_id| node_id >= self.nodes.len())
        {
            return false;
        }
        if self
            .available_nodes
            .iter()
            .any(|node_id| *node_id >= self.nodes.len())
        {
            return false;
        }
        if self
            .visited_nodes
            .iter()
            .any(|node_id| *node_id >= self.nodes.len())
        {
            return false;
        }

        true
    }
}

fn level_seed(base_seed: u64, level: usize) -> u64 {
    if level <= 1 {
        base_seed
    } else {
        base_seed ^ (level as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ ((level as u64) << 17)
    }
}

pub(crate) fn credits_reward_for_room(kind: RoomKind) -> u32 {
    match kind {
        RoomKind::Combat => COMBAT_CREDITS_REWARD,
        RoomKind::Elite => ELITE_CREDITS_REWARD,
        RoomKind::Boss => BOSS_CREDITS_REWARD,
        RoomKind::Start | RoomKind::Rest | RoomKind::Shop | RoomKind::Event => 0,
    }
}

impl DungeonRun {
    fn rebuild_current_level(&mut self) {
        self.nodes = generate_nodes(
            level_seed(self.seed, self.current_level),
            self.current_level,
        );
        self.current_node = None;
        self.available_nodes = self
            .nodes
            .first()
            .map(|node| node.next.clone())
            .unwrap_or_default();
        self.visited_nodes = vec![0];
    }

    fn advance_level(&mut self) {
        self.current_level += 1;
        self.rebuild_current_level();
    }
}

fn generate_nodes(seed: u64, level: usize) -> Vec<DungeonNode> {
    let profile = level_profile(level);
    let start_lane = MAP_COLUMNS / 2;
    let boss_depth = MAP_REGULAR_DEPTHS + 1;
    let mut rng = XorShift64::new(seed ^ 0x5A17_5A17_12D0_4E55);
    let path_count = profile.path_count_min
        + rng.next_index(
            profile
                .path_count_max
                .saturating_sub(profile.path_count_min)
                + 1,
        );
    let mut start_lanes =
        distinct_lanes(&mut rng, MAP_COLUMNS, path_count, profile.start_lane_window);
    start_lanes.sort_unstable();

    let mut paths = Vec::with_capacity(start_lanes.len());
    let mut segment_edges: Vec<Vec<(usize, usize)>> =
        vec![Vec::new(); MAP_REGULAR_DEPTHS.saturating_sub(1)];
    for (path_index, &first_lane) in start_lanes.iter().enumerate() {
        let mut lanes = Vec::with_capacity(MAP_REGULAR_DEPTHS);
        let mut current = first_lane;
        lanes.push(current);
        for (segment_index, edges) in segment_edges.iter_mut().enumerate() {
            let next = choose_next_lane(
                &mut rng,
                current,
                edges,
                path_index,
                start_lanes.len(),
                segment_index,
            );
            edges.push((current, next));
            lanes.push(next);
            current = next;
        }
        paths.push(lanes);
    }

    let mut positions = BTreeSet::new();
    positions.insert((0usize, start_lane));
    positions.insert((boss_depth, start_lane));
    let mut raw_edges = BTreeSet::new();
    for &lane in &start_lanes {
        raw_edges.insert((0usize, start_lane, 1usize, lane));
    }
    for path in &paths {
        for (depth_index, &lane) in path.iter().enumerate() {
            positions.insert((depth_index + 1, lane));
            if let Some(&next_lane) = path.get(depth_index + 1) {
                raw_edges.insert((depth_index + 1, lane, depth_index + 2, next_lane));
            }
        }
        if let Some(&last_lane) = path.last() {
            raw_edges.insert((MAP_REGULAR_DEPTHS, last_lane, boss_depth, start_lane));
        }
    }
    bridge_edges(&positions, &mut raw_edges, &mut rng);

    let mut nodes: Vec<DungeonNode> = positions
        .into_iter()
        .enumerate()
        .map(|(id, (depth, lane))| DungeonNode {
            id,
            depth,
            lane,
            kind: RoomKind::Combat,
            next: Vec::new(),
        })
        .collect();
    let id_by_position: BTreeMap<(usize, usize), usize> = nodes
        .iter()
        .map(|node| ((node.depth, node.lane), node.id))
        .collect();
    let mut parents = vec![Vec::new(); nodes.len()];
    for (from_depth, from_lane, to_depth, to_lane) in raw_edges {
        let Some(&from_id) = id_by_position.get(&(from_depth, from_lane)) else {
            continue;
        };
        let Some(&to_id) = id_by_position.get(&(to_depth, to_lane)) else {
            continue;
        };
        if !nodes[from_id].next.contains(&to_id) {
            nodes[from_id].next.push(to_id);
        }
        parents[to_id].push(from_id);
    }
    for node in &mut nodes {
        node.next.sort_unstable();
    }

    assign_room_kinds(&mut nodes, &parents, &mut rng, profile);
    nodes
}

fn assign_room_kinds(
    nodes: &mut [DungeonNode],
    parents: &[Vec<usize>],
    rng: &mut XorShift64,
    profile: LevelProfile,
) {
    let boss_depth = nodes.iter().map(|node| node.depth).max().unwrap_or(0);
    let pre_boss_depth = boss_depth.saturating_sub(1);

    for index in 0..nodes.len() {
        let depth = nodes[index].depth;
        nodes[index].kind = if depth == 0 {
            RoomKind::Start
        } else if depth == boss_depth {
            RoomKind::Boss
        } else if depth == pre_boss_depth {
            let parent_has_rest = parents[index]
                .iter()
                .any(|parent| matches!(nodes[*parent].kind, RoomKind::Rest));
            let roll = rng.next_u64() % 1000;
            if !parent_has_rest && roll < profile.pre_boss_rest_roll {
                RoomKind::Rest
            } else {
                RoomKind::Combat
            }
        } else if depth <= profile.safe_combat_depths {
            RoomKind::Combat
        } else {
            let parent_has_elite = parents[index]
                .iter()
                .any(|parent| matches!(nodes[*parent].kind, RoomKind::Elite));
            let parent_has_rest = parents[index]
                .iter()
                .any(|parent| matches!(nodes[*parent].kind, RoomKind::Rest));
            let roll = rng.next_u64() % 1000;
            let allow_elite = depth >= profile.force_elite_min_depth
                && depth + 2 < boss_depth
                && !parent_has_elite;
            let allow_rest =
                depth >= profile.force_rest_min_depth && depth + 3 < boss_depth && !parent_has_rest;
            if allow_elite && roll < profile.elite_roll {
                RoomKind::Elite
            } else if allow_rest && roll < profile.rest_roll {
                RoomKind::Rest
            } else {
                RoomKind::Combat
            }
        };
    }

    ensure_kind_present(
        nodes,
        RoomKind::Elite,
        profile.force_elite_min_depth,
        pre_boss_depth.saturating_sub(1),
        rng,
    );
    ensure_kind_present(
        nodes,
        RoomKind::Rest,
        profile.force_rest_min_depth,
        pre_boss_depth.saturating_sub(2),
        rng,
    );
    ensure_shop_present(
        nodes,
        profile.force_elite_min_depth,
        pre_boss_depth.saturating_sub(2),
        rng,
    );
    ensure_event_count(
        nodes,
        2,
        profile.safe_combat_depths + 1,
        pre_boss_depth.saturating_sub(2),
        rng,
    );
}

fn ensure_kind_present(
    nodes: &mut [DungeonNode],
    desired: RoomKind,
    min_depth: usize,
    max_depth: usize,
    rng: &mut XorShift64,
) {
    if nodes
        .iter()
        .any(|node| node.kind == desired && node.depth >= min_depth && node.depth <= max_depth)
    {
        return;
    }

    let mut candidates: Vec<usize> = nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| {
            (node.depth >= min_depth
                && node.depth <= max_depth
                && matches!(node.kind, RoomKind::Combat))
            .then_some(index)
        })
        .collect();
    if candidates.is_empty() {
        return;
    }
    let choice = rng.next_index(candidates.len());
    nodes[candidates.swap_remove(choice)].kind = desired;
}

fn ensure_shop_present(
    nodes: &mut [DungeonNode],
    min_depth: usize,
    max_depth: usize,
    rng: &mut XorShift64,
) {
    if nodes.iter().any(|node| {
        node.kind == RoomKind::Shop && node.depth >= min_depth && node.depth <= max_depth
    }) {
        return;
    }

    let mut candidates: Vec<usize> = nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| {
            (node.depth >= min_depth
                && node.depth <= max_depth
                && matches!(node.kind, RoomKind::Combat))
            .then_some(index)
        })
        .collect();
    if candidates.is_empty() {
        return;
    }

    let choice = rng.next_index(candidates.len());
    nodes[candidates.swap_remove(choice)].kind = RoomKind::Shop;
}

fn ensure_event_count(
    nodes: &mut [DungeonNode],
    desired_count: usize,
    min_depth: usize,
    max_depth: usize,
    rng: &mut XorShift64,
) {
    let mut current_count = nodes
        .iter()
        .filter(|node| matches!(node.kind, RoomKind::Event))
        .count();
    if current_count >= desired_count {
        return;
    }

    let mut candidates: Vec<usize> = nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| {
            (node.depth >= min_depth
                && node.depth <= max_depth
                && matches!(node.kind, RoomKind::Combat))
            .then_some(index)
        })
        .collect();

    while current_count < desired_count && !candidates.is_empty() {
        let choice = rng.next_index(candidates.len());
        let node_index = candidates.swap_remove(choice);
        nodes[node_index].kind = RoomKind::Event;
        current_count += 1;
    }
}

fn choose_next_lane(
    rng: &mut XorShift64,
    current_lane: usize,
    existing_edges: &[(usize, usize)],
    path_index: usize,
    path_count: usize,
    segment_index: usize,
) -> usize {
    let required_targets = min_segment_target_count(path_count, segment_index);
    let unique_targets = distinct_target_lanes(existing_edges);
    let remaining_paths_after = path_count.saturating_sub(path_index + 1);
    let must_claim_new_target = unique_targets.len() + remaining_paths_after < required_targets;
    let existing_from_current: Vec<usize> = existing_edges
        .iter()
        .filter_map(|&(from_lane, to_lane)| (from_lane == current_lane).then_some(to_lane))
        .collect();
    if !must_claim_new_target
        && !existing_from_current.is_empty()
        && (existing_from_current.len() >= 2 || rng.next_u64() % 1000 < 820)
    {
        return existing_from_current[rng.next_index(existing_from_current.len())];
    }

    let mut candidates = Vec::with_capacity(3);
    if current_lane > 0 {
        candidates.push(current_lane - 1);
    }
    candidates.push(current_lane);
    if current_lane + 1 < MAP_COLUMNS {
        candidates.push(current_lane + 1);
    }

    choose_best_lane(
        rng,
        current_lane,
        existing_edges,
        &candidates,
        true,
        &unique_targets,
        must_claim_new_target,
    )
    .or_else(|| {
        choose_best_lane(
            rng,
            current_lane,
            existing_edges,
            &candidates,
            false,
            &unique_targets,
            must_claim_new_target,
        )
    })
    .unwrap_or_else(|| candidates[rng.next_index(candidates.len())])
}

fn edges_cross(from_a: usize, to_a: usize, from_b: usize, to_b: usize) -> bool {
    (from_a < from_b && to_a > to_b) || (from_a > from_b && to_a < to_b)
}

fn distinct_lanes(
    rng: &mut XorShift64,
    lane_count: usize,
    count: usize,
    start_lane_window: usize,
) -> Vec<usize> {
    let take = count.min(lane_count);
    let window = start_lane_window.clamp(take, lane_count);
    let window_start = if lane_count > window {
        rng.next_index(lane_count - window + 1)
    } else {
        0
    };
    let mut lanes: Vec<usize> = (window_start..window_start + window).collect();
    for index in (1..lanes.len()).rev() {
        let swap_with = rng.next_index(index + 1);
        lanes.swap(index, swap_with);
    }
    lanes.truncate(take.min(lanes.len()));
    lanes
}

fn bridge_edges(
    positions: &BTreeSet<(usize, usize)>,
    raw_edges: &mut BTreeSet<(usize, usize, usize, usize)>,
    rng: &mut XorShift64,
) {
    let mut lanes_by_depth: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for &(depth, lane) in positions {
        lanes_by_depth.entry(depth).or_default().push(lane);
    }
    for lanes in lanes_by_depth.values_mut() {
        lanes.sort_unstable();
        lanes.dedup();
    }

    let max_depth = lanes_by_depth.keys().copied().max().unwrap_or(0);
    let mut segments_without_bridge = 0usize;
    let mut lane_last_bridge_depth = vec![None; MAP_COLUMNS];
    let mut last_bridge_center = None;
    let mut previous_segment_had_bridge = false;
    for depth in 1..max_depth.saturating_sub(1) {
        let candidates = bridge_candidates(
            depth,
            &lanes_by_depth,
            raw_edges,
            &lane_last_bridge_depth,
            last_bridge_center,
        );
        let next_candidates = bridge_candidates(
            depth + 1,
            &lanes_by_depth,
            raw_edges,
            &lane_last_bridge_depth,
            last_bridge_center,
        );
        let must_bridge =
            !candidates.is_empty() && segments_without_bridge >= MAP_MAX_SEGMENTS_WITHOUT_BRIDGE;
        let should_preserve_bridge_window =
            !candidates.is_empty() && segments_without_bridge >= 1 && next_candidates.is_empty();
        let bridge_roll_threshold = if segments_without_bridge >= 1 {
            520
        } else {
            220
        };
        let should_add_bridge = must_bridge
            || should_preserve_bridge_window
            || (!candidates.is_empty()
                && !previous_segment_had_bridge
                && rng.next_u64() % 1000 < bridge_roll_threshold);

        if should_add_bridge {
            let best_score = candidates
                .first()
                .map(|candidate| candidate.score)
                .unwrap_or_default();
            let best_candidates: Vec<BridgeCandidate> = candidates
                .iter()
                .take_while(|candidate| candidate.score <= best_score + 4)
                .copied()
                .collect();
            let choice = best_candidates[rng.next_index(best_candidates.len())];
            raw_edges.insert((depth, choice.from_lane, depth + 1, choice.to_lane));
            segments_without_bridge = 0;
            previous_segment_had_bridge = true;
            last_bridge_center = Some(choice.center_x2);
            lane_last_bridge_depth[choice.from_lane] = Some(depth);
            lane_last_bridge_depth[choice.to_lane] = Some(depth);
        } else {
            segments_without_bridge += 1;
            previous_segment_had_bridge = false;
        }
    }
}

fn choose_best_lane(
    rng: &mut XorShift64,
    current_lane: usize,
    existing_edges: &[(usize, usize)],
    candidates: &[usize],
    enforce_merge_cap: bool,
    unique_targets: &[usize],
    must_claim_new_target: bool,
) -> Option<usize> {
    let mut best_lane = None;
    let mut best_score = u64::MAX;
    for lane in candidates.iter().copied() {
        if existing_edges
            .iter()
            .any(|&(from_lane, to_lane)| edges_cross(current_lane, lane, from_lane, to_lane))
        {
            continue;
        }

        let lane_usage = existing_edges
            .iter()
            .filter(|&&(_, to_lane)| to_lane == lane)
            .count();
        if enforce_merge_cap && lane_usage >= MAP_MAX_SEGMENT_MERGES {
            continue;
        }
        let is_new_target = !unique_targets.contains(&lane);
        if must_claim_new_target && !is_new_target {
            continue;
        }

        let drift = current_lane.abs_diff(lane) as u64;
        let center_bias = (lane as isize - (MAP_COLUMNS as isize / 2)).unsigned_abs() as u64;
        let merge_bonus = (lane_usage as u64).min(1) * 10;
        let diversity_bonus = if is_new_target { 9 } else { 0 };
        let score = (drift * 8 + center_bias * 4 + rng.next_u64() % 17)
            .saturating_sub(merge_bonus + diversity_bonus);
        if score < best_score {
            best_score = score;
            best_lane = Some(lane);
        }
    }
    best_lane
}

fn min_segment_target_count(path_count: usize, segment_index: usize) -> usize {
    if path_count <= 1 {
        1
    } else if segment_index == 0 || segment_index + 2 >= MAP_REGULAR_DEPTHS {
        path_count.min(2)
    } else {
        2
    }
}

fn distinct_target_lanes(existing_edges: &[(usize, usize)]) -> Vec<usize> {
    let mut targets: Vec<usize> = existing_edges.iter().map(|&(_, to_lane)| to_lane).collect();
    targets.sort_unstable();
    targets.dedup();
    targets
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BridgeCandidate {
    score: usize,
    from_lane: usize,
    to_lane: usize,
    center_x2: usize,
}

fn bridge_candidates(
    depth: usize,
    lanes_by_depth: &BTreeMap<usize, Vec<usize>>,
    raw_edges: &BTreeSet<(usize, usize, usize, usize)>,
    lane_last_bridge_depth: &[Option<usize>],
    last_bridge_center: Option<usize>,
) -> Vec<BridgeCandidate> {
    let Some(from_lanes) = lanes_by_depth.get(&depth) else {
        return Vec::new();
    };
    let Some(to_lanes) = lanes_by_depth.get(&(depth + 1)) else {
        return Vec::new();
    };

    let mut candidates = Vec::new();
    for &from_lane in from_lanes {
        let outgoing_count = raw_edges
            .iter()
            .filter(|&&(from_depth, lane, _, _)| from_depth == depth && lane == from_lane)
            .count();
        if outgoing_count >= 2 {
            continue;
        }

        for &to_lane in to_lanes {
            if from_lane.abs_diff(to_lane) > MAP_MAX_BRIDGE_SPAN {
                continue;
            }
            if raw_edges.contains(&(depth, from_lane, depth + 1, to_lane)) {
                continue;
            }
            let incoming_count = raw_edges
                .iter()
                .filter(|&&(_, _, to_depth, lane)| to_depth == depth + 1 && lane == to_lane)
                .count();
            if incoming_count > MAP_MAX_SEGMENT_MERGES {
                continue;
            }
            if raw_edges
                .iter()
                .any(|&(from_depth, lane_a, to_depth, lane_b)| {
                    from_depth == depth
                        && to_depth == depth + 1
                        && edges_cross(from_lane, to_lane, lane_a, lane_b)
                })
            {
                continue;
            }

            let span = from_lane.abs_diff(to_lane);
            let center_bias = (to_lane as isize - (MAP_COLUMNS as isize / 2)).unsigned_abs();
            let center_x2 = from_lane + to_lane;
            let repeat_center_penalty = last_bridge_center
                .map(|last_center| {
                    let distance = center_x2.abs_diff(last_center);
                    14usize.saturating_sub(distance * 6)
                })
                .unwrap_or(0);
            let freshness_penalty = [from_lane, to_lane]
                .into_iter()
                .filter_map(|lane| lane_last_bridge_depth.get(lane).and_then(|depth| *depth))
                .map(|last_depth| {
                    let age = depth.saturating_sub(last_depth);
                    18usize / age.max(1)
                })
                .sum::<usize>();
            let score = outgoing_count * 10
                + incoming_count * 8
                + span * 6
                + center_bias
                + repeat_center_penalty
                + freshness_penalty;
            candidates.push(BridgeCandidate {
                score,
                from_lane,
                to_lane,
                center_x2,
            });
        }
    }

    candidates.sort_by_key(|candidate| candidate.score);
    candidates
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::CombatState;
    use std::collections::{BTreeMap, BTreeSet, VecDeque};

    const TEST_RUN_SEED: u64 = 0x0BAD_5EED;
    const TEST_ALT_RUN_SEED: u64 = 0xDEAD_BEEF;
    const TEST_BOSS_COMBAT_SEED_ONE: u64 = 0xD00D_0001;
    const TEST_BOSS_COMBAT_SEED_TWO: u64 = 0xD00D_0002;
    const TEST_BOSS_COMBAT_SEED_THREE: u64 = 0xD00D_0003;

    #[test]
    fn generated_map_has_start_and_boss() {
        let run = DungeonRun::new(TEST_RUN_SEED);
        let level_one_profile = level_profile(1);

        assert_eq!(
            run.nodes.first().map(|node| node.kind),
            Some(RoomKind::Start)
        );
        assert!(
            run.nodes
                .iter()
                .any(|node| matches!(node.kind, RoomKind::Boss))
        );
        assert!(!run.available_nodes.is_empty());
        assert!(run.available_nodes.len() >= level_one_profile.path_count_min);
        assert!(run.available_nodes.len() <= level_one_profile.path_count_max);
    }

    #[test]
    fn new_runs_start_with_thirty_two_hp() {
        let run = DungeonRun::new(TEST_RUN_SEED);

        assert_eq!(run.player_hp, START_HP);
        assert_eq!(run.player_max_hp, START_HP);
        assert_eq!(START_HP, 32);
    }

    #[test]
    fn new_runs_start_without_modules_before_the_picker() {
        let run = DungeonRun::new(TEST_RUN_SEED);

        assert!(run.modules.is_empty());
    }

    #[test]
    fn rest_heal_remains_proportional_to_the_lower_starting_hp() {
        let mut run = DungeonRun::new(TEST_RUN_SEED);
        run.player_hp = 20;

        assert_eq!(run.rest_heal_amount(), 6);

        run.player_hp = 29;
        assert_eq!(run.rest_heal_amount(), 3);
    }

    #[test]
    fn generated_edges_only_advance_one_depth() {
        let run = DungeonRun::new(TEST_RUN_SEED);

        for node in &run.nodes {
            for next_id in &node.next {
                let next = &run.nodes[*next_id];
                assert_eq!(next.depth, node.depth + 1);
            }
        }
    }

    #[test]
    fn every_generated_node_is_reachable_from_start() {
        let run = DungeonRun::new(TEST_RUN_SEED);
        let mut seen = BTreeSet::new();
        let mut queue = VecDeque::from([0usize]);
        while let Some(node_id) = queue.pop_front() {
            if !seen.insert(node_id) {
                continue;
            }
            queue.extend(run.nodes[node_id].next.iter().copied());
        }

        assert_eq!(seen.len(), run.nodes.len());
    }

    #[test]
    fn generated_map_has_branching_connections() {
        let run = DungeonRun::new(TEST_RUN_SEED);

        assert!(run.nodes.iter().any(|node| node.next.len() > 1));
    }

    #[test]
    fn generated_map_keeps_outgoing_links_under_control() {
        let run = DungeonRun::new(TEST_RUN_SEED);

        assert!(
            run.nodes
                .iter()
                .filter(|node| !matches!(node.kind, RoomKind::Start))
                .all(|node| node.next.len() <= 2)
        );
    }

    #[test]
    fn rest_upgrade_options_collapse_duplicate_cards() {
        let run = DungeonRun::new(TEST_RUN_SEED);
        let options = run.upgradable_card_indices();

        assert_eq!(options.len(), 4);
    }

    #[test]
    fn resolving_rest_upgrade_replaces_one_card_in_the_deck() {
        let mut run = DungeonRun::new(TEST_RUN_SEED);
        run.nodes = vec![
            DungeonNode {
                id: 0,
                depth: 0,
                lane: 0,
                kind: RoomKind::Start,
                next: vec![1],
            },
            DungeonNode {
                id: 1,
                depth: 1,
                lane: 0,
                kind: RoomKind::Rest,
                next: vec![],
            },
        ];
        run.current_node = Some(1);

        let (from, to, progress) = run.resolve_rest_upgrade(0).unwrap();

        assert_eq!(from, CardId::FlareSlash);
        assert_eq!(to, CardId::FlareSlashPlus);
        assert_eq!(run.deck[0], CardId::FlareSlashPlus);
        assert_eq!(progress, DungeonProgress::Completed);
        assert_eq!(run.rests_completed, 1);
        assert_eq!(
            run.combats_cleared + run.elites_cleared + run.rests_completed + run.bosses_cleared,
            1
        );
    }

    #[test]
    fn combat_victory_increments_run_room_totals() {
        let mut run = DungeonRun::new(TEST_RUN_SEED);
        run.nodes = vec![
            DungeonNode {
                id: 0,
                depth: 0,
                lane: 0,
                kind: RoomKind::Start,
                next: vec![1],
            },
            DungeonNode {
                id: 1,
                depth: 1,
                lane: 0,
                kind: RoomKind::Combat,
                next: vec![],
            },
        ];
        run.current_node = Some(1);

        let (progress, credits_gained) = run.resolve_combat_victory(34).unwrap();

        assert_eq!(progress, DungeonProgress::Completed);
        assert_eq!(credits_gained, COMBAT_CREDITS_REWARD);
        assert_eq!(run.credits, COMBAT_CREDITS_REWARD);
        assert_eq!(run.combats_cleared, 1);
        assert_eq!(
            run.combats_cleared + run.elites_cleared + run.rests_completed + run.bosses_cleared,
            1
        );
    }

    #[test]
    fn debug_select_node_ignores_availability() {
        let mut run = DungeonRun::new(TEST_RUN_SEED);
        let target = run.nodes.last().map(|node| node.id).unwrap();

        assert!(run.select_node(target).is_none());
        assert!(run.debug_select_node(target).is_some());
    }

    #[test]
    fn selecting_an_event_node_returns_the_event_selection() {
        let mut run = DungeonRun::new(TEST_RUN_SEED);
        run.current_level = 2;
        run.nodes = vec![
            DungeonNode {
                id: 0,
                depth: 0,
                lane: 0,
                kind: RoomKind::Start,
                next: vec![1],
            },
            DungeonNode {
                id: 1,
                depth: 3,
                lane: 0,
                kind: RoomKind::Event,
                next: vec![],
            },
        ];
        run.available_nodes = vec![1];

        assert_eq!(
            run.select_node(1),
            Some(NodeSelection::Event(
                ordered_events_for_level(TEST_RUN_SEED, 2)[0]
            ))
        );
        assert_eq!(run.current_room_kind(), Some(RoomKind::Event));
    }

    #[test]
    fn event_nodes_in_the_same_level_map_to_distinct_events() {
        let mut run = DungeonRun::new(TEST_RUN_SEED);
        run.current_level = 3;
        run.nodes = vec![
            DungeonNode {
                id: 0,
                depth: 0,
                lane: 0,
                kind: RoomKind::Start,
                next: vec![1, 2],
            },
            DungeonNode {
                id: 1,
                depth: 3,
                lane: 1,
                kind: RoomKind::Event,
                next: vec![],
            },
            DungeonNode {
                id: 2,
                depth: 5,
                lane: 0,
                kind: RoomKind::Event,
                next: vec![],
            },
        ];

        let ordered_events = ordered_events_for_level(TEST_RUN_SEED, 3);

        assert_eq!(run.event_for_node(1), Some(ordered_events[0]));
        assert_eq!(run.event_for_node(2), Some(ordered_events[1]));
        assert_ne!(run.event_for_node(1), run.event_for_node(2));
    }

    #[test]
    fn selecting_an_unmapped_event_node_does_not_change_current_node() {
        let mut run = DungeonRun::new(TEST_RUN_SEED);
        run.nodes = vec![
            DungeonNode {
                id: 0,
                depth: 0,
                lane: 0,
                kind: RoomKind::Start,
                next: vec![3],
            },
            DungeonNode {
                id: 1,
                depth: 3,
                lane: 0,
                kind: RoomKind::Event,
                next: vec![],
            },
            DungeonNode {
                id: 2,
                depth: 4,
                lane: 0,
                kind: RoomKind::Event,
                next: vec![],
            },
            DungeonNode {
                id: 3,
                depth: 5,
                lane: 0,
                kind: RoomKind::Event,
                next: vec![],
            },
        ];
        run.current_node = Some(0);
        run.available_nodes = vec![3];

        assert_eq!(run.select_node(3), None);
        assert_eq!(run.current_node, Some(0));
    }

    #[test]
    fn debug_selecting_an_unmapped_event_node_does_not_change_current_node() {
        let mut run = DungeonRun::new(TEST_RUN_SEED);
        run.nodes = vec![
            DungeonNode {
                id: 0,
                depth: 0,
                lane: 0,
                kind: RoomKind::Start,
                next: vec![3],
            },
            DungeonNode {
                id: 1,
                depth: 3,
                lane: 0,
                kind: RoomKind::Event,
                next: vec![],
            },
            DungeonNode {
                id: 2,
                depth: 4,
                lane: 0,
                kind: RoomKind::Event,
                next: vec![],
            },
            DungeonNode {
                id: 3,
                depth: 5,
                lane: 0,
                kind: RoomKind::Event,
                next: vec![],
            },
        ];
        run.current_node = Some(0);

        assert_eq!(run.debug_select_node(3), None);
        assert_eq!(run.current_node, Some(0));
    }

    #[test]
    fn selecting_a_shop_node_returns_the_shop_selection() {
        let mut run = DungeonRun::new(TEST_RUN_SEED);
        run.nodes = vec![
            DungeonNode {
                id: 0,
                depth: 0,
                lane: 0,
                kind: RoomKind::Start,
                next: vec![1],
            },
            DungeonNode {
                id: 1,
                depth: 1,
                lane: 0,
                kind: RoomKind::Shop,
                next: vec![],
            },
        ];
        run.available_nodes = vec![1];

        assert_eq!(run.select_node(1), Some(NodeSelection::Shop));
        assert_eq!(run.current_room_kind(), Some(RoomKind::Shop));
    }

    #[test]
    fn risky_event_choices_preserve_one_hp() {
        let mut run = DungeonRun::new(TEST_RUN_SEED);
        run.player_hp = 4;
        run.nodes = vec![
            DungeonNode {
                id: 0,
                depth: 0,
                lane: 0,
                kind: RoomKind::Start,
                next: vec![1],
            },
            DungeonNode {
                id: 1,
                depth: 3,
                lane: 0,
                kind: RoomKind::Event,
                next: vec![],
            },
        ];
        run.current_node = Some(1);

        let (resolution, progress) = run
            .resolve_event_choice(EventId::SalvageCache, 1)
            .expect("event choice should resolve");

        assert_eq!(progress, DungeonProgress::Completed);
        assert_eq!(run.player_hp, 1);
        assert_eq!(run.credits, 28);
        assert_eq!(
            resolution,
            EventResolution::Credits {
                hp_lost: 3,
                credits_gained: 28,
            }
        );
    }

    #[test]
    fn event_choice_rejects_a_mismatched_event_id() {
        let mut run = DungeonRun::new(TEST_RUN_SEED);
        run.player_hp = 20;
        run.current_level = 1;
        run.nodes = vec![
            DungeonNode {
                id: 0,
                depth: 0,
                lane: 0,
                kind: RoomKind::Start,
                next: vec![1],
            },
            DungeonNode {
                id: 1,
                depth: 3,
                lane: 0,
                kind: RoomKind::Event,
                next: vec![],
            },
        ];
        run.current_node = Some(1);
        let wrong_event = ordered_events_for_level(TEST_RUN_SEED, 1)[1];

        assert_eq!(run.resolve_event_choice(wrong_event, 0), None);
        assert_eq!(run.player_hp, 20);
        assert_eq!(run.credits, 0);
        assert_eq!(run.current_node, Some(1));
    }

    #[test]
    fn shop_purchase_spends_credits_and_adds_the_card() {
        let mut run = DungeonRun::new(TEST_RUN_SEED);
        run.credits = 24;
        run.nodes = vec![
            DungeonNode {
                id: 0,
                depth: 0,
                lane: 0,
                kind: RoomKind::Start,
                next: vec![1],
            },
            DungeonNode {
                id: 1,
                depth: 1,
                lane: 0,
                kind: RoomKind::Shop,
                next: vec![],
            },
        ];
        run.current_node = Some(1);
        let initial_deck_len = run.deck.len();

        let progress = run
            .resolve_shop_purchase(CardId::BarrierField, 24)
            .expect("shop purchase should resolve");

        assert_eq!(progress, DungeonProgress::Completed);
        assert_eq!(run.credits, 0);
        assert_eq!(run.deck.len(), initial_deck_len + 1);
        assert_eq!(run.deck.last(), Some(&CardId::BarrierField));
    }

    #[test]
    fn shop_purchase_requires_enough_credits() {
        let mut run = DungeonRun::new(TEST_RUN_SEED);
        run.credits = 15;
        run.nodes = vec![
            DungeonNode {
                id: 0,
                depth: 0,
                lane: 0,
                kind: RoomKind::Start,
                next: vec![1],
            },
            DungeonNode {
                id: 1,
                depth: 1,
                lane: 0,
                kind: RoomKind::Shop,
                next: vec![],
            },
        ];
        run.current_node = Some(1);
        let initial_deck = run.deck.clone();

        assert_eq!(run.resolve_shop_purchase(CardId::BarrierField, 24), None);
        assert_eq!(run.credits, 15);
        assert_eq!(run.deck, initial_deck);
        assert_eq!(run.current_node, Some(1));
    }

    #[test]
    fn leaving_shop_completes_the_node_without_spending_credits() {
        let mut run = DungeonRun::new(TEST_RUN_SEED);
        run.credits = 24;
        run.nodes = vec![
            DungeonNode {
                id: 0,
                depth: 0,
                lane: 0,
                kind: RoomKind::Start,
                next: vec![1],
            },
            DungeonNode {
                id: 1,
                depth: 1,
                lane: 0,
                kind: RoomKind::Shop,
                next: vec![],
            },
        ];
        run.current_node = Some(1);
        let initial_deck = run.deck.clone();

        let progress = run.resolve_shop_leave().expect("shop leave should resolve");

        assert_eq!(progress, DungeonProgress::Completed);
        assert_eq!(run.credits, 24);
        assert_eq!(run.deck, initial_deck);
    }

    #[test]
    fn recover_hp_clamps_to_the_maximum() {
        let mut run = DungeonRun::new(TEST_RUN_SEED);
        run.player_hp = 30;

        assert_eq!(run.recover_hp(3), 2);
        assert_eq!(run.player_hp, 32);
    }

    #[test]
    fn current_room_seed_uses_wrapping_math() {
        let mut run = DungeonRun::new(TEST_RUN_SEED);
        run.current_level = 2;
        run.current_node = Some(1);

        assert_eq!(
            run.current_room_seed(),
            Some(level_seed(run.seed, 2).wrapping_add((2u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)))
        );
    }

    #[test]
    fn boss_completion_advances_to_the_next_level_before_the_final_boss() {
        let mut run = DungeonRun::new(TEST_RUN_SEED);
        run.nodes = vec![
            DungeonNode {
                id: 0,
                depth: 0,
                lane: 3,
                kind: RoomKind::Start,
                next: vec![1],
            },
            DungeonNode {
                id: 1,
                depth: 1,
                lane: 3,
                kind: RoomKind::Boss,
                next: vec![],
            },
        ];
        run.current_node = Some(1);
        run.available_nodes.clear();

        let (progress, credits_gained) = run.resolve_combat_victory(31).unwrap();

        assert_eq!(progress, DungeonProgress::Continue);
        assert_eq!(credits_gained, BOSS_CREDITS_REWARD);
        assert_eq!(run.credits, BOSS_CREDITS_REWARD);
        assert_eq!(run.current_level(), 2);
        assert_eq!(run.player_hp, 31);
        assert_eq!(run.bosses_cleared, 1);
        assert_eq!(
            run.combats_cleared + run.elites_cleared + run.rests_completed + run.bosses_cleared,
            1
        );
        assert_eq!(run.visited_nodes, vec![0]);
        assert!(matches!(
            run.nodes.first().map(|node| node.kind),
            Some(RoomKind::Start)
        ));
        assert!(
            run.nodes
                .iter()
                .any(|node| matches!(node.kind, RoomKind::Boss))
        );
        assert!(!run.available_nodes.is_empty());
    }

    #[test]
    fn final_boss_completion_finishes_the_run() {
        let mut run = DungeonRun::new(TEST_RUN_SEED);
        run.current_level = DUNGEON_LEVEL_COUNT;
        run.nodes = vec![
            DungeonNode {
                id: 0,
                depth: 0,
                lane: 3,
                kind: RoomKind::Start,
                next: vec![1],
            },
            DungeonNode {
                id: 1,
                depth: 1,
                lane: 3,
                kind: RoomKind::Boss,
                next: vec![],
            },
        ];
        run.current_node = Some(1);
        run.available_nodes.clear();

        let (progress, credits_gained) = run.resolve_combat_victory(22).unwrap();

        assert_eq!(progress, DungeonProgress::Completed);
        assert_eq!(credits_gained, BOSS_CREDITS_REWARD);
        assert_eq!(run.credits, BOSS_CREDITS_REWARD);
        assert_eq!(run.current_level(), DUNGEON_LEVEL_COUNT);
        assert!(run.available_nodes.is_empty());
    }

    #[test]
    fn elite_victory_awards_elite_credits() {
        let mut run = DungeonRun::new(TEST_RUN_SEED);
        run.nodes = vec![
            DungeonNode {
                id: 0,
                depth: 0,
                lane: 0,
                kind: RoomKind::Start,
                next: vec![1],
            },
            DungeonNode {
                id: 1,
                depth: 1,
                lane: 0,
                kind: RoomKind::Elite,
                next: vec![],
            },
        ];
        run.current_node = Some(1);

        let (progress, credits_gained) = run.resolve_combat_victory(29).unwrap();

        assert_eq!(progress, DungeonProgress::Completed);
        assert_eq!(credits_gained, ELITE_CREDITS_REWARD);
        assert_eq!(run.credits, ELITE_CREDITS_REWARD);
        assert_eq!(run.elites_cleared, 1);
    }

    #[test]
    fn boss_symbol_sides_follow_the_current_level() {
        let mut run = DungeonRun::new(TEST_RUN_SEED);
        assert_eq!(run.boss_symbol_sides(), 5);
        run.current_level = 2;
        assert_eq!(run.boss_symbol_sides(), 6);
        run.current_level = 3;
        assert_eq!(run.boss_symbol_sides(), 7);
    }

    #[test]
    fn debug_set_level_rebuilds_the_requested_map() {
        let mut run = DungeonRun::new(TEST_RUN_SEED);
        run.current_node = Some(3);
        run.available_nodes.clear();
        run.visited_nodes = vec![0, 1, 3];

        assert!(run.debug_set_level(3));
        assert_eq!(run.current_level(), 3);
        assert_eq!(run.current_node, None);
        assert_eq!(run.visited_nodes, vec![0]);
        assert_eq!(run.nodes, generate_nodes(level_seed(run.seed, 3), 3));
        assert_eq!(
            run.available_nodes,
            run.nodes
                .first()
                .map(|node| node.next.clone())
                .unwrap_or_default()
        );
        assert!(!run.debug_set_level(99));
        assert_eq!(run.current_level(), 3);
    }

    #[test]
    fn level_briefings_define_distinct_enemy_rosters() {
        let level_one = level_briefing(1);
        let level_two = level_briefing(2);
        let level_three = level_briefing(3);

        assert_eq!(level_one.combat_enemy, EnemyProfileId::ScoutDrone);
        assert_eq!(level_two.elite_enemy, EnemyProfileId::PrismArray);
        assert_eq!(level_three.boss_enemy, EnemyProfileId::HeptarchCore);
        assert_ne!(level_one.codename, level_two.codename);
        assert_ne!(level_two.codename, level_three.codename);
        assert!(level_one.summary.contains("greedy turns"));
        assert!(level_two.summary.contains("burst turns"));
        assert!(level_three.summary.contains("closes hard"));
    }

    #[test]
    fn elite_encounters_draw_from_the_act_elite_table() {
        let elite_node = DungeonNode {
            id: 7,
            depth: 5,
            lane: 2,
            kind: RoomKind::Elite,
            next: vec![],
        };

        for level in 1..=3 {
            let mut run = DungeonRun::new(TEST_RUN_SEED + level as u64);
            run.current_level = level;
            let profile = run.current_level_profile();
            let enemy = run.encounter_enemy_profile(&elite_node);

            assert!(profile.elite_enemy_table.contains(&enemy));
        }
    }

    #[test]
    fn elite_encounters_are_deterministic_for_the_same_seed_and_node() {
        let elite_node = DungeonNode {
            id: 7,
            depth: 5,
            lane: 2,
            kind: RoomKind::Elite,
            next: vec![],
        };
        let mut run = DungeonRun::new(TEST_RUN_SEED);
        run.current_level = 2;

        let first = run.encounter_enemy_profile(&elite_node);
        let second = run.encounter_enemy_profile(&elite_node);

        assert_eq!(first, second);
    }

    #[test]
    fn elite_encounters_unlock_multiple_profiles_per_act() {
        let elite_node = DungeonNode {
            id: 7,
            depth: 5,
            lane: 2,
            kind: RoomKind::Elite,
            next: vec![],
        };

        for level in 1..=3 {
            let mut seen = Vec::new();

            for seed in 0..64u64 {
                let mut run = DungeonRun::new(
                    seed.wrapping_mul(0xD6E8_FD50_19E3_7C4B)
                        .wrapping_add(level as u64 * 31),
                );
                run.current_level = level;
                let enemy = run.encounter_enemy_profile(&elite_node);
                if !seen.contains(&enemy) {
                    seen.push(enemy);
                }
            }

            assert!(
                seen.len() >= 2,
                "level {level} did not produce multiple elite profiles"
            );
        }
    }

    #[test]
    fn early_safe_combats_keep_the_signature_enemy() {
        for level in 1..=3 {
            for seed in 0..32u64 {
                let mut run = DungeonRun::new(
                    seed.wrapping_mul(0x9E37_79B9_7F4A_7C15)
                        .wrapping_add(level as u64 * 17),
                );
                run.current_level = level;
                run.rebuild_current_level();
                let profile = run.current_level_profile();

                for node in run
                    .nodes
                    .iter()
                    .filter(|node| matches!(node.kind, RoomKind::Combat))
                    .filter(|node| node.depth <= profile.safe_combat_depths)
                {
                    assert_eq!(
                        run.encounter_enemy_profile(node),
                        profile.briefing.combat_enemy
                    );
                }
            }
        }
    }

    #[test]
    fn later_combats_unlock_multiple_normal_enemy_profiles_per_level() {
        for level in 1..=3 {
            let mut seen = Vec::new();

            for seed in 0..64u64 {
                let mut run = DungeonRun::new(
                    seed.wrapping_mul(0xBF58_476D_1CE4_E5B9)
                        .wrapping_add(level as u64 * 29),
                );
                run.current_level = level;
                run.rebuild_current_level();
                let safe_depths = run.current_level_profile().safe_combat_depths;

                for node in run
                    .nodes
                    .iter()
                    .filter(|node| matches!(node.kind, RoomKind::Combat))
                    .filter(|node| node.depth > safe_depths)
                {
                    let enemy = run.encounter_enemy_profile(node);
                    if !seen.contains(&enemy) {
                        seen.push(enemy);
                    }
                }
            }

            assert!(
                seen.len() >= 2,
                "level {level} did not unlock multiple normal enemies"
            );
        }
    }

    #[test]
    fn later_normal_combats_spawn_dual_enemy_encounters() {
        for level in 1..=3 {
            let mut run = DungeonRun::new(TEST_ALT_RUN_SEED + level as u64);
            run.current_level = level;
            run.rebuild_current_level();
            let profile = run.current_level_profile();

            let mut saw_dual_encounter = false;
            for node in run
                .nodes
                .iter()
                .filter(|node| matches!(node.kind, RoomKind::Combat))
                .filter(|node| node.depth >= profile.force_elite_min_depth)
            {
                let setup = run.encounter_setup_for(node);
                if setup.enemies.len() == 2 {
                    saw_dual_encounter = true;
                    assert_ne!(setup.enemies[0].profile, setup.enemies[1].profile);
                    assert!(setup.enemies[0].hp >= setup.enemies[1].hp);
                }
            }

            assert!(
                saw_dual_encounter,
                "level {level} did not produce a dual-enemy normal combat"
            );
        }
    }

    #[test]
    fn level_profiles_scale_encounter_difficulty_across_acts() {
        let combat_node = DungeonNode {
            id: 1,
            depth: 4,
            lane: 2,
            kind: RoomKind::Combat,
            next: vec![],
        };
        let elite_node = DungeonNode {
            id: 2,
            depth: 5,
            lane: 2,
            kind: RoomKind::Elite,
            next: vec![],
        };
        let boss_node = DungeonNode {
            id: 3,
            depth: 9,
            lane: 3,
            kind: RoomKind::Boss,
            next: vec![],
        };

        let mut run = DungeonRun::new(TEST_RUN_SEED);
        run.current_level = 1;
        let level_one_combat = run.encounter_setup_for(&combat_node);
        let level_one_elite = run.encounter_setup_for(&elite_node);
        let level_one_boss = run.encounter_setup_for(&boss_node);

        run.current_level = 2;
        let level_two_combat = run.encounter_setup_for(&combat_node);
        let level_two_elite = run.encounter_setup_for(&elite_node);
        let level_two_boss = run.encounter_setup_for(&boss_node);

        run.current_level = 3;
        let level_three_combat = run.encounter_setup_for(&combat_node);
        let level_three_elite = run.encounter_setup_for(&elite_node);
        let level_three_boss = run.encounter_setup_for(&boss_node);

        assert!(level_one_combat.enemies[0].hp < level_two_combat.enemies[0].hp);
        assert!(level_two_combat.enemies[0].hp < level_three_combat.enemies[0].hp);
        assert!(level_one_elite.enemies[0].block < level_two_elite.enemies[0].block);
        assert!(level_two_elite.enemies[0].block < level_three_elite.enemies[0].block);
        assert_eq!(level_one_boss.enemies[0].hp, 72);
        assert_eq!(level_one_boss.enemies[0].block, 6);
        assert_eq!(level_two_boss.enemies[0].hp, 88);
        assert_eq!(level_two_boss.enemies[0].block, 10);
        assert_eq!(level_three_boss.enemies[0].hp, 108);
        assert_eq!(level_three_boss.enemies[0].block, 12);
    }

    #[test]
    fn boss_encounters_open_with_their_signature_patterns() {
        let boss_node = DungeonNode {
            id: 3,
            depth: 9,
            lane: 3,
            kind: RoomKind::Boss,
            next: vec![],
        };
        let mut run = DungeonRun::new(TEST_RUN_SEED);

        run.current_level = 1;
        let (penta_state, _) = CombatState::new_with_setup(
            TEST_BOSS_COMBAT_SEED_ONE,
            run.encounter_setup_for(&boss_node),
        );

        run.current_level = 2;
        let (hex_state, _) = CombatState::new_with_setup(
            TEST_BOSS_COMBAT_SEED_TWO,
            run.encounter_setup_for(&boss_node),
        );

        run.current_level = 3;
        let (hept_state, _) = CombatState::new_with_setup(
            TEST_BOSS_COMBAT_SEED_THREE,
            run.encounter_setup_for(&boss_node),
        );

        assert_eq!(penta_state.current_intent(0).unwrap().name, "Target Prism");
        assert_eq!(hex_state.current_intent(0).unwrap().name, "Hex Shell");
        assert_eq!(
            hept_state.current_intent(0).unwrap().name,
            "Singularity Shell"
        );
    }

    #[test]
    fn later_acts_shift_map_rhythm_toward_more_branches_and_fewer_rests() {
        let mut level_one_openers = 0usize;
        let mut level_three_openers = 0usize;
        let mut level_one_rests = 0usize;
        let mut level_three_rests = 0usize;
        let mut level_one_elites = 0usize;
        let mut level_three_elites = 0usize;

        for seed in 0..64u64 {
            let act_one = generate_nodes(level_seed(seed, 1), 1);
            let act_three = generate_nodes(level_seed(seed, 3), 3);

            level_one_openers += act_one.iter().filter(|node| node.depth == 1).count();
            level_three_openers += act_three.iter().filter(|node| node.depth == 1).count();
            level_one_rests += act_one
                .iter()
                .filter(|node| matches!(node.kind, RoomKind::Rest))
                .count();
            level_three_rests += act_three
                .iter()
                .filter(|node| matches!(node.kind, RoomKind::Rest))
                .count();
            level_one_elites += act_one
                .iter()
                .filter(|node| matches!(node.kind, RoomKind::Elite))
                .count();
            level_three_elites += act_three
                .iter()
                .filter(|node| matches!(node.kind, RoomKind::Elite))
                .count();
        }

        assert!(level_three_openers > level_one_openers);
        assert!(level_three_elites > level_one_elites);
        assert!(level_three_rests < level_one_rests);
    }

    #[test]
    fn generated_maps_include_exactly_one_shop_per_level() {
        for level in 1..=DUNGEON_LEVEL_COUNT {
            for seed in 0..128u64 {
                let nodes = generate_nodes(level_seed(seed, level), level);
                let shops = nodes
                    .iter()
                    .filter(|node| matches!(node.kind, RoomKind::Shop))
                    .count();

                assert_eq!(
                    shops, 1,
                    "level {level} seed {seed:#x} should have one shop"
                );
            }
        }
    }

    #[test]
    fn generated_maps_include_exactly_two_events_per_level() {
        for level in 1..=DUNGEON_LEVEL_COUNT {
            for seed in 0..128u64 {
                let nodes = generate_nodes(level_seed(seed, level), level);
                let events = nodes
                    .iter()
                    .filter(|node| matches!(node.kind, RoomKind::Event))
                    .count();

                assert_eq!(
                    events, 2,
                    "level {level} seed {seed:#x} should have two events"
                );
            }
        }
    }

    #[test]
    fn generated_events_stay_in_the_mid_run_combat_band() {
        for level in 1..=DUNGEON_LEVEL_COUNT {
            let profile = level_profile(level);
            for seed in 0..128u64 {
                let nodes = generate_nodes(level_seed(seed, level), level);
                let boss_depth = nodes.iter().map(|node| node.depth).max().unwrap_or(0);
                let pre_boss_depth = boss_depth.saturating_sub(1);
                let events: Vec<_> = nodes
                    .iter()
                    .filter(|node| matches!(node.kind, RoomKind::Event))
                    .collect();

                assert_eq!(
                    events.len(),
                    2,
                    "level {level} seed {seed:#x} should have two events"
                );
                for event in events {
                    assert!(
                        event.depth > profile.safe_combat_depths,
                        "level {level} seed {seed:#x} placed event too early at depth {}",
                        event.depth
                    );
                    assert!(
                        event.depth <= pre_boss_depth.saturating_sub(2),
                        "level {level} seed {seed:#x} placed event too late at depth {}",
                        event.depth
                    );
                }
            }
        }
    }

    #[test]
    fn generated_shops_stay_in_the_mid_run_combat_band() {
        for level in 1..=DUNGEON_LEVEL_COUNT {
            let profile = level_profile(level);
            for seed in 0..128u64 {
                let nodes = generate_nodes(level_seed(seed, level), level);
                let boss_depth = nodes.iter().map(|node| node.depth).max().unwrap_or(0);
                let pre_boss_depth = boss_depth.saturating_sub(1);
                let shop = nodes
                    .iter()
                    .find(|node| matches!(node.kind, RoomKind::Shop))
                    .expect("shop should exist");

                assert!(
                    shop.depth >= profile.force_elite_min_depth,
                    "level {level} seed {seed:#x} placed shop too early at depth {}",
                    shop.depth
                );
                assert!(
                    shop.depth <= pre_boss_depth.saturating_sub(2),
                    "level {level} seed {seed:#x} placed shop too late at depth {}",
                    shop.depth
                );
            }
        }
    }

    #[test]
    fn generated_maps_do_not_always_put_rest_before_boss() {
        let has_non_rest_pre_boss = (0..64u64).any(|seed| {
            let run = DungeonRun::new(seed.wrapping_mul(0x94D0_49BB_1331_11EB).wrapping_add(7));
            let pre_boss_depth = run.max_depth().saturating_sub(1);
            run.nodes
                .iter()
                .filter(|node| node.depth == pre_boss_depth)
                .any(|node| !matches!(node.kind, RoomKind::Rest))
        });

        assert!(has_non_rest_pre_boss);
    }

    #[test]
    fn generated_map_keeps_mid_run_width() {
        for seed in 0..128u64 {
            let run = DungeonRun::new(seed.wrapping_mul(0xD6E8_FD50_19E3_7C4B).wrapping_add(11));
            let narrow_mid_depths = (2..run.max_depth().saturating_sub(1))
                .filter(|&depth| run.nodes.iter().filter(|node| node.depth == depth).count() <= 1)
                .count();
            let max_narrow_depths = if run.available_nodes.len() <= 2 { 3 } else { 2 };

            assert!(
                narrow_mid_depths <= max_narrow_depths,
                "seed {seed:#x} collapsed into a narrow mid-map on {narrow_mid_depths} depths"
            );
        }
    }

    #[test]
    fn generated_map_avoids_long_isolated_stretches() {
        for seed in 0..128u64 {
            let run = DungeonRun::new(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(17));
            let mut isolated_streak = 0usize;
            let mut max_isolated_streak = 0usize;
            let max_allowed_streak = if run.available_nodes.len() <= 2 { 7 } else { 5 };

            for depth in 1..run.max_depth().saturating_sub(1) {
                if segment_is_isolated(&run, depth) {
                    isolated_streak += 1;
                    max_isolated_streak = max_isolated_streak.max(isolated_streak);
                } else {
                    isolated_streak = 0;
                }
            }

            assert!(
                max_isolated_streak <= max_allowed_streak,
                "seed {seed:#x} produced an isolated streak of {max_isolated_streak}"
            );
        }
    }

    #[test]
    fn generated_map_keeps_segment_density_reasonable() {
        for seed in 0..128u64 {
            let run = DungeonRun::new(seed.wrapping_mul(0xA24B_AED4_963E_E407).wrapping_add(29));

            for depth in 1..run.max_depth().saturating_sub(1) {
                let stats = segment_stats(&run, depth);
                assert!(
                    stats.max_incoming <= MAP_MAX_SEGMENT_MERGES + 1,
                    "seed {seed:#x} depth {depth} had {} incoming edges into one node",
                    stats.max_incoming
                );
                assert!(
                    stats.multi_connection_nodes <= 4,
                    "seed {seed:#x} depth {depth} had {} busy nodes in one segment",
                    stats.multi_connection_nodes
                );
            }
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct SegmentStats {
        max_incoming: usize,
        multi_connection_nodes: usize,
        multi_lane: bool,
        compact_span: bool,
    }

    fn segment_is_isolated(run: &DungeonRun, depth: usize) -> bool {
        let stats = segment_stats(run, depth);
        stats.multi_lane && stats.compact_span && stats.multi_connection_nodes == 0
    }

    fn segment_stats(run: &DungeonRun, depth: usize) -> SegmentStats {
        let from_nodes: Vec<&DungeonNode> = run
            .nodes
            .iter()
            .filter(|node| node.depth == depth)
            .collect();
        let to_nodes: Vec<&DungeonNode> = run
            .nodes
            .iter()
            .filter(|node| node.depth == depth + 1)
            .collect();
        let mut incoming: BTreeMap<usize, usize> = BTreeMap::new();
        let mut multi_outgoing = 0usize;
        let min_lane = from_nodes
            .iter()
            .chain(to_nodes.iter())
            .map(|node| node.lane)
            .min()
            .unwrap_or(0);
        let max_lane = from_nodes
            .iter()
            .chain(to_nodes.iter())
            .map(|node| node.lane)
            .max()
            .unwrap_or(0);

        for node in &from_nodes {
            let outgoing_to_next: Vec<usize> = node
                .next
                .iter()
                .copied()
                .filter(|next_id| run.nodes[*next_id].depth == depth + 1)
                .collect();
            if outgoing_to_next.len() > 1 {
                multi_outgoing += 1;
            }
            for next_id in outgoing_to_next {
                *incoming.entry(next_id).or_default() += 1;
            }
        }

        let max_incoming = incoming.values().copied().max().unwrap_or(0);
        let multi_incoming = incoming.values().filter(|&&count| count > 1).count();
        SegmentStats {
            max_incoming,
            multi_connection_nodes: multi_outgoing + multi_incoming,
            multi_lane: from_nodes.len() >= 2 && to_nodes.len() >= 2,
            compact_span: max_lane.saturating_sub(min_lane) <= 3,
        }
    }
}
