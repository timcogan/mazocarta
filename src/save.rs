use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::combat::{CombatOutcome, EncounterEnemySetup, EncounterSetup, TurnPhase};
use crate::content::{
    CardId, EnemyProfileId, EventId, ModuleId, RewardTier, card_slug, resolve_card_slug,
};
use crate::dungeon::RoomKind;

// Save v3 replaces the serialized combat status fields with
// Focus/Rhythm/Momentum, so older snapshots are intentionally rejected by the
// exact-version restore policy.
pub(crate) const SAVE_FORMAT_VERSION: u32 = 3;
const CURRENT_GAME_VERSION: &str = env!("CARGO_PKG_VERSION");
const DEFAULT_REPLACEMENT_CARD: CardId = CardId::FlareSlash;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RunSaveEnvelope {
    pub(crate) save_format_version: u32,
    pub(crate) game_version: String,
    pub(crate) active_state: SavedRunState,
    pub(crate) fallback_checkpoint: SavedCheckpoint,
    pub(crate) log: Vec<String>,
}

impl RunSaveEnvelope {
    pub(crate) fn new(
        active_state: SavedRunState,
        fallback_checkpoint: SavedCheckpoint,
        log: Vec<String>,
    ) -> Self {
        Self {
            save_format_version: SAVE_FORMAT_VERSION,
            game_version: CURRENT_GAME_VERSION.to_string(),
            active_state,
            fallback_checkpoint,
            log,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "screen", rename_all = "snake_case")]
pub(crate) enum SavedRunState {
    Map {
        dungeon: SavedDungeonRun,
    },
    ModuleSelect {
        dungeon: SavedDungeonRun,
        module_select: SavedModuleSelectState,
    },
    LevelIntro {
        dungeon: SavedDungeonRun,
    },
    Rest {
        dungeon: SavedDungeonRun,
    },
    Shop {
        dungeon: SavedDungeonRun,
        shop: SavedShopState,
    },
    Event {
        dungeon: SavedDungeonRun,
        event: SavedEventState,
    },
    Reward {
        dungeon: SavedDungeonRun,
        reward: SavedRewardState,
    },
    Combat {
        dungeon: SavedDungeonRun,
        combat: SavedCombatState,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum SavedCheckpoint {
    Map {
        dungeon: SavedDungeonRun,
    },
    ModuleSelect {
        dungeon: SavedDungeonRun,
        module_select: SavedModuleSelectState,
    },
    LevelIntro {
        dungeon: SavedDungeonRun,
    },
    Rest {
        dungeon: SavedDungeonRun,
    },
    Shop {
        dungeon: SavedDungeonRun,
        shop: SavedShopState,
    },
    Event {
        dungeon: SavedDungeonRun,
        event: SavedEventState,
    },
    Reward {
        dungeon: SavedDungeonRun,
        reward: SavedRewardState,
    },
    EncounterStart {
        dungeon: SavedDungeonRun,
        encounter_setup: SavedEncounterSetup,
        source_deck: Vec<String>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SavedDungeonRun {
    pub(crate) seed: u64,
    pub(crate) current_level: usize,
    pub(crate) nodes: Vec<SavedDungeonNode>,
    pub(crate) current_node: Option<usize>,
    pub(crate) available_nodes: Vec<usize>,
    pub(crate) visited_nodes: Vec<usize>,
    pub(crate) deck: Vec<String>,
    // MIGRATION(save v3): Supports saves where the legacy `modules` field is
    // missing. Missing `modules` restores as the default starter module.
    // Remove when minimum supported save format > 3.
    #[serde(default)]
    pub(crate) modules: Option<Vec<String>>,
    pub(crate) player_hp: i32,
    pub(crate) player_max_hp: i32,
    // MIGRATION(save v3): Supports saves where the legacy `credits` field is
    // missing. Missing `credits` restores as 0. Remove when minimum supported
    // save format > 3.
    #[serde(default)]
    pub(crate) credits: u32,
    pub(crate) combats_cleared: usize,
    pub(crate) elites_cleared: usize,
    pub(crate) rests_completed: usize,
    pub(crate) bosses_cleared: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SavedDungeonNode {
    pub(crate) id: usize,
    pub(crate) depth: usize,
    pub(crate) lane: usize,
    pub(crate) kind: String,
    pub(crate) next: Vec<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SavedRewardState {
    pub(crate) tier: String,
    pub(crate) options: Vec<String>,
    pub(crate) followup_completed_run: bool,
    pub(crate) seed: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SavedModuleSelectState {
    pub(crate) options: Vec<String>,
    pub(crate) seed: u64,
    // MIGRATION(save v3): Supports saves where the legacy `kind` field is
    // missing. Missing `kind` restores as the starter module-select context.
    // Remove when minimum supported save format > 3.
    #[serde(default)]
    pub(crate) kind: Option<String>,
    // MIGRATION(save v3): Supports saves where the legacy `boss_level` field is
    // missing. Missing `boss_level` deserializes as `None`; fallback restore
    // uses boss level 1, while exact restore still errors if a boss reward
    // requires it. Remove when minimum supported save format > 3.
    #[serde(default)]
    pub(crate) boss_level: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SavedShopState {
    pub(crate) offers: Vec<SavedShopOffer>,
    pub(crate) seed: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SavedShopOffer {
    pub(crate) card: String,
    pub(crate) price: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SavedEventState {
    pub(crate) event: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SavedCombatState {
    pub(crate) player: SavedPlayerState,
    pub(crate) enemies: Vec<SavedEnemyState>,
    pub(crate) deck: SavedDeckState,
    pub(crate) phase: String,
    pub(crate) turn: u32,
    pub(crate) rng_state: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SavedPlayerState {
    pub(crate) fighter: SavedFighterState,
    pub(crate) energy: u8,
    pub(crate) max_energy: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SavedEnemyState {
    pub(crate) fighter: SavedFighterState,
    pub(crate) profile: String,
    pub(crate) intent_index: usize,
    pub(crate) on_hit_bleed: u8,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SavedFighterState {
    pub(crate) hp: i32,
    pub(crate) max_hp: i32,
    pub(crate) block: i32,
    pub(crate) bleed: u8,
    pub(crate) focus: i8,
    pub(crate) rhythm: i8,
    pub(crate) momentum: i8,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SavedDeckState {
    pub(crate) draw_pile: Vec<String>,
    pub(crate) hand: Vec<String>,
    pub(crate) discard_pile: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SavedEncounterSetup {
    pub(crate) player_hp: i32,
    pub(crate) player_max_hp: i32,
    pub(crate) player_max_energy: u8,
    pub(crate) enemies: Vec<SavedEncounterEnemyState>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SavedEncounterEnemyState {
    pub(crate) hp: i32,
    pub(crate) max_hp: i32,
    pub(crate) block: i32,
    pub(crate) profile: String,
    pub(crate) intent_index: usize,
    pub(crate) on_hit_bleed: u8,
}

pub(crate) fn serialize_envelope(envelope: &RunSaveEnvelope) -> Result<String, String> {
    serde_json::to_string(envelope).map_err(|error| error.to_string())
}

pub(crate) fn parse_run_save(raw: &str) -> Result<RunSaveEnvelope, String> {
    let value: Value = serde_json::from_str(raw).map_err(|error| error.to_string())?;
    let version = value
        .get("save_format_version")
        .and_then(Value::as_u64)
        .ok_or_else(|| "Missing save_format_version.".to_string())? as u32;

    if version != SAVE_FORMAT_VERSION {
        return Err(format!("Unsupported save format version {version}."));
    }

    serde_json::from_value(value).map_err(|error| error.to_string())
}

pub(crate) fn serialize_card_id(id: CardId) -> &'static str {
    card_slug(id)
}

pub(crate) fn serialize_module_id(id: ModuleId) -> &'static str {
    match id {
        ModuleId::AegisDrive => "aegis_drive",
        ModuleId::TargetingRelay => "targeting_relay",
        ModuleId::Nanoforge => "nanoforge",
        ModuleId::CapacitorBank => "capacitor_bank",
        ModuleId::PrismScope => "prism_scope",
        ModuleId::SalvageLedger => "salvage_ledger",
        ModuleId::OverclockCore => "overclock_core",
        ModuleId::SuppressionField => "suppression_field",
        ModuleId::RecoveryMatrix => "recovery_matrix",
    }
}

pub(crate) fn resolve_card_id(id: &str) -> Option<CardId> {
    resolve_card_slug(id)
}

pub(crate) fn resolve_module_id(id: &str) -> Option<ModuleId> {
    match id {
        "aegis_drive" => Some(ModuleId::AegisDrive),
        "targeting_relay" => Some(ModuleId::TargetingRelay),
        "nanoforge" => Some(ModuleId::Nanoforge),
        "capacitor_bank" => Some(ModuleId::CapacitorBank),
        "prism_scope" => Some(ModuleId::PrismScope),
        "salvage_ledger" => Some(ModuleId::SalvageLedger),
        "overclock_core" => Some(ModuleId::OverclockCore),
        "suppression_field" => Some(ModuleId::SuppressionField),
        "recovery_matrix" => Some(ModuleId::RecoveryMatrix),
        _ => None,
    }
}

pub(crate) fn resolve_deck_card_id(id: &str) -> CardId {
    resolve_card_id(id).unwrap_or(DEFAULT_REPLACEMENT_CARD)
}

pub(crate) fn serialize_enemy_profile(profile: EnemyProfileId) -> &'static str {
    match profile {
        EnemyProfileId::ScoutDrone => "scout_drone",
        EnemyProfileId::NeedlerDrone => "needler_drone",
        EnemyProfileId::RampartDrone => "rampart_drone",
        EnemyProfileId::SpineSentry => "spine_sentry",
        EnemyProfileId::PentaCore => "penta_core",
        EnemyProfileId::VoltMantis => "volt_mantis",
        EnemyProfileId::ShardWeaver => "shard_weaver",
        EnemyProfileId::PrismArray => "prism_array",
        EnemyProfileId::GlassBishop => "glass_bishop",
        EnemyProfileId::HexarchCore => "hexarch_core",
        EnemyProfileId::NullRaider => "null_raider",
        EnemyProfileId::RiftStalker => "rift_stalker",
        EnemyProfileId::SiegeSpider => "siege_spider",
        EnemyProfileId::RiftBastion => "rift_bastion",
        EnemyProfileId::HeptarchCore => "heptarch_core",
    }
}

pub(crate) fn resolve_enemy_profile(id: &str) -> Option<EnemyProfileId> {
    match id {
        "scout_drone" => Some(EnemyProfileId::ScoutDrone),
        "needler_drone" => Some(EnemyProfileId::NeedlerDrone),
        "rampart_drone" => Some(EnemyProfileId::RampartDrone),
        "spine_sentry" => Some(EnemyProfileId::SpineSentry),
        "penta_core" => Some(EnemyProfileId::PentaCore),
        "volt_mantis" => Some(EnemyProfileId::VoltMantis),
        "shard_weaver" => Some(EnemyProfileId::ShardWeaver),
        "prism_array" => Some(EnemyProfileId::PrismArray),
        "glass_bishop" => Some(EnemyProfileId::GlassBishop),
        "hexarch_core" => Some(EnemyProfileId::HexarchCore),
        "null_raider" => Some(EnemyProfileId::NullRaider),
        "rift_stalker" => Some(EnemyProfileId::RiftStalker),
        "siege_spider" => Some(EnemyProfileId::SiegeSpider),
        "rift_bastion" => Some(EnemyProfileId::RiftBastion),
        "heptarch_core" => Some(EnemyProfileId::HeptarchCore),
        _ => None,
    }
}

pub(crate) fn serialize_reward_tier(tier: RewardTier) -> &'static str {
    match tier {
        RewardTier::Combat => "combat",
        RewardTier::Elite => "elite",
        RewardTier::Boss => "boss",
    }
}

pub(crate) fn serialize_event_id(id: EventId) -> &'static str {
    match id {
        EventId::SalvageCache => "salvage_cache",
        EventId::RelayTerminal => "relay_terminal",
        EventId::ClinicPod => "clinic_pod",
        EventId::ExchangeConsole => "exchange_console",
        EventId::PrototypeRack => "prototype_rack",
        EventId::CoolingVault => "cooling_vault",
    }
}

pub(crate) fn resolve_event_id(id: &str) -> Option<EventId> {
    match id {
        "salvage_cache" => Some(EventId::SalvageCache),
        "relay_terminal" => Some(EventId::RelayTerminal),
        "clinic_pod" => Some(EventId::ClinicPod),
        "exchange_console" => Some(EventId::ExchangeConsole),
        "prototype_rack" => Some(EventId::PrototypeRack),
        "cooling_vault" => Some(EventId::CoolingVault),
        _ => None,
    }
}

pub(crate) fn resolve_reward_tier(id: &str) -> Option<RewardTier> {
    match id {
        "combat" => Some(RewardTier::Combat),
        "elite" => Some(RewardTier::Elite),
        "boss" => Some(RewardTier::Boss),
        _ => None,
    }
}

pub(crate) fn serialize_room_kind(kind: RoomKind) -> &'static str {
    match kind {
        RoomKind::Start => "start",
        RoomKind::Combat => "combat",
        RoomKind::Elite => "elite",
        RoomKind::Rest => "rest",
        RoomKind::Shop => "shop",
        RoomKind::Event => "event",
        RoomKind::Boss => "boss",
    }
}

pub(crate) fn resolve_room_kind(id: &str) -> Option<RoomKind> {
    match id {
        "start" => Some(RoomKind::Start),
        "combat" => Some(RoomKind::Combat),
        "elite" => Some(RoomKind::Elite),
        "rest" => Some(RoomKind::Rest),
        "shop" => Some(RoomKind::Shop),
        "event" => Some(RoomKind::Event),
        "boss" => Some(RoomKind::Boss),
        _ => None,
    }
}

pub(crate) fn serialize_turn_phase(phase: TurnPhase) -> &'static str {
    match phase {
        TurnPhase::PlayerTurn => "player_turn",
        TurnPhase::EnemyTurn => "enemy_turn",
        TurnPhase::Ended(CombatOutcome::Victory) => "ended_victory",
        TurnPhase::Ended(CombatOutcome::Defeat) => "ended_defeat",
    }
}

pub(crate) fn resolve_turn_phase(id: &str) -> Option<TurnPhase> {
    match id {
        "player_turn" => Some(TurnPhase::PlayerTurn),
        "enemy_turn" => Some(TurnPhase::EnemyTurn),
        "ended_victory" => Some(TurnPhase::Ended(CombatOutcome::Victory)),
        "ended_defeat" => Some(TurnPhase::Ended(CombatOutcome::Defeat)),
        _ => None,
    }
}

pub(crate) fn save_encounter_setup(setup: EncounterSetup) -> SavedEncounterSetup {
    SavedEncounterSetup {
        player_hp: setup.player_hp,
        player_max_hp: setup.player_max_hp,
        player_max_energy: setup.player_max_energy,
        enemies: setup
            .enemies
            .into_iter()
            .map(|enemy| SavedEncounterEnemyState {
                hp: enemy.hp,
                max_hp: enemy.max_hp,
                block: enemy.block,
                profile: serialize_enemy_profile(enemy.profile).to_string(),
                intent_index: enemy.intent_index,
                on_hit_bleed: enemy.on_hit_bleed,
            })
            .collect(),
    }
}

pub(crate) fn resolve_encounter_setup(setup: &SavedEncounterSetup) -> Option<EncounterSetup> {
    Some(EncounterSetup {
        player_hp: setup.player_hp,
        player_max_hp: setup.player_max_hp,
        player_max_energy: setup.player_max_energy,
        enemies: setup
            .enemies
            .iter()
            .map(|enemy| {
                Some(EncounterEnemySetup {
                    hp: enemy.hp,
                    max_hp: enemy.max_hp,
                    block: enemy.block,
                    profile: resolve_enemy_profile(&enemy.profile)?,
                    intent_index: enemy.intent_index,
                    on_hit_bleed: enemy.on_hit_bleed,
                })
            })
            .collect::<Option<Vec<_>>>()?,
    })
}
