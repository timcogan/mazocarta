use std::collections::VecDeque;
use std::fmt::Write;

use crate::combat::{
    Actor, CombatAction, CombatEvent, CombatOutcome, CombatState, DeckState, EncounterSetup,
    EnemyState, FighterState, PlayerState, StatusKind, StatusSet, preview_scaled_value,
};
use crate::content::{
    AxisKind, CardArchetype, CardDef, CardId, EnemyIntent, EnemyProfileId, EnemySpriteLayerTone,
    EventId, Language, ModuleDef, ModuleId, RewardTier, ShopOffer, all_base_cards,
    boss_module_choices, card_def, default_starter_module, enemy_profile_level, enemy_sprite_def,
    localized_card_def, localized_card_name, localized_enemy_intent, localized_enemy_name,
    localized_event_choice_body, localized_event_choice_title, localized_event_def,
    localized_module_def, localized_text, reward_choices, shop_offers, starter_module_choices,
    upgraded_card,
};
use crate::dungeon::{
    DungeonProgress, DungeonRun, EventResolution, NodeSelection, RoomKind, credits_reward_for_room,
    localized_level_codename, localized_level_summary,
};
use crate::run_logic::{
    apply_post_victory_modules as apply_post_victory_module_effects,
    combat_seed_for_dungeon as shared_combat_seed_for_dungeon,
};
use crate::save::{
    RunSaveEnvelope, SavedCheckpoint, SavedCombatState, SavedDeckState, SavedDungeonNode,
    SavedDungeonRun, SavedEnemyState, SavedEventState, SavedFighterState, SavedModuleSelectState,
    SavedPlayerState, SavedRewardState, SavedRunState, SavedShopOffer, SavedShopState,
    parse_run_save, resolve_card_id, resolve_deck_card_id, resolve_encounter_setup,
    resolve_enemy_profile, resolve_event_id, resolve_module_id, resolve_reward_tier,
    resolve_room_kind, resolve_turn_phase, save_encounter_setup, serialize_card_id,
    serialize_enemy_profile, serialize_envelope, serialize_event_id, serialize_module_id,
    serialize_reward_tier, serialize_room_kind, serialize_turn_phase,
};

const LOGICAL_WIDTH: f32 = 1280.0;
const LOGICAL_HEIGHT: f32 = 720.0;
const BASE_SEED: u64 = 0xA57A_C47A_2204_0001;
const RUN_SEED_MASK: u64 = 0xFFFF_FFFF;
const GAME_TITLE: &str = "Mazocarta";
const GAME_VERSION: &str = env!("CARGO_PKG_VERSION");
const BUILD_APP_CHANNEL: Option<&str> = option_env!("MAZOCARTA_APP_CHANNEL");
const APP_BUILD_TIMESTAMP_UTC: Option<&str> = option_env!("MAZOCARTA_APP_BUILD_TIMESTAMP_UTC");
const APP_GIT_SHA_SHORT: Option<&str> = option_env!("MAZOCARTA_APP_GIT_SHA_SHORT");
const PLAYER_NAME: &str = "Player";
const GUARD_LABEL: &str = "Shield";
const COMBAT_HEART_ICON_ASSET_PATH: &str = "./icons/combat/heart.svg";
const COMBAT_SHIELD_ICON_ASSET_PATH: &str = "./icons/combat/shield.svg";
const COMBAT_ENERGY_ICON_ASSET_PATH: &str = "./icons/combat/energy.svg";
const COMBAT_DECK_ICON_ASSET_PATH: &str = "./icons/combat/deck.svg";
const COMBAT_INLINE_ICON_HEIGHT_RATIO: f32 = 1.0;
const COMBAT_INLINE_ICON_ASPECT_RATIO: f32 = 0.75;
const COMBAT_INLINE_ICON_TEXT_GAP_RATIO: f32 = 0.24;
const COMBAT_ACTION_UI_SCALE: f32 = 0.75;
const PANEL_TEXT_GAP: &str = " ";
const NEXT_SIGNAL_LABEL: &str = "Next";
const TERM_GREEN: &str = "#33ff66";
const TERM_GREEN_SOFT: &str = "#8dffad";
const TERM_GREEN_TEXT: &str = "#c9ffd7";
const TERM_GREEN_DIM: &str = "#6f9f7b";
const TERM_CYAN: &str = "#3df5ff";
const TERM_CYAN_SOFT: &str = "#a8fcff";
const TERM_BLUE_SOFT: &str = "#9bb7ff";
const TERM_PINK: &str = "#ff4fd8";
const TERM_PINK_SOFT: &str = "#ff9cf0";
const TERM_ORANGE: &str = "#ffb852";
const TERM_LIME_SOFT: &str = "#ebff9a";
const COLOR_TILE_FILL: &str = "rgba(0, 0, 0, 1.0)";
const COLOR_GREEN_STROKE_STRONG: &str = "rgba(51, 255, 102, 0.92)";
const COLOR_GREEN_STROKE_START: &str = "rgba(51, 255, 102, 0.85)";
const COLOR_GREEN_STROKE_IDLE: &str = "rgba(51, 255, 102, 0.55)";
const COLOR_GREEN_STROKE_PANEL: &str = "rgba(51, 255, 102, 0.38)";
const COLOR_GREEN_STROKE_CARD: &str = "rgba(51, 255, 102, 0.28)";
const COLOR_CYAN_STROKE_DISABLED: &str = "rgba(61, 245, 255, 0.22)";
const COLOR_CYAN_STROKE_TARGET: &str = "rgba(61, 245, 255, 0.72)";
const COLOR_CYAN_STROKE_STRONG: &str = "rgba(61, 245, 255, 0.92)";
const COLOR_CYAN_STROKE_IDLE: &str = "rgba(61, 245, 255, 0.60)";
const COLOR_BLUE_STROKE_IDLE: &str = "rgba(155, 183, 255, 0.60)";
const COLOR_BLUE_STROKE_STRONG: &str = "rgba(155, 183, 255, 0.92)";
const COLOR_PINK_STROKE_STRONG: &str = "rgba(255, 79, 216, 0.92)";
const COLOR_LIME_STROKE_TARGET: &str = "rgba(216, 255, 61, 0.72)";
const COLOR_GRAY_STROKE_SELECTED: &str = "rgba(196, 196, 196, 0.92)";
const COLOR_GRAY_STROKE_HOVER: &str = "rgba(166, 166, 166, 0.72)";
const COLOR_GRAY_STROKE_DISABLED: &str = "rgba(136, 136, 136, 0.45)";
const COLOR_GRAY_STROKE_HINT: &str = "rgba(136, 136, 136, 0.72)";
const COLOR_WHITE_STROKE_PATH: &str = "rgba(255, 255, 255, 0.78)";
const BUTTON_RADIUS: f32 = 8.0;
const CARD_RADIUS: f32 = 8.0;
const ENEMY_PANEL_RADIUS: f32 = 8.0;
const UI_TILE_FILL_ALPHA: f32 = 0.02;
const UI_TILE_STROKE_WIDTH: f32 = 0.5;
const UI_TILE_STROKE_ALPHA_BOOST: f32 = 0.18;
const UI_TILE_STROKE_ALPHA_SCALE: f32 = 1.0;
const ENEMY_PANEL_ICON_ALPHA: f32 = 0.92;
const ENEMY_PANEL_ICON_DISABLED_ALPHA: f32 = 0.78;
const CARD_WIDTH: f32 = 190.0;
const CARD_HEIGHT: f32 = 160.0;
const HAND_MIN_GAP: f32 = 10.0;
const HAND_TWO_ROW_SCALE: f32 = 0.82;
const LOW_HAND_CARD_SCALE: f32 = 1.16;
const LOW_HAND_MAX_COUNT: usize = 3;
const MAX_COMBAT_HAND_ROWS: usize = 3;
const MAX_COMBAT_ENEMY_ROWS: usize = 2;
const MIN_COMBAT_TILE_SCALE: f32 = 0.35;
const COMBAT_LAYOUT_SCORE_EPSILON: f32 = 0.01;
const TOP_BUTTON_FONT_SIZE: f32 = 20.0;
const LOW_HAND_TOP_BUTTON_FONT_SIZE: f32 = 22.0;
const START_BUTTON_FONT_SIZE: f32 = 28.0;
const OVERLAY_BUTTON_MIN_FONT_SIZE: f32 = 18.0;
const OVERLAY_BUTTON_MIN_PAD_X: f32 = 8.0;
const OVERLAY_BUTTON_MIN_PAD_Y: f32 = 6.0;
const OVERLAY_BUTTON_ROW_GAP: f32 = 16.0;
const OVERLAY_BUTTON_MIN_ROW_GAP: f32 = 8.0;
const OVERLAY_BUTTON_STACK_GAP: f32 = 12.0;
const RESET_BUTTON_PAD_X: f32 = 10.0;
const RESET_BUTTON_PAD_Y: f32 = 12.0;
const RESULT_BUTTON_LABEL: &str = "Main Menu";
const LEGEND_BUTTON_LABEL: &str = "Legend";
const LOGO_ASSET_PATH: &str = "./mazocarta.svg";
const LAYOUT_TRANSITION_MS: f32 = 140.0;
const SCREEN_TRANSITION_MS: f32 = 220.0;
const RESULT_SCREEN_TRANSITION_MS: f32 = 750.0;
const OPENING_INTRO_LINE_FADE_MS: f32 = 680.0;
const OPENING_INTRO_LINE_PAUSE_MS: f32 = 520.0;
const OPENING_INTRO_BUTTON_TRANSITION_MS: f32 = 180.0;
const LEGEND_TRANSITION_MS: f32 = 160.0;
const BOOT_MODAL_TRANSITION_MS: f32 = 160.0;
const LEGEND_BACKDROP_BLUR: f32 = 7.0;
const BOOT_RESTART_MODAL_BLUR: f32 = 8.0;
const COMBAT_TURN_BANNER_MS: f32 = 900.0;
const COMBAT_OUTCOME_VFX_HOLD_MS: f32 = 320.0;
const COMBAT_PLAYBACK_STEP_MS: f32 = 78.0;
const COMBAT_PLAYBACK_PAUSE_DAMAGE_MS: f32 = 96.0;
const COMBAT_PLAYBACK_PAUSE_BLOCK_MS: f32 = 82.0;
const COMBAT_PLAYBACK_PAUSE_STATUS_MS: f32 = 70.0;
const COMBAT_PLAYBACK_PAUSE_LOG_MS: f32 = 56.0;
const COMBAT_STAT_COUNTDOWN_MAX_STEPS: usize = 8;
const MAP_NODE_RADIUS: f32 = 18.0;
const MAP_NODE_DIAMETER: f32 = MAP_NODE_RADIUS * 2.0;
const MAP_LINE_WIDTH: f32 = 3.0;
// Keep adjacent map lanes within one node width of empty space on wide viewports.
const MAP_MAX_ADJACENT_LANE_CENTER_SPACING: f32 = MAP_NODE_DIAMETER * 2.0;
const BOOT_HERO_SHIFT_UP: f32 = 54.0;
const MAP_DEBUG_SEED_SIZE: f32 = 14.0;
const MAP_DEBUG_BUTTON_FONT_SIZE: f32 = 15.0;
const MAP_DEBUG_BUTTON_PAD_X: f32 = 10.0;
const MAP_DEBUG_BUTTON_PAD_Y: f32 = 6.0;
const MAP_DEBUG_BUTTON_GAP: f32 = 6.0;
const BOOT_CONTINUE_LABEL: &str = "Continue";
const BOOT_RESTART_LABEL: &str = "Restart";
const BOOT_SETTINGS_LABEL: &str = "Settings";
const BOOT_INSTALL_LABEL: &str = "Install";
const BOOT_UPDATE_LABEL: &str = "Update";
const BOOT_DEBUG_CLEAR_LABEL: &str = "Reset";
const BOOT_RESTART_CONFIRM_TITLE: &str = "Are you sure you want to restart?";
const BOOT_RESTART_CONFIRM_CANCEL_LABEL: &str = "Cancel";
const REWARD_SKIP_LABEL: &str = "Skip";
const SHOP_LEAVE_LABEL: &str = "Leave";
const MAP_INFO_LABEL: &str = "Info";
const MAP_DEBUG_FILL_DECK_LABEL: &str = "Fill Deck";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum AppScreen {
    Boot,
    OpeningIntro,
    Map,
    ModuleSelect,
    LevelIntro,
    Rest,
    Shop,
    Event,
    Combat,
    Reward,
    Result(CombatOutcome),
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Viewport {
    width: f32,
    height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Rect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl Rect {
    fn contains(self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.w && y >= self.y && y <= self.y + self.h
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HitTarget {
    Start,
    Continue,
    Settings,
    Install,
    Update,
    SettingsModal,
    SettingsLanguageEnglish,
    SettingsLanguageSpanish,
    InstallHelpModal,
    InstallHelpClose,
    DebugLevelDown,
    DebugLevelUp,
    DebugFillDeck,
    Share,
    Restart,
    Menu,
    Legend,
    LegendPanel,
    Info,
    RunInfoPanel,
    EnemyInspectPanel,
    RestHeal,
    RestCard(usize),
    RestConfirm,
    RestPagePrev,
    RestPageNext,
    ShopCard(usize),
    ShopLeave,
    EventChoice(usize),
    RewardCard(usize),
    RewardSkip,
    ModuleSelectCard(usize),
    EndTurn,
    EndBattle,
    Enemy(usize),
    Player,
    MapNode(usize),
    Card(usize),
    RestartModal,
    RestartConfirm,
    RestartCancel,
    DebugClearSave,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RestSelection {
    Heal,
    Upgrade(usize),
}

#[derive(Clone, Copy, Debug, Default)]
struct UiState {
    selected_card: Option<usize>,
    rest_selection: Option<RestSelection>,
    rest_page: usize,
    hover: Option<HitTarget>,
    legend_open: bool,
    legend_progress: f32,
    run_info_open: bool,
    run_info_progress: f32,
    enemy_inspect_enemy: Option<usize>,
    enemy_inspect_open: bool,
    enemy_inspect_progress: f32,
    restart_confirm_open: bool,
    restart_confirm_progress: f32,
    settings_open: bool,
    settings_progress: f32,
    install_help_open: bool,
    install_help_progress: f32,
}

#[derive(Clone, Debug)]
struct Floater {
    text: String,
    x: f32,
    y: f32,
    ttl_ms: f32,
    total_ms: f32,
    color: (u8, u8, u8),
}

#[derive(Clone, Debug)]
struct PixelShard {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    size: f32,
    ttl_ms: f32,
    total_ms: f32,
    color: (u8, u8, u8),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ActorDisplayedStats {
    hp: i32,
    block: i32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct PlayerDisplayedMeta {
    energy: i32,
    draw_pile: i32,
    discard_pile: i32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct DisplayedCombatStats {
    player: ActorDisplayedStats,
    player_meta: PlayerDisplayedMeta,
    enemies: Vec<ActorDisplayedStats>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CombatStat {
    Hp,
    Block,
    Energy,
    DrawPile,
    DiscardPile,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StatTint {
    Damage,
    BlockGain,
    NeutralLoss,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CombatPlaybackKind {
    EnemyTurn,
    PlayerAction,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum StatChangeOp {
    Add(i32),
    Subtract(i32),
    Set(i32),
    Values(Vec<i32>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct QueuedStatChange {
    actor: Actor,
    stat: CombatStat,
    op: StatChangeOp,
    tint: StatTint,
}

#[derive(Clone, Debug)]
struct StatCountdown {
    actor: Actor,
    stat: CombatStat,
    values: Vec<i32>,
    target: i32,
    ttl_ms: f32,
    total_ms: f32,
    tint: StatTint,
}

#[derive(Clone, Debug)]
struct TurnBanner {
    text: String,
    color: (u8, u8, u8),
    ttl_ms: f32,
    total_ms: f32,
}

#[derive(Clone, Debug, Default)]
struct CombatFeedbackState {
    displayed: DisplayedCombatStats,
    displayed_intents: Vec<EnemyIntent>,
    playback_kind: Option<CombatPlaybackKind>,
    stat_queue: VecDeque<Vec<QueuedStatChange>>,
    active_stats: Vec<StatCountdown>,
    playback_queue: VecDeque<CombatEvent>,
    playback_pause_ms: f32,
    outcome_hold_ms: f32,
    turn_banner: Option<TurnBanner>,
    auto_playback_active: bool,
    pending_outcome: Option<CombatOutcome>,
}

#[derive(Clone, Debug)]
struct Layout {
    start_button: Rect,
    restart_button: Rect,
    clear_save_button: Option<Rect>,
    menu_button: Rect,
    end_turn_button: Rect,
    end_battle_button: Option<Rect>,
    enemy_indices: Vec<usize>,
    #[cfg_attr(not(test), allow(dead_code))]
    enemy_arrangement: CombatGridArrangement,
    enemy_rects: Vec<Rect>,
    player_rect: Rect,
    #[cfg_attr(not(test), allow(dead_code))]
    hand_arrangement: CombatGridArrangement,
    hand_rects: Vec<Rect>,
    hint_rect: Option<Rect>,
    low_hand_layout: bool,
    tile_scale: f32,
    tile_insets: TileInsets,
}

#[derive(Clone, Debug)]
struct MapLayout {
    menu_button: Rect,
    info_button: Rect,
    legend_button: Rect,
    legend_modal: Rect,
    debug_level_down_button: Option<Rect>,
    debug_level_up_button: Option<Rect>,
    debug_level_text_position: Option<(f32, f32)>,
    debug_fill_deck_button: Option<Rect>,
    nodes: Vec<MapNodeLayout>,
    edges: Vec<MapEdgeLayout>,
}

#[derive(Clone, Debug)]
struct RewardLayout {
    card_rects: Vec<Rect>,
    credits_y: f32,
    skip_button: Rect,
}

#[derive(Clone, Debug)]
struct ShopLayout {
    card_rects: Vec<Rect>,
    price_ys: Vec<f32>,
    credits_y: f32,
    leave_button: Rect,
}

#[derive(Clone, Debug)]
struct EventLayout {
    title_y: f32,
    choice_rects: Vec<Rect>,
}

#[derive(Clone, Debug)]
struct ModuleSelectLayout {
    title_y: f32,
    card_rects: Vec<Rect>,
}

impl Layout {
    fn enemy_rect(&self, enemy_index: usize) -> Option<Rect> {
        self.enemy_indices
            .iter()
            .position(|index| *index == enemy_index)
            .and_then(|slot| self.enemy_rects.get(slot).copied())
    }
}

#[derive(Clone, Copy, Debug)]
struct RunInfoLayout {
    modal_rect: Rect,
}

#[derive(Clone, Copy, Debug)]
struct EnemyInspectLayout {
    modal_rect: Rect,
    sprite_rect: Rect,
    title_size: f32,
    title_y: f32,
}

#[derive(Clone, Copy, Debug)]
struct ResultButtons {
    share_button: Option<Rect>,
    menu_button: Rect,
}

#[derive(Clone, Debug)]
struct RestLayout {
    heal_rect: Rect,
    card_rects: Vec<Rect>,
    visible_upgrade_indices: Vec<usize>,
    prev_button: Option<FittedPrimaryButton>,
    next_button: Option<FittedPrimaryButton>,
    page_status_label: Option<String>,
    page_status_x: Option<f32>,
    page_status_y: Option<f32>,
    page_status_size: Option<f32>,
    current_page: usize,
    page_count: usize,
    confirm_rect: Rect,
}

#[derive(Clone, Copy, Debug)]
struct MapNodeLayout {
    id: usize,
    rect: Rect,
    center_x: f32,
    center_y: f32,
}

#[derive(Clone, Copy, Debug)]
struct MapEdgeLayout {
    from_id: usize,
    to_id: usize,
    from_x: f32,
    from_y: f32,
    to_x: f32,
    to_y: f32,
}

#[derive(Clone, Debug)]
struct LayoutTransition {
    from_layout: Layout,
    to_layout: Layout,
    hand_from_rects: Vec<Option<Rect>>,
    ttl_ms: f32,
    total_ms: f32,
}

#[derive(Clone, Debug)]
struct ScreenTransition {
    from_screen: AppScreen,
    to_screen: AppScreen,
    style: ScreenTransitionStyle,
    from_boot_has_saved_run: bool,
    to_boot_has_saved_run: bool,
    ttl_ms: f32,
    total_ms: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ScreenTransitionStyle {
    Motion,
    Fade,
}

#[derive(Clone, Copy, Debug)]
struct BootHeroLayout {
    logo_rect: Rect,
    title_size: f32,
    title_baseline_y: f32,
    start_button_y: f32,
}

#[derive(Clone, Copy, Debug)]
struct BootButtonsLayout {
    start_button: Rect,
    restart_button: Rect,
    settings_button: Rect,
    install_button: Option<Rect>,
    update_button: Option<Rect>,
    clear_save_button: Option<Rect>,
}

#[derive(Clone, Copy, Debug)]
struct EventChoiceTileStyle<'a> {
    stroke: &'a str,
    title_color: &'a str,
    body_color: &'a str,
}

#[derive(Clone, Copy, Debug)]
struct MapNodeSymbolLayout {
    center_x: f32,
    center_y: f32,
    radius: f32,
}

#[derive(Clone, Copy, Debug)]
struct TextLineLayout {
    x: f32,
    y: f32,
    size: f32,
}

#[derive(Clone, Copy, Debug)]
struct ActorStatsLineLayout {
    x: f32,
    y: f32,
    size: f32,
    group_gap: f32,
}

#[derive(Clone, Copy, Debug)]
struct StatusRowLayout {
    x: f32,
    y: f32,
    width: f32,
    size: f32,
    gap: f32,
}

#[derive(Default)]
struct RestoredStatePayload {
    reward: Option<RewardState>,
    shop: Option<ShopState>,
    event: Option<EventState>,
    module_select: Option<ModuleSelectState>,
    combat: Option<CombatState>,
    log: VecDeque<String>,
}

#[derive(Clone, Debug)]
struct RestartConfirmLayout {
    modal_rect: Rect,
    restart_button: FittedPrimaryButton,
    cancel_button: FittedPrimaryButton,
    title_lines: Vec<String>,
    title_size: f32,
}

#[derive(Clone, Debug)]
struct SettingsLayout {
    modal_rect: Rect,
    english_button: FittedPrimaryButton,
    spanish_button: FittedPrimaryButton,
    title_lines: Vec<String>,
    subtitle_lines: Vec<String>,
    title_size: f32,
    subtitle_size: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum InstallCapability {
    Unavailable,
    PromptAvailable,
    IosGuide,
    Installed,
}

impl InstallCapability {
    pub(crate) fn from_code(code: u32) -> Self {
        match code {
            1 => Self::PromptAvailable,
            2 => Self::IosGuide,
            3 => Self::Installed,
            _ => Self::Unavailable,
        }
    }

    fn button_visible(self) -> bool {
        matches!(self, Self::PromptAvailable | Self::IosGuide)
    }
}

#[derive(Clone, Debug)]
struct InstallHelpLayout {
    modal_rect: Rect,
    close_button: FittedPrimaryButton,
    title_lines: Vec<String>,
    body_lines: Vec<String>,
    title_size: f32,
    body_size: f32,
}

#[derive(Clone, Copy, Debug)]
struct FittedPrimaryButton {
    rect: Rect,
    font_size: f32,
}

#[derive(Clone, Debug)]
struct OpeningIntroProgress {
    line_alphas: Vec<f32>,
    complete: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OverlayButtonFlow {
    Row,
    Stack,
}

#[derive(Clone, Debug)]
struct OverlayButtonMetrics {
    flow: OverlayButtonFlow,
    font_size: f32,
    item_gap: f32,
    widths: Vec<f32>,
    height: f32,
    block_w: f32,
    block_h: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RewardFollowup {
    completed_run: bool,
}

#[derive(Clone, Debug)]
struct RewardState {
    tier: RewardTier,
    options: Vec<CardId>,
    followup: RewardFollowup,
    seed: u64,
}

#[derive(Clone, Debug)]
struct ShopState {
    offers: Vec<ShopOffer>,
    seed: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ModuleSelectContext {
    Starter,
    BossReward { boss_level: usize },
}

#[derive(Clone, Debug)]
struct ModuleSelectState {
    options: Vec<ModuleId>,
    seed: u64,
    context: ModuleSelectContext,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct EventState {
    event: EventId,
}

#[derive(Clone, Debug)]
struct LevelIntroState {
    level: usize,
    codename: &'static str,
    summary: &'static str,
}

#[derive(Clone, Debug, Default)]
struct OpeningIntroState {
    elapsed_ms: f32,
    button_transition_ms: f32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FinalVictorySummary {
    act_names: Vec<&'static str>,
    total_levels: usize,
    player_hp: i32,
    player_max_hp: i32,
    deck_count: usize,
    seed: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DefeatSummary {
    current_level: usize,
    total_levels: usize,
    sector_name: &'static str,
    failure_room: Option<RoomKind>,
    failure_enemy: Option<&'static str>,
    combats_cleared: usize,
    elites_cleared: usize,
    rests_completed: usize,
    bosses_cleared: usize,
    player_hp: i32,
    player_max_hp: i32,
    deck_count: usize,
    seed: u64,
}

#[derive(Clone, Debug)]
struct RestState {
    heal_amount: i32,
    upgrade_options: Vec<usize>,
}

#[derive(Clone, Debug)]
struct RestPageInfo {
    current_page: usize,
    page_count: usize,
    columns: usize,
    visible_upgrade_indices: Vec<usize>,
}

#[derive(Clone, Copy, Debug)]
struct CardBoxMetrics {
    pad_x: f32,
    top_pad: f32,
    bottom_pad: f32,
    title_size: f32,
    cost_size: f32,
    body_size: f32,
    body_max_width: f32,
    title_gap: f32,
    title_body_gap: f32,
    body_gap: f32,
    title_chars: usize,
    body_chars: usize,
    min_height: f32,
}

#[derive(Clone, Copy, Debug)]
struct EnemyPanelMetrics {
    info_body_size: f32,
    info_body_line_gap: f32,
    info_body_chars: usize,
    stats_size: f32,
    status_size: f32,
    status_row_height: f32,
    status_gap: f32,
    top_pad: f32,
    line_gap: f32,
    width: f32,
    height: f32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct EnemyIntentLines {
    first_line_label: String,
    first_line_summary: String,
    continuation_lines: Vec<String>,
}

impl EnemyIntentLines {
    fn first_line_width(&self, font_size: f32) -> f32 {
        text_width(&self.first_line_label, font_size)
            + text_width(&self.first_line_summary, font_size)
    }

    fn max_width(&self, font_size: f32) -> f32 {
        self.continuation_lines
            .iter()
            .map(|line| text_width(line, font_size))
            .fold(self.first_line_width(font_size), f32::max)
    }

    fn line_count(&self) -> usize {
        1 + self.continuation_lines.len()
    }
}

#[derive(Clone, Copy, Debug)]
struct PlayerPanelMetrics {
    label_size: f32,
    stats_size: f32,
    meta_size: f32,
    status_size: f32,
    status_row_height: f32,
    status_gap: f32,
    top_pad: f32,
    line_gap: f32,
    width: f32,
    height: f32,
}

#[derive(Clone, Copy, Debug)]
struct TileInsets {
    pad_x: f32,
    top_pad: f32,
    bottom_pad: f32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CombatGridArrangement {
    row_counts: Vec<usize>,
}

impl CombatGridArrangement {
    fn empty() -> Self {
        Self {
            row_counts: Vec::new(),
        }
    }

    fn balanced(item_count: usize, row_count: usize) -> Self {
        debug_assert!(row_count > 0);
        debug_assert!(row_count <= item_count);

        let base = item_count / row_count;
        let remainder = item_count % row_count;
        let row_counts = (0..row_count)
            .map(|row| base + usize::from(row < remainder))
            .collect();

        Self { row_counts }
    }

    fn item_count(&self) -> usize {
        self.row_counts.iter().sum()
    }

    fn row_count(&self) -> usize {
        self.row_counts.len()
    }

    fn is_empty(&self) -> bool {
        self.row_counts.is_empty()
    }
}

#[derive(Clone, Debug)]
struct CombatLayoutPlan {
    hand: CombatGridArrangement,
    enemies: CombatGridArrangement,
    low_hand_layout: bool,
    tile_scale: f32,
    score: CombatLayoutScore,
}

#[derive(Clone, Copy, Debug)]
struct CombatLayoutScore {
    fits: bool,
    hand_card_w: f32,
    tile_scale: f32,
}

#[derive(Clone, Copy, Debug)]
struct CombatLayoutContext {
    tile_gap: f32,
    start_button: Rect,
    restart_button: Rect,
    clear_save_button: Option<Rect>,
}

pub(crate) struct App {
    screen: AppScreen,
    combat: CombatState,
    dungeon: Option<DungeonRun>,
    rest: Option<RestState>,
    shop: Option<ShopState>,
    event: Option<EventState>,
    module_select: Option<ModuleSelectState>,
    reward: Option<RewardState>,
    level_intro: Option<LevelIntroState>,
    opening_intro: Option<OpeningIntroState>,
    ui: UiState,
    viewport: Viewport,
    pointer_pos: Option<(f32, f32)>,
    floaters: Vec<Floater>,
    pixel_shards: Vec<PixelShard>,
    enemy_vfx_rects: Vec<Option<Rect>>,
    enemy_defeat_vfx_started: Vec<bool>,
    combat_feedback: CombatFeedbackState,
    layout_transition: Option<LayoutTransition>,
    combat_layout_target: Option<Layout>,
    screen_transition: Option<ScreenTransition>,
    log: VecDeque<String>,
    frame: Vec<u8>,
    dirty: bool,
    boot_time_ms: f32,
    language: Language,
    language_generation: u32,
    install_capability: InstallCapability,
    restart_count: u64,
    seed_entropy: u64,
    debug_mode: bool,
    share_request: Option<String>,
    victory_burst_cooldown_ms: f32,
    has_saved_run: bool,
    run_save_snapshot: Option<String>,
    run_save_generation: u32,
    resume_request_pending: bool,
    install_request_pending: bool,
    update_available: bool,
    update_request_pending: bool,
    restore_buffer: Vec<u8>,
}

impl App {
    pub(crate) fn new() -> Self {
        let (combat, _) = CombatState::new(BASE_SEED);
        let enemy_count = combat.enemy_count();
        let displayed_stats = displayed_combat_stats(&combat);
        let displayed_intents = displayed_enemy_intents(&combat, Language::English);

        Self {
            screen: AppScreen::Boot,
            combat,
            dungeon: None,
            rest: None,
            shop: None,
            event: None,
            module_select: None,
            reward: None,
            level_intro: None,
            opening_intro: None,
            ui: UiState::default(),
            viewport: Viewport {
                width: LOGICAL_WIDTH,
                height: LOGICAL_HEIGHT,
            },
            pointer_pos: None,
            floaters: Vec::new(),
            pixel_shards: Vec::new(),
            enemy_vfx_rects: Vec::new(),
            enemy_defeat_vfx_started: vec![false; enemy_count],
            combat_feedback: CombatFeedbackState {
                displayed: displayed_stats,
                displayed_intents,
                ..CombatFeedbackState::default()
            },
            layout_transition: None,
            combat_layout_target: None,
            screen_transition: None,
            log: VecDeque::new(),
            frame: Vec::new(),
            dirty: true,
            boot_time_ms: 0.0,
            language: Language::English,
            language_generation: 0,
            install_capability: InstallCapability::Unavailable,
            restart_count: 0,
            seed_entropy: BASE_SEED ^ 0x51A7_C0DE_1EAF_BAAD,
            debug_mode: false,
            share_request: None,
            victory_burst_cooldown_ms: 0.0,
            has_saved_run: false,
            run_save_snapshot: None,
            run_save_generation: 0,
            resume_request_pending: false,
            install_request_pending: false,
            update_available: false,
            update_request_pending: false,
            restore_buffer: Vec::new(),
        }
    }

    fn logical_width(&self) -> f32 {
        self.viewport.width.max(1.0)
    }

    fn logical_height(&self) -> f32 {
        self.viewport.height.max(1.0)
    }

    fn logical_center_x(&self) -> f32 {
        self.logical_width() * 0.5
    }

    pub(crate) fn is_boot_screen(&self) -> bool {
        matches!(self.screen, AppScreen::Boot)
    }

    fn legend_visible(&self) -> bool {
        self.ui.legend_progress > 0.001
    }

    fn legend_eased_progress(&self) -> f32 {
        ease_out_cubic(self.ui.legend_progress)
    }

    fn run_info_visible(&self) -> bool {
        self.ui.run_info_progress > 0.001
    }

    fn run_info_eased_progress(&self) -> f32 {
        ease_out_cubic(self.ui.run_info_progress)
    }

    fn enemy_inspect_visible(&self) -> bool {
        self.ui.enemy_inspect_progress > 0.001
    }

    fn enemy_inspect_eased_progress(&self) -> f32 {
        ease_out_cubic(self.ui.enemy_inspect_progress)
    }

    fn restart_confirm_visible(&self) -> bool {
        self.ui.restart_confirm_progress > 0.001
    }

    fn restart_confirm_eased_progress(&self) -> f32 {
        ease_out_cubic(self.ui.restart_confirm_progress)
    }

    fn settings_visible(&self) -> bool {
        self.ui.settings_progress > 0.001
    }

    fn settings_eased_progress(&self) -> f32 {
        ease_out_cubic(self.ui.settings_progress)
    }

    fn install_help_visible(&self) -> bool {
        self.ui.install_help_progress > 0.001
    }

    fn install_help_eased_progress(&self) -> f32 {
        ease_out_cubic(self.ui.install_help_progress)
    }

    pub(crate) fn set_language(&mut self, language: Language) {
        if self.language == language {
            return;
        }
        self.language = language;
        if matches!(self.screen, AppScreen::Combat) {
            self.relocalize_combat_feedback_intents();
        }
        self.language_generation = self.language_generation.wrapping_add(1);
        self.refresh_hover();
        self.dirty = true;
        if self.dirty {
            self.rebuild_frame();
        }
    }

    pub(crate) fn language_code(&self) -> u32 {
        self.language.code()
    }

    pub(crate) fn language_generation(&self) -> u32 {
        self.language_generation
    }

    pub(crate) fn set_install_capability(&mut self, capability: InstallCapability) {
        if self.install_capability == capability {
            return;
        }
        self.install_capability = capability;
        if !capability.button_visible() {
            self.ui.install_help_open = false;
            self.install_request_pending = false;
        }
        self.refresh_hover();
        self.dirty = true;
        if self.dirty {
            self.rebuild_frame();
        }
    }

    pub(crate) fn set_update_available(&mut self, available: bool) {
        if self.update_available == available {
            return;
        }
        self.update_available = available;
        if !available {
            self.update_request_pending = false;
        }
        self.refresh_hover();
        self.dirty = true;
        if self.dirty {
            self.rebuild_frame();
        }
    }

    pub(crate) fn install_request_pending(&self) -> bool {
        self.install_request_pending
    }

    pub(crate) fn clear_install_request(&mut self) {
        self.install_request_pending = false;
    }

    pub(crate) fn update_request_pending(&self) -> bool {
        self.update_request_pending
    }

    pub(crate) fn clear_update_request(&mut self) {
        self.update_request_pending = false;
    }

    fn tr<'a>(&self, english: &'a str, spanish: &'a str) -> &'a str {
        localized_text(self.language, english, spanish)
    }

    fn tick_opening_intro(&mut self, dt_ms: f32) -> bool {
        if !matches!(self.screen, AppScreen::OpeningIntro) || self.screen_transition.is_some() {
            return false;
        }

        let total_duration_ms = self.opening_intro_total_duration_ms();
        let Some(opening_intro) = self.opening_intro.as_mut() else {
            return false;
        };
        let mut changed = false;
        if opening_intro.elapsed_ms < total_duration_ms {
            let next_elapsed_ms = (opening_intro.elapsed_ms + dt_ms).min(total_duration_ms);
            if (next_elapsed_ms - opening_intro.elapsed_ms).abs() > f32::EPSILON {
                opening_intro.elapsed_ms = next_elapsed_ms;
                changed = true;
            }
        }
        if opening_intro.elapsed_ms >= total_duration_ms
            && opening_intro.button_transition_ms < OPENING_INTRO_BUTTON_TRANSITION_MS
        {
            let next_transition_ms = (opening_intro.button_transition_ms + dt_ms)
                .min(OPENING_INTRO_BUTTON_TRANSITION_MS);
            if (next_transition_ms - opening_intro.button_transition_ms).abs() > f32::EPSILON {
                opening_intro.button_transition_ms = next_transition_ms;
                changed = true;
            }
        }

        changed
    }

    fn localized_card_def(&self, card: CardId) -> CardDef {
        localized_card_def(card, self.language)
    }

    fn localized_combat_enemy_intent(&self, enemy_index: usize) -> Option<EnemyIntent> {
        self.combat
            .enemy(enemy_index)
            .map(|enemy| localized_enemy_intent(enemy.profile, enemy.intent_index, self.language))
    }

    fn relocalize_combat_feedback_intents(&mut self) {
        for enemy_index in 0..self.combat_feedback.displayed_intents.len() {
            if let Some(intent) = self.localized_combat_enemy_intent(enemy_index) {
                self.combat_feedback.displayed_intents[enemy_index] = intent;
            }
        }
    }

    fn combat_card_description(&self, card: CardId) -> String {
        let def = self.localized_card_def(card);
        scaled_card_description(def.description, self.combat.player.fighter.statuses)
    }

    fn localized_module_def(&self, module: ModuleId) -> ModuleDef {
        localized_module_def(module, self.language)
    }

    fn module_select_title_for(&self, context: ModuleSelectContext) -> &'static str {
        match context {
            ModuleSelectContext::Starter => self.tr("Choose your module", "Elige tu modulo"),
            ModuleSelectContext::BossReward { .. } => self.tr("Choose a module", "Elige un modulo"),
        }
    }

    fn module_select_title(&self) -> &'static str {
        self.module_select
            .as_ref()
            .map(|module_select| self.module_select_title_for(module_select.context))
            .unwrap_or_else(|| self.tr("Choose your module", "Elige tu modulo"))
    }

    fn run_info_modules_block_height(
        &self,
        modules: &[ModuleId],
        inner_w: f32,
        module_name_size: f32,
        module_body_size: f32,
        module_title_top_gap: f32,
        module_gap: f32,
    ) -> f32 {
        if modules.is_empty() {
            return module_body_size;
        }

        let module_chars = ((inner_w / (module_body_size * 0.62)).floor() as usize).max(14);
        modules
            .iter()
            .enumerate()
            .fold(0.0, |height, (module_index, module)| {
                let body_lines =
                    wrap_text(self.localized_module_def(*module).description, module_chars)
                        .len()
                        .max(1);
                let mut next_height = height
                    + module_title_top_gap
                    + module_name_size
                    + 6.0
                    + body_lines as f32 * module_body_size
                    + body_lines.saturating_sub(1) as f32 * 5.0;
                if module_index + 1 < modules.len() {
                    next_height += module_gap;
                }
                next_height
            })
    }

    fn sync_combat_feedback_to_combat(&mut self) {
        self.enemy_defeat_vfx_started = (0..self.combat.enemy_count())
            .map(|enemy_index| !self.combat.enemy_is_alive(enemy_index))
            .collect();
        self.combat_feedback = CombatFeedbackState {
            displayed: displayed_combat_stats(&self.combat),
            displayed_intents: displayed_enemy_intents(&self.combat, self.language),
            ..CombatFeedbackState::default()
        };
    }

    fn combat_input_locked(&self) -> bool {
        matches!(self.screen, AppScreen::Combat)
            && (self.combat_feedback.auto_playback_active
                || self.combat_feedback.pending_outcome.is_some()
                || self.combat_feedback.playback_pause_ms > 0.0
                || !self.combat_feedback.active_stats.is_empty()
                || !self.combat_feedback.stat_queue.is_empty())
    }

    fn current_displayed_intent(&self, enemy_index: usize) -> EnemyIntent {
        self.combat_feedback
            .displayed_intents
            .get(enemy_index)
            .copied()
            .or_else(|| self.localized_combat_enemy_intent(enemy_index))
            .unwrap_or(crate::content::EnemyIntent {
                name: self.tr("Offline", "Sin señal"),
                summary: self.tr("No next signal.", "Sin siguiente señal."),
                damage: 0,
                hits: 0,
                gain_block: 0,
                prime_bleed: 0,
                self_focus: 0,
                self_rhythm: 0,
                self_momentum: 0,
                target_focus: 0,
                target_rhythm: 0,
                target_momentum: 0,
                apply_bleed: 0,
            })
    }

    fn enemy_display_name(&self, enemy_index: usize) -> &'static str {
        self.combat
            .enemy_profile(enemy_index)
            .map(|profile| localized_enemy_name(profile, self.language))
            .unwrap_or(self.tr("Enemy", "Enemigo"))
    }

    fn enemy_turn_label(&self) -> String {
        let living_enemies: Vec<_> = (0..self.combat.enemy_count())
            .filter(|enemy_index| self.combat.enemy_is_alive(*enemy_index))
            .collect();
        match living_enemies.as_slice() {
            [enemy_index] => self.enemy_display_name(*enemy_index).to_string(),
            _ => "Enemies".to_string(),
        }
    }

    fn enemy_signal_summary(&self, enemy_index: usize) -> &'static str {
        if self
            .combat
            .enemy(enemy_index)
            .is_some_and(|enemy| enemy.fighter.hp > 0)
        {
            self.current_displayed_intent(enemy_index).summary
        } else {
            self.tr("No next signal.", "Sin siguiente señal.")
        }
    }

    fn visible_enemy_indices(&self) -> Vec<usize> {
        (0..self.combat.enemy_count())
            .filter(|enemy_index| {
                self.combat.enemy_is_alive(*enemy_index)
                    || !self
                        .enemy_defeat_vfx_started
                        .get(*enemy_index)
                        .copied()
                        .unwrap_or(false)
            })
            .collect()
    }

    fn displayed_actor_stats(&self, actor: Actor) -> ActorDisplayedStats {
        match actor {
            Actor::Player => self.combat_feedback.displayed.player,
            Actor::Enemy(enemy_index) => self
                .combat_feedback
                .displayed
                .enemies
                .get(enemy_index)
                .copied()
                .or_else(|| {
                    self.combat
                        .enemy(enemy_index)
                        .map(|enemy| ActorDisplayedStats {
                            hp: enemy.fighter.hp,
                            block: enemy.fighter.block,
                        })
                })
                .unwrap_or_default(),
        }
    }

    fn displayed_player_meta(&self) -> PlayerDisplayedMeta {
        let displayed = self.combat_feedback.displayed.player_meta;
        if displayed == PlayerDisplayedMeta::default()
            && self.combat_feedback.playback_kind.is_none()
            && self.combat_feedback.active_stats.is_empty()
            && self.combat_feedback.stat_queue.is_empty()
        {
            return PlayerDisplayedMeta {
                energy: self.combat.player.energy as i32,
                draw_pile: self.combat.deck.draw_pile.len() as i32,
                discard_pile: self.combat.deck.discard_pile.len() as i32,
            };
        }
        displayed
    }

    fn displayed_player_meta_mut(&mut self) -> &mut PlayerDisplayedMeta {
        &mut self.combat_feedback.displayed.player_meta
    }

    fn sync_displayed_player_meta_to_combat(&mut self) {
        self.combat_feedback.displayed.player_meta = PlayerDisplayedMeta {
            energy: self.combat.player.energy as i32,
            draw_pile: self.combat.deck.draw_pile.len() as i32,
            discard_pile: self.combat.deck.discard_pile.len() as i32,
        };
    }

    fn queue_player_meta_stat_set(&mut self, stat: CombatStat, target: i32) {
        let current = self.displayed_stat_value(Actor::Player, stat);
        self.queue_stat_change(QueuedStatChange {
            actor: Actor::Player,
            stat,
            op: StatChangeOp::Set(target),
            tint: if target >= current {
                StatTint::BlockGain
            } else {
                StatTint::NeutralLoss
            },
        });
    }

    fn queue_player_meta_stat_group(
        &mut self,
        changes: impl IntoIterator<Item = QueuedStatChange>,
    ) {
        self.queue_stat_change_group(
            changes
                .into_iter()
                .filter(|change| match &change.op {
                    StatChangeOp::Values(values) => !values.is_empty(),
                    _ => true,
                })
                .collect::<Vec<_>>(),
        );
    }

    fn queue_player_reshuffle_meta_animation(&mut self) {
        let displayed = self.displayed_player_meta();
        let transfer = displayed.discard_pile.max(0);
        let mut draw_values = stat_countdown_values(displayed.draw_pile, 0);
        if transfer > 0 {
            draw_values.push(transfer);
        }
        let mut discard_values = stat_countdown_values(displayed.discard_pile, transfer);
        if transfer > 0 || displayed.discard_pile > 0 {
            discard_values.push(0);
        }

        self.queue_player_meta_stat_group([
            QueuedStatChange {
                actor: Actor::Player,
                stat: CombatStat::DrawPile,
                op: StatChangeOp::Values(draw_values),
                tint: StatTint::NeutralLoss,
            },
            QueuedStatChange {
                actor: Actor::Player,
                stat: CombatStat::DiscardPile,
                op: StatChangeOp::Values(discard_values),
                tint: StatTint::BlockGain,
            },
        ]);
    }

    fn mark_enemy_defeat_vfx_started(&mut self, enemy_index: usize) {
        if self.enemy_defeat_vfx_started.len() <= enemy_index {
            self.enemy_defeat_vfx_started.resize(enemy_index + 1, false);
        }
        self.enemy_defeat_vfx_started[enemy_index] = true;
    }

    fn begin_enemy_defeat_vfx(&mut self, enemy_index: usize) {
        if self
            .enemy_defeat_vfx_started
            .get(enemy_index)
            .copied()
            .unwrap_or(false)
        {
            return;
        }

        let from_layout = self.layout();
        let burst_rect = from_layout
            .enemy_rect(enemy_index)
            .or_else(|| self.enemy_vfx_rects.get(enemy_index).copied().flatten());
        self.mark_enemy_defeat_vfx_started(enemy_index);
        if let Some(rect) = burst_rect {
            self.spawn_enemy_pixel_burst(rect);
        }
        if matches!(self.screen, AppScreen::Combat) {
            self.begin_layout_transition(from_layout, self.combat.hand_len(), None);
        }
    }

    fn displayed_actor_stats_mut(&mut self, actor: Actor) -> &mut ActorDisplayedStats {
        match actor {
            Actor::Player => &mut self.combat_feedback.displayed.player,
            Actor::Enemy(enemy_index) => {
                while self.combat_feedback.displayed.enemies.len() <= enemy_index {
                    let next_index = self.combat_feedback.displayed.enemies.len();
                    let next = self
                        .combat
                        .enemy(next_index)
                        .map(|enemy| ActorDisplayedStats {
                            hp: enemy.fighter.hp,
                            block: enemy.fighter.block,
                        })
                        .unwrap_or_default();
                    self.combat_feedback.displayed.enemies.push(next);
                }
                &mut self.combat_feedback.displayed.enemies[enemy_index]
            }
        }
    }

    fn displayed_stat_value(&self, actor: Actor, stat: CombatStat) -> i32 {
        match stat {
            CombatStat::Hp => self.displayed_actor_stats(actor).hp,
            CombatStat::Block => self.displayed_actor_stats(actor).block,
            CombatStat::Energy => {
                debug_assert!(matches!(actor, Actor::Player));
                self.displayed_player_meta().energy
            }
            CombatStat::DrawPile => {
                debug_assert!(matches!(actor, Actor::Player));
                self.displayed_player_meta().draw_pile
            }
            CombatStat::DiscardPile => {
                debug_assert!(matches!(actor, Actor::Player));
                self.displayed_player_meta().discard_pile
            }
        }
    }

    fn set_displayed_stat_value(&mut self, actor: Actor, stat: CombatStat, value: i32) {
        match stat {
            CombatStat::Hp => self.displayed_actor_stats_mut(actor).hp = value.max(0),
            CombatStat::Block => self.displayed_actor_stats_mut(actor).block = value.max(0),
            CombatStat::Energy => {
                debug_assert!(matches!(actor, Actor::Player));
                self.displayed_player_meta_mut().energy = value.max(0);
            }
            CombatStat::DrawPile => {
                debug_assert!(matches!(actor, Actor::Player));
                self.displayed_player_meta_mut().draw_pile = value.max(0);
            }
            CombatStat::DiscardPile => {
                debug_assert!(matches!(actor, Actor::Player));
                self.displayed_player_meta_mut().discard_pile = value.max(0);
            }
        }
    }

    fn queue_stat_change(&mut self, change: QueuedStatChange) {
        self.queue_stat_change_group([change]);
    }

    fn queue_stat_change_group(&mut self, changes: impl IntoIterator<Item = QueuedStatChange>) {
        let changes: Vec<_> = changes.into_iter().collect();
        if changes.is_empty() {
            return;
        }
        self.combat_feedback.stat_queue.push_back(changes);
    }

    fn prime_next_stat_countdown_if_idle(&mut self) -> bool {
        if !self.combat_feedback.active_stats.is_empty() {
            return false;
        }

        while let Some(changes) = self.combat_feedback.stat_queue.pop_front() {
            let mut countdowns = Vec::new();

            for change in changes {
                let current = self.displayed_stat_value(change.actor, change.stat);
                let (values, target) = match change.op {
                    StatChangeOp::Add(amount) => {
                        let target = current.saturating_add(amount.max(0));
                        (stat_countdown_values(current, target), target)
                    }
                    StatChangeOp::Subtract(amount) => {
                        let target = current.saturating_sub(amount.max(0));
                        (stat_countdown_values(current, target), target)
                    }
                    StatChangeOp::Set(value) => {
                        let target = value.max(0);
                        (stat_countdown_values(current, target), target)
                    }
                    StatChangeOp::Values(values) => {
                        let target = values.last().copied().unwrap_or(current);
                        (values, target)
                    }
                };

                if values.is_empty() {
                    self.set_displayed_stat_value(change.actor, change.stat, target);
                    continue;
                }

                countdowns.push(StatCountdown {
                    actor: change.actor,
                    stat: change.stat,
                    values,
                    target,
                    ttl_ms: COMBAT_PLAYBACK_STEP_MS,
                    total_ms: COMBAT_PLAYBACK_STEP_MS,
                    tint: change.tint,
                });
            }

            if !countdowns.is_empty() {
                self.combat_feedback.active_stats = countdowns;
                return true;
            }
        }

        false
    }

    fn show_turn_banner(&mut self, actor: Actor) {
        let (text, color) = match actor {
            Actor::Player => (self.tr("Your Turn", "Tu turno"), (61, 245, 255)),
            Actor::Enemy(_) => (self.tr("Enemy Turn", "Turno enemigo"), (255, 156, 240)),
        };
        self.combat_feedback.turn_banner = Some(TurnBanner {
            text: text.to_string(),
            color,
            ttl_ms: COMBAT_TURN_BANNER_MS,
            total_ms: COMBAT_TURN_BANNER_MS,
        });
    }

    fn begin_end_turn_playback(
        &mut self,
        events: Vec<CombatEvent>,
        displayed: DisplayedCombatStats,
        intents: Vec<EnemyIntent>,
    ) {
        self.begin_combat_playback(
            CombatPlaybackKind::EnemyTurn,
            events,
            displayed,
            intents,
            self.combat.outcome(),
        );
    }

    fn begin_player_action_playback(
        &mut self,
        events: Vec<CombatEvent>,
        displayed: DisplayedCombatStats,
        intents: Vec<EnemyIntent>,
    ) {
        self.begin_combat_playback(
            CombatPlaybackKind::PlayerAction,
            events,
            displayed,
            intents,
            self.combat.outcome(),
        );
    }

    fn begin_combat_playback(
        &mut self,
        kind: CombatPlaybackKind,
        events: Vec<CombatEvent>,
        displayed: DisplayedCombatStats,
        intents: Vec<EnemyIntent>,
        pending_outcome: Option<CombatOutcome>,
    ) {
        self.combat_feedback = CombatFeedbackState {
            displayed,
            displayed_intents: intents,
            playback_kind: Some(kind),
            playback_queue: events.into(),
            auto_playback_active: true,
            pending_outcome,
            ..CombatFeedbackState::default()
        };
    }

    fn handle_combat_event(&mut self, event: CombatEvent, playback: bool) -> f32 {
        match event {
            CombatEvent::CombatStarted => {
                if self.combat.enemy_count() > 1 {
                    self.push_log("The encounter begins.");
                } else {
                    self.push_log("The duel begins.");
                }
                COMBAT_PLAYBACK_PAUSE_LOG_MS
            }
            CombatEvent::TurnStarted { actor, turn } => {
                match actor {
                    Actor::Player => self.push_log(match self.language {
                        Language::English => format!("Turn {turn}. Fresh hand, full focus."),
                        Language::Spanish => {
                            format!("Turno {turn}. Mano nueva, foco total.")
                        }
                    }),
                    Actor::Enemy(_) => self.push_log(match self.language {
                        Language::English => format!("{} act.", self.enemy_turn_label()),
                        Language::Spanish => format!("{} actuan.", self.enemy_turn_label()),
                    }),
                }
                if playback && matches!(actor, Actor::Player) {
                    self.queue_player_meta_stat_set(
                        CombatStat::Energy,
                        self.combat.player.energy as i32,
                    );
                }
                if playback {
                    self.show_turn_banner(actor);
                    COMBAT_TURN_BANNER_MS * 0.5
                } else {
                    0.0
                }
            }
            CombatEvent::TurnEnded { actor } => {
                match actor {
                    Actor::Player => {
                        self.push_log(self.tr("You yield the initiative.", "Cedes la iniciativa."))
                    }
                    Actor::Enemy(_) => self.push_log(match self.language {
                        Language::English => {
                            format!("{} fall back.", self.enemy_turn_label())
                        }
                        Language::Spanish => {
                            format!("{} retroceden.", self.enemy_turn_label())
                        }
                    }),
                }
                COMBAT_PLAYBACK_PAUSE_LOG_MS
            }
            CombatEvent::CardDrawn { .. } => {
                if playback
                    && self.combat_feedback.playback_kind == Some(CombatPlaybackKind::EnemyTurn)
                {
                    self.queue_player_meta_stat_set(
                        CombatStat::DrawPile,
                        self.displayed_player_meta().draw_pile.saturating_sub(1),
                    );
                } else if playback
                    && self.combat_feedback.playback_kind == Some(CombatPlaybackKind::PlayerAction)
                {
                    self.sync_displayed_player_meta_to_combat();
                }
                0.0
            }
            CombatEvent::CardPlayed { card } => {
                self.push_log(match self.language {
                    Language::English => {
                        format!("Played {}.", localized_card_name(card, self.language))
                    }
                    Language::Spanish => {
                        format!("Jugaste {}.", localized_card_name(card, self.language))
                    }
                });
                if playback
                    && self.combat_feedback.playback_kind == Some(CombatPlaybackKind::PlayerAction)
                {
                    self.sync_displayed_player_meta_to_combat();
                }
                COMBAT_PLAYBACK_PAUSE_LOG_MS
            }
            CombatEvent::CardBurned { card } => {
                self.push_log(match self.language {
                    Language::English => format!(
                        "Hand full. {} slips to discard.",
                        localized_card_name(card, self.language)
                    ),
                    Language::Spanish => format!(
                        "La mano esta llena. {} va al descarte.",
                        localized_card_name(card, self.language)
                    ),
                });
                if playback
                    && self.combat_feedback.playback_kind == Some(CombatPlaybackKind::PlayerAction)
                {
                    self.sync_displayed_player_meta_to_combat();
                }
                COMBAT_PLAYBACK_PAUSE_LOG_MS
            }
            CombatEvent::CardCreated { card } => {
                self.push_log(match self.language {
                    Language::English => {
                        format!("Generated {}.", localized_card_name(card, self.language))
                    }
                    Language::Spanish => {
                        format!("Generaste {}.", localized_card_name(card, self.language))
                    }
                });
                if playback
                    && self.combat_feedback.playback_kind == Some(CombatPlaybackKind::PlayerAction)
                {
                    self.sync_displayed_player_meta_to_combat();
                }
                COMBAT_PLAYBACK_PAUSE_LOG_MS
            }
            CombatEvent::CardsDiscarded { count } => {
                if count > 0 {
                    self.push_log(match self.language {
                        Language::English => format!("Discarded {count} card(s)."),
                        Language::Spanish => format!("Descartaste {count} carta(s)."),
                    });
                    if playback
                        && self.combat_feedback.playback_kind == Some(CombatPlaybackKind::EnemyTurn)
                    {
                        self.queue_player_meta_stat_set(
                            CombatStat::DiscardPile,
                            self.displayed_player_meta().discard_pile + count as i32,
                        );
                    } else if playback
                        && self.combat_feedback.playback_kind
                            == Some(CombatPlaybackKind::PlayerAction)
                    {
                        self.sync_displayed_player_meta_to_combat();
                    }
                    COMBAT_PLAYBACK_PAUSE_LOG_MS
                } else {
                    0.0
                }
            }
            CombatEvent::Reshuffled => {
                self.push_log(self.tr(
                    "Discard pile reshuffled into the draw pile.",
                    "El descarte se mezclo dentro del mazo.",
                ));
                if playback
                    && self.combat_feedback.playback_kind == Some(CombatPlaybackKind::EnemyTurn)
                {
                    self.queue_player_reshuffle_meta_animation();
                } else if playback
                    && self.combat_feedback.playback_kind == Some(CombatPlaybackKind::PlayerAction)
                {
                    self.sync_displayed_player_meta_to_combat();
                }
                COMBAT_PLAYBACK_PAUSE_LOG_MS
            }
            CombatEvent::EnergySpent { .. } => {
                if playback
                    && self.combat_feedback.playback_kind == Some(CombatPlaybackKind::PlayerAction)
                {
                    self.sync_displayed_player_meta_to_combat();
                }
                0.0
            }
            CombatEvent::RequirementNotMet {
                axis,
                threshold,
                actual,
            } => {
                self.push_log(match self.language {
                    Language::English => format!(
                        "Requires {} > {}. Current {}.",
                        axis_display_name(axis, self.language),
                        threshold,
                        actual
                    ),
                    Language::Spanish => format!(
                        "Requiere {} > {}. Actual {}.",
                        axis_display_name(axis, self.language),
                        threshold,
                        actual
                    ),
                });
                if playback
                    && self.combat_feedback.playback_kind == Some(CombatPlaybackKind::PlayerAction)
                {
                    self.sync_displayed_player_meta_to_combat();
                }
                COMBAT_PLAYBACK_PAUSE_STATUS_MS
            }
            CombatEvent::DamageDealt {
                source,
                target,
                amount,
            } => {
                self.push_damage_log(source, target, amount);
                self.spawn_damage_floater(target, amount);
                if playback {
                    self.queue_stat_change(QueuedStatChange {
                        actor: target,
                        stat: CombatStat::Hp,
                        op: StatChangeOp::Subtract(amount),
                        tint: StatTint::Damage,
                    });
                }
                COMBAT_PLAYBACK_PAUSE_DAMAGE_MS
            }
            CombatEvent::BlockGained { actor, amount } => {
                self.spawn_block_floater(actor, amount);
                self.push_log(match actor {
                    Actor::Player => match self.language {
                        Language::English => {
                            format!("You gain {amount} {}.", self.tr("Shield", "Escudo"))
                        }
                        Language::Spanish => {
                            format!("Obtienes {amount} de {}.", self.tr("Shield", "Escudo"))
                        }
                    },
                    Actor::Enemy(enemy_index) => format!(
                        "{} {} {amount} {}.",
                        self.enemy_display_name(enemy_index),
                        self.tr("gains", "gana"),
                        self.tr("Shield", "Escudo")
                    ),
                });
                if playback {
                    self.queue_stat_change(QueuedStatChange {
                        actor,
                        stat: CombatStat::Block,
                        op: StatChangeOp::Add(amount),
                        tint: StatTint::BlockGain,
                    });
                }
                COMBAT_PLAYBACK_PAUSE_BLOCK_MS
            }
            CombatEvent::BlockSpent { actor, amount } => {
                self.push_log(match actor {
                    Actor::Player => match self.language {
                        Language::English => {
                            format!("Your {} absorbs {amount}.", self.tr("Shield", "Escudo"))
                        }
                        Language::Spanish => {
                            format!("Tu {} absorbe {amount}.", self.tr("Shield", "Escudo"))
                        }
                    },
                    Actor::Enemy(enemy_index) => format!(
                        "{} {} {amount}.",
                        self.enemy_display_name(enemy_index),
                        match self.language {
                            Language::English => "Shield absorbs",
                            Language::Spanish => "Escudo absorbe",
                        }
                    ),
                });
                if playback {
                    self.queue_stat_change(QueuedStatChange {
                        actor,
                        stat: CombatStat::Block,
                        op: StatChangeOp::Subtract(amount),
                        tint: StatTint::Damage,
                    });
                }
                COMBAT_PLAYBACK_PAUSE_BLOCK_MS
            }
            CombatEvent::BlockCleared { actor, amount } => {
                self.push_log(match actor {
                    Actor::Player => match self.language {
                        Language::English => {
                            format!("Your old {} fades ({amount}).", self.tr("Shield", "Escudo"))
                        }
                        Language::Spanish => format!(
                            "Tu {} anterior se desvanece ({amount}).",
                            self.tr("Shield", "Escudo")
                        ),
                    },
                    Actor::Enemy(enemy_index) => format!(
                        "{} {} ({amount}).",
                        self.enemy_display_name(enemy_index),
                        match self.language {
                            Language::English => "loses old Shield",
                            Language::Spanish => "pierde su Escudo anterior",
                        }
                    ),
                });
                if playback {
                    self.queue_stat_change(QueuedStatChange {
                        actor,
                        stat: CombatStat::Block,
                        op: StatChangeOp::Set(0),
                        tint: StatTint::NeutralLoss,
                    });
                }
                COMBAT_PLAYBACK_PAUSE_BLOCK_MS
            }
            CombatEvent::StatusApplied {
                target,
                status,
                amount,
            } => {
                let delta = signed_axis_value(amount);
                self.push_log(match (target, status) {
                    (Actor::Player, StatusKind::Bleed) => match self.language {
                        Language::English => format!(
                            "You gain {} {}.",
                            status_display_name(status, self.language),
                            amount
                        ),
                        Language::Spanish => format!(
                            "Obtienes {} {}.",
                            status_display_name(status, self.language),
                            amount
                        ),
                    },
                    (Actor::Enemy(enemy_index), StatusKind::Bleed) => format!(
                        "{} {} {} {}.",
                        self.enemy_display_name(enemy_index),
                        self.tr("gains", "gana"),
                        status_display_name(status, self.language),
                        amount
                    ),
                    (Actor::Player, _) => match self.language {
                        Language::English => format!(
                            "Your {} changes by {}.",
                            status_display_name(status, self.language),
                            delta
                        ),
                        Language::Spanish => format!(
                            "Tu {} cambia en {}.",
                            status_display_name(status, self.language),
                            delta
                        ),
                    },
                    (Actor::Enemy(enemy_index), _) => match self.language {
                        Language::English => format!(
                            "{} {} changes by {}.",
                            self.enemy_display_name(enemy_index),
                            status_display_name(status, self.language),
                            delta
                        ),
                        Language::Spanish => format!(
                            "{} {} cambia en {}.",
                            self.enemy_display_name(enemy_index),
                            status_display_name(status, self.language),
                            delta
                        ),
                    },
                });
                self.spawn_status_floater(target, status, amount);
                COMBAT_PLAYBACK_PAUSE_STATUS_MS
            }
            CombatEvent::StatusTicked {
                actor,
                status,
                amount,
            } => {
                self.push_log(match status {
                    StatusKind::Bleed => match actor {
                        Actor::Player => match self.language {
                            Language::English => format!(
                                "{} deals {amount} to you.",
                                status_display_name(status, self.language)
                            ),
                            Language::Spanish => format!(
                                "{} te hace {amount}.",
                                status_display_name(status, self.language)
                            ),
                        },
                        Actor::Enemy(enemy_index) => format!(
                            "{} {} {amount} a {}.",
                            status_display_name(status, self.language),
                            self.tr("deals", "hace"),
                            self.enemy_display_name(enemy_index)
                        ),
                    },
                    StatusKind::Focus | StatusKind::Rhythm | StatusKind::Momentum => match actor {
                        Actor::Player => match self.language {
                            Language::English => {
                                format!(
                                    "Your {} fades.",
                                    status_display_name(status, self.language)
                                )
                            }
                            Language::Spanish => format!(
                                "Tu {} se disipa.",
                                status_display_name(status, self.language)
                            ),
                        },
                        Actor::Enemy(enemy_index) => format!(
                            "{} {} {}.",
                            self.enemy_display_name(enemy_index),
                            self.tr("loses", "pierde"),
                            status_display_name(status, self.language)
                        ),
                    },
                });
                COMBAT_PLAYBACK_PAUSE_STATUS_MS
            }
            CombatEvent::ActorDefeated { actor } => {
                if playback {
                    self.combat_feedback.outcome_hold_ms = self
                        .combat_feedback
                        .outcome_hold_ms
                        .max(COMBAT_OUTCOME_VFX_HOLD_MS);
                    if let Actor::Enemy(enemy_index) = actor {
                        self.begin_enemy_defeat_vfx(enemy_index);
                    }
                }
                COMBAT_PLAYBACK_PAUSE_DAMAGE_MS
            }
            CombatEvent::EnemyPrimedBleed {
                enemy_index,
                amount,
            } => {
                self.push_log(format!(
                    "{} {} {} {amount}.",
                    self.enemy_display_name(enemy_index),
                    self.tr("next hit applies", "aplica en el prox. golpe"),
                    status_display_name(StatusKind::Bleed, self.language)
                ));
                COMBAT_PLAYBACK_PAUSE_STATUS_MS
            }
            CombatEvent::IntentAdvanced {
                enemy_index,
                intent,
            } => {
                let localized_intent = self
                    .localized_combat_enemy_intent(enemy_index)
                    .unwrap_or(intent);
                if self.combat_feedback.displayed_intents.len() <= enemy_index {
                    self.combat_feedback.displayed_intents =
                        displayed_enemy_intents(&self.combat, self.language);
                }
                if self.combat_feedback.displayed_intents.len() <= enemy_index {
                    self.combat_feedback
                        .displayed_intents
                        .resize(enemy_index + 1, localized_intent);
                }
                self.combat_feedback.displayed_intents[enemy_index] = localized_intent;
                self.push_log(format!(
                    "{} {} {}.",
                    self.enemy_display_name(enemy_index),
                    self.tr("next intent:", "siguiente accion:"),
                    localized_intent.name
                ));
                COMBAT_PLAYBACK_PAUSE_LOG_MS
            }
            CombatEvent::NotEnoughEnergy { needed, available } => {
                self.push_log(match self.language {
                    Language::English => {
                        format!("Not enough energy. Need {needed}, have {available}.")
                    }
                    Language::Spanish => {
                        format!(
                            "No hay suficiente energía. Necesitas {needed} y tienes {available}."
                        )
                    }
                });
                COMBAT_PLAYBACK_PAUSE_LOG_MS
            }
            CombatEvent::InvalidAction { reason } => {
                self.push_log(reason);
                COMBAT_PLAYBACK_PAUSE_LOG_MS
            }
            CombatEvent::CombatWon => {
                self.push_log(self.tr("Threat neutralized.", "Amenaza neutralizada."));
                COMBAT_PLAYBACK_PAUSE_DAMAGE_MS
            }
            CombatEvent::CombatLost => {
                self.push_log(self.tr("Shield collapse.", "Colapso total."));
                COMBAT_PLAYBACK_PAUSE_DAMAGE_MS
            }
        }
    }

    fn tick_combat_feedback(&mut self, dt_ms: f32) -> bool {
        let mut changed = false;

        if let Some(banner) = &mut self.combat_feedback.turn_banner {
            banner.ttl_ms -= dt_ms;
            changed = true;
        }
        if self
            .combat_feedback
            .turn_banner
            .as_ref()
            .is_some_and(|banner| banner.ttl_ms <= 0.0)
        {
            self.combat_feedback.turn_banner = None;
        }

        if self.combat_feedback.playback_pause_ms > 0.0 {
            self.combat_feedback.playback_pause_ms =
                (self.combat_feedback.playback_pause_ms - dt_ms).max(0.0);
            changed = true;
        }
        if self.combat_feedback.outcome_hold_ms > 0.0 {
            self.combat_feedback.outcome_hold_ms =
                (self.combat_feedback.outcome_hold_ms - dt_ms).max(0.0);
            changed = true;
        }

        if !self.combat_feedback.active_stats.is_empty() {
            for active in &mut self.combat_feedback.active_stats {
                active.ttl_ms -= dt_ms;
            }
            changed = true;

            loop {
                let should_advance = self
                    .combat_feedback
                    .active_stats
                    .first()
                    .is_some_and(|active| active.ttl_ms <= 0.0);
                if !should_advance {
                    break;
                }

                let mut next_active_stats = Vec::new();
                let active_stats = std::mem::take(&mut self.combat_feedback.active_stats);
                for mut active in active_stats {
                    if let Some(next_value) = active.values.first().copied() {
                        active.values.remove(0);
                        self.set_displayed_stat_value(active.actor, active.stat, next_value);
                        if let (Actor::Enemy(enemy_index), CombatStat::Hp) =
                            (active.actor, active.stat)
                        {
                            if next_value <= 0 && !self.combat.enemy_is_alive(enemy_index) {
                                self.begin_enemy_defeat_vfx(enemy_index);
                            }
                        }
                        changed = true;
                        if !active.values.is_empty() {
                            active.ttl_ms += active.total_ms;
                            next_active_stats.push(active);
                        }
                    } else {
                        self.set_displayed_stat_value(active.actor, active.stat, active.target);
                        if let (Actor::Enemy(enemy_index), CombatStat::Hp) =
                            (active.actor, active.stat)
                        {
                            if active.target <= 0 && !self.combat.enemy_is_alive(enemy_index) {
                                self.begin_enemy_defeat_vfx(enemy_index);
                            }
                        }
                        changed = true;
                    }
                }
                self.combat_feedback.active_stats = next_active_stats;
            }
        }

        changed |= self.prime_next_stat_countdown_if_idle();

        if self.combat_feedback.auto_playback_active
            && self.combat_feedback.playback_pause_ms <= 0.0
            && self.combat_feedback.active_stats.is_empty()
            && self.combat_feedback.stat_queue.is_empty()
        {
            if let Some(event) = self.combat_feedback.playback_queue.pop_front() {
                self.combat_feedback.playback_pause_ms = self.handle_combat_event(event, true);
                changed = true;
                changed |= self.prime_next_stat_countdown_if_idle();
            } else {
                self.combat_feedback.auto_playback_active = false;
                changed = true;
            }
        }

        if !self.combat_feedback.auto_playback_active
            && self.combat_feedback.pending_outcome.is_some()
            && self.combat_feedback.playback_pause_ms <= 0.0
            && self.combat_feedback.outcome_hold_ms <= 0.0
            && self.combat_feedback.active_stats.is_empty()
            && self.combat_feedback.stat_queue.is_empty()
        {
            self.finalize_pending_combat_outcome();
            changed = true;
        }

        if !self.combat_feedback.auto_playback_active
            && self.combat_feedback.pending_outcome.is_none()
            && self.combat_feedback.playback_pause_ms <= 0.0
            && self.combat_feedback.active_stats.is_empty()
            && self.combat_feedback.stat_queue.is_empty()
            && self.combat_feedback.playback_kind.is_some()
        {
            let from_layout = self.layout();
            let previous_kind = self.combat_feedback.playback_kind.take();
            if previous_kind == Some(CombatPlaybackKind::EnemyTurn) && self.combat.hand_len() > 0 {
                self.begin_hand_reveal_transition(from_layout);
            }
            changed = true;
        }

        changed
    }

    pub(crate) fn initialize(&mut self, width: f32, height: f32) {
        self.resize(width, height);
        self.rebuild_frame();
    }

    pub(crate) fn resize(&mut self, width: f32, height: f32) {
        self.viewport.width = width.max(1.0);
        self.viewport.height = height.max(1.0);
        if matches!(self.screen, AppScreen::Rest) {
            self.sync_rest_page_state();
        }
        self.pointer_pos = None;
        self.ui.hover = None;
        self.layout_transition = None;
        self.screen_transition = None;
        self.combat_layout_target = if matches!(self.screen, AppScreen::Combat) {
            Some(self.layout_target())
        } else {
            None
        };
        self.dirty = true;
    }

    pub(crate) fn tick(&mut self, dt_ms: f32) {
        let dt_ms = dt_ms.clamp(0.0, 64.0);
        self.boot_time_ms += dt_ms;
        let combat_locked_before = self.combat_input_locked();
        self.snapshot_combat_layout_target();

        let mut changed = matches!(
            self.screen,
            AppScreen::Boot | AppScreen::OpeningIntro | AppScreen::Map | AppScreen::LevelIntro
        );
        changed |= self.tick_opening_intro(dt_ms);
        let legend_target = if self.ui.legend_open { 1.0 } else { 0.0 };
        if (self.ui.legend_progress - legend_target).abs() > 0.001 {
            let step = (dt_ms / LEGEND_TRANSITION_MS).clamp(0.0, 1.0);
            self.ui.legend_progress = if self.ui.legend_progress < legend_target {
                (self.ui.legend_progress + step).min(legend_target)
            } else {
                (self.ui.legend_progress - step).max(legend_target)
            };
            self.refresh_hover();
            changed = true;
        } else {
            self.ui.legend_progress = legend_target;
        }
        let run_info_target = if self.ui.run_info_open { 1.0 } else { 0.0 };
        if (self.ui.run_info_progress - run_info_target).abs() > 0.001 {
            let step = (dt_ms / LEGEND_TRANSITION_MS).clamp(0.0, 1.0);
            self.ui.run_info_progress = if self.ui.run_info_progress < run_info_target {
                (self.ui.run_info_progress + step).min(run_info_target)
            } else {
                (self.ui.run_info_progress - step).max(run_info_target)
            };
            self.refresh_hover();
            changed = true;
        } else {
            self.ui.run_info_progress = run_info_target;
        }
        if self.ui.enemy_inspect_open
            && !self
                .ui
                .enemy_inspect_enemy
                .is_some_and(|enemy_index| self.combat.enemy_is_alive(enemy_index))
        {
            self.ui.enemy_inspect_open = false;
            self.refresh_hover();
            changed = true;
        }
        let enemy_inspect_target = if self.ui.enemy_inspect_open { 1.0 } else { 0.0 };
        if (self.ui.enemy_inspect_progress - enemy_inspect_target).abs() > 0.001 {
            let step = (dt_ms / LEGEND_TRANSITION_MS).clamp(0.0, 1.0);
            self.ui.enemy_inspect_progress =
                if self.ui.enemy_inspect_progress < enemy_inspect_target {
                    (self.ui.enemy_inspect_progress + step).min(enemy_inspect_target)
                } else {
                    (self.ui.enemy_inspect_progress - step).max(enemy_inspect_target)
                };
            self.refresh_hover();
            changed = true;
        } else {
            self.ui.enemy_inspect_progress = enemy_inspect_target;
        }
        if !self.ui.enemy_inspect_open && !self.enemy_inspect_visible() {
            self.ui.enemy_inspect_enemy = None;
        }
        let restart_target = if self.ui.restart_confirm_open {
            1.0
        } else {
            0.0
        };
        if (self.ui.restart_confirm_progress - restart_target).abs() > 0.001 {
            let step = (dt_ms / BOOT_MODAL_TRANSITION_MS).clamp(0.0, 1.0);
            self.ui.restart_confirm_progress = if self.ui.restart_confirm_progress < restart_target
            {
                (self.ui.restart_confirm_progress + step).min(restart_target)
            } else {
                (self.ui.restart_confirm_progress - step).max(restart_target)
            };
            self.refresh_hover();
            changed = true;
        } else {
            self.ui.restart_confirm_progress = restart_target;
        }
        let settings_target = if self.ui.settings_open { 1.0 } else { 0.0 };
        if (self.ui.settings_progress - settings_target).abs() > 0.001 {
            let step = (dt_ms / BOOT_MODAL_TRANSITION_MS).clamp(0.0, 1.0);
            self.ui.settings_progress = if self.ui.settings_progress < settings_target {
                (self.ui.settings_progress + step).min(settings_target)
            } else {
                (self.ui.settings_progress - step).max(settings_target)
            };
            self.refresh_hover();
            changed = true;
        } else {
            self.ui.settings_progress = settings_target;
        }
        let install_help_target = if self.ui.install_help_open { 1.0 } else { 0.0 };
        if (self.ui.install_help_progress - install_help_target).abs() > 0.001 {
            let step = (dt_ms / BOOT_MODAL_TRANSITION_MS).clamp(0.0, 1.0);
            self.ui.install_help_progress = if self.ui.install_help_progress < install_help_target {
                (self.ui.install_help_progress + step).min(install_help_target)
            } else {
                (self.ui.install_help_progress - step).max(install_help_target)
            };
            self.refresh_hover();
            changed = true;
        } else {
            self.ui.install_help_progress = install_help_target;
        }
        for floater in &mut self.floaters {
            floater.ttl_ms -= dt_ms;
            floater.y -= dt_ms * 0.032;
            changed = true;
        }
        self.floaters.retain(|floater| floater.ttl_ms > 0.0);
        for shard in &mut self.pixel_shards {
            shard.ttl_ms -= dt_ms;
            shard.x += shard.vx * dt_ms;
            shard.y += shard.vy * dt_ms;
            shard.vy += dt_ms * 0.00035;
            changed = true;
        }
        self.pixel_shards.retain(|shard| shard.ttl_ms > 0.0);
        if let Some(transition) = &mut self.layout_transition {
            transition.ttl_ms -= dt_ms;
            changed = true;
        }
        if self
            .layout_transition
            .as_ref()
            .is_some_and(|transition| transition.ttl_ms <= 0.0)
        {
            self.layout_transition = None;
        }
        if self.layout_transition.is_some() {
            self.refresh_hover();
        }
        if let Some(transition) = &mut self.screen_transition {
            transition.ttl_ms -= dt_ms;
            changed = true;
        }
        let mut screen_transition_changed = false;
        if self
            .screen_transition
            .as_ref()
            .is_some_and(|transition| transition.ttl_ms <= 0.0)
        {
            self.screen_transition = None;
            if !matches!(self.screen, AppScreen::Reward) {
                self.reward = None;
            }
            if !matches!(self.screen, AppScreen::Shop) {
                self.shop = None;
            }
            if !matches!(self.screen, AppScreen::Event) {
                self.event = None;
            }
            if !matches!(self.screen, AppScreen::Rest) {
                self.rest = None;
            }
            if !matches!(self.screen, AppScreen::LevelIntro) {
                self.level_intro = None;
            }
            if !matches!(self.screen, AppScreen::OpeningIntro) {
                self.opening_intro = None;
            }
            screen_transition_changed = true;
        }
        if self.screen_transition.is_some() || screen_transition_changed {
            self.refresh_hover();
        }

        if self.final_victory_summary().is_some() {
            self.victory_burst_cooldown_ms -= dt_ms;
            if self.victory_burst_cooldown_ms <= 0.0 {
                self.spawn_random_victory_burst();
                let seed = self
                    .boot_time_ms
                    .to_bits()
                    .wrapping_add(self.pixel_shards.len() as u32 * 17)
                    .wrapping_add(self.restart_count as u32 * 29);
                self.victory_burst_cooldown_ms = 150.0 + noise01(seed) * 210.0;
                changed = true;
            }
        } else {
            self.victory_burst_cooldown_ms = 0.0;
        }

        changed |= self.tick_combat_feedback(dt_ms);
        self.refresh_combat_layout_transition();
        if combat_locked_before != self.combat_input_locked() {
            self.refresh_hover();
        }

        if changed {
            self.dirty = true;
        }

        if self.dirty {
            self.rebuild_frame();
        }
    }

    pub(crate) fn pointer_move(&mut self, x: f32, y: f32) {
        self.pointer_pos = self.to_logical(x, y);
        self.refresh_hover();
    }

    pub(crate) fn pointer_down(&mut self, x: f32, y: f32) {
        if self.screen_transition.is_some() {
            return;
        }

        let Some((lx, ly)) = self.to_logical(x, y) else {
            if self.ui.selected_card.is_some() {
                self.snapshot_combat_layout_target();
            }
            if self.ui.selected_card.take().is_some() {
                self.refresh_hover();
                self.dirty = true;
            }
            return;
        };

        match self.screen {
            AppScreen::Boot => self.handle_boot_pointer(lx, ly),
            AppScreen::OpeningIntro => self.handle_opening_intro_pointer(lx, ly),
            AppScreen::Map => self.handle_map_pointer(lx, ly),
            AppScreen::ModuleSelect => self.handle_module_select_pointer(lx, ly),
            AppScreen::LevelIntro => self.handle_level_intro_pointer(lx, ly),
            AppScreen::Rest => self.handle_rest_pointer(lx, ly),
            AppScreen::Shop => self.handle_shop_pointer(lx, ly),
            AppScreen::Event => self.handle_event_pointer(lx, ly),
            AppScreen::Reward => self.handle_reward_pointer(lx, ly),
            AppScreen::Combat => self.handle_combat_pointer(lx, ly),
            AppScreen::Result(_) => match self.hit_test(lx, ly) {
                Some(HitTarget::Share) => self.queue_share_request(),
                Some(HitTarget::Restart) => self.return_to_menu(),
                _ => {}
            },
        }
    }

    pub(crate) fn pointer_up(&mut self, _x: f32, _y: f32) {}

    pub(crate) fn key_down(&mut self, key_code: u32) {
        if self.screen_transition.is_some() {
            return;
        }

        match self.screen {
            AppScreen::Boot => {
                if self.ui.install_help_open || self.install_help_visible() {
                    match key_code {
                        13 | 27 | 32 => self.close_install_help(),
                        _ => {}
                    }
                } else if self.ui.settings_open || self.settings_visible() {
                    match key_code {
                        27 => self.close_settings(),
                        49 => self.set_language_from_boot(Language::English),
                        50 => self.set_language_from_boot(Language::Spanish),
                        _ => {}
                    }
                } else if self.ui.restart_confirm_open || self.restart_confirm_visible() {
                    match key_code {
                        13 | 32 => self.confirm_restart_run(),
                        27 => self.close_restart_confirm(),
                        _ => {}
                    }
                } else {
                    match key_code {
                        13 | 32 => self.activate_boot_primary_action(),
                        83 | 115 => self.open_settings(),
                        73 | 105 => self.activate_boot_install_action(),
                        85 | 117 => self.request_update(),
                        _ => {}
                    }
                }
            }
            AppScreen::OpeningIntro => {
                if matches!(key_code, 13 | 27 | 32) {
                    self.handle_opening_intro_action();
                }
            }
            AppScreen::Map => match key_code {
                27 => {
                    if self.run_info_visible() || self.ui.run_info_open {
                        self.close_run_info();
                    } else if self.legend_visible() || self.ui.legend_open {
                        self.ui.legend_open = false;
                        self.refresh_hover();
                        self.dirty = true;
                    } else {
                        self.return_to_menu();
                    }
                }
                49..=57 => {
                    if self.legend_visible()
                        || self.ui.legend_open
                        || self.run_info_visible()
                        || self.ui.run_info_open
                    {
                        return;
                    }
                    let index = (key_code - 49) as usize;
                    if let Some(node_id) = self
                        .dungeon
                        .as_ref()
                        .and_then(|dungeon| dungeon.available_nodes.get(index).copied())
                    {
                        self.select_map_node(node_id);
                    }
                }
                _ => {}
            },
            AppScreen::ModuleSelect => {
                if let 49..=57 = key_code {
                    let index = (key_code - 49) as usize;
                    if self
                        .module_select
                        .as_ref()
                        .is_some_and(|module_select| index < module_select.options.len())
                    {
                        self.claim_module_select(index);
                    }
                }
            }
            AppScreen::LevelIntro => {
                if matches!(key_code, 13 | 32 | 27) {
                    self.continue_from_level_intro();
                }
            }
            AppScreen::Rest => match key_code {
                13 | 32 => self.confirm_rest_selection(),
                27 if self.ui.rest_selection.take().is_some() => {
                    self.dirty = true;
                }
                37 => self.set_rest_page(self.ui.rest_page.saturating_sub(1)),
                39 => self.set_rest_page(self.ui.rest_page.saturating_add(1)),
                49..=57 => {
                    let Some(layout) = self.rest_layout() else {
                        return;
                    };
                    let index = (key_code - 49) as usize;
                    if self.rest_heal_actionable() {
                        if index == 0 {
                            self.select_rest_option(RestSelection::Heal);
                        } else if let Some(&option_index) =
                            layout.visible_upgrade_indices.get(index - 1)
                        {
                            self.select_rest_option(RestSelection::Upgrade(option_index));
                        }
                    } else if let Some(&option_index) = layout.visible_upgrade_indices.get(index) {
                        self.select_rest_option(RestSelection::Upgrade(option_index));
                    }
                }
                _ => {}
            },
            AppScreen::Shop => match key_code {
                27 | 48 => self.leave_shop(),
                49..=57 => {
                    let index = (key_code - 49) as usize;
                    if self
                        .shop
                        .as_ref()
                        .is_some_and(|shop| index < shop.offers.len())
                    {
                        self.claim_shop_offer(index);
                    }
                }
                _ => {}
            },
            AppScreen::Event => {
                if let 49..=57 = key_code {
                    let index = (key_code - 49) as usize;
                    if index < 2 {
                        self.claim_event_choice(index);
                    }
                }
            }
            AppScreen::Reward => match key_code {
                48 => self.skip_reward(),
                49..=57 => {
                    let index = (key_code - 49) as usize;
                    if self
                        .reward
                        .as_ref()
                        .is_some_and(|reward| index < reward.options.len())
                    {
                        self.claim_reward(index);
                    }
                }
                _ => {}
            },
            AppScreen::Combat => match key_code {
                27 if self.ui.run_info_open => self.close_run_info(),
                27 if self.ui.enemy_inspect_open => self.close_enemy_inspect(),
                27 if self.run_info_visible() => self.close_run_info(),
                27 if self.enemy_inspect_visible() => self.close_enemy_inspect(),
                _ if self.ui.run_info_open
                    || self.run_info_visible()
                    || self.ui.enemy_inspect_open
                    || self.enemy_inspect_visible() => {}
                _ if self.combat_input_locked() => {}
                13 | 32 => self.perform_action(CombatAction::EndTurn),
                27 if self.ui.selected_card.take().is_some() => {
                    self.push_log("Selection cleared.");
                    self.dirty = true;
                }
                49..=57 => {
                    let index = (key_code - 49) as usize;
                    if let Some(selected) = self.ui.selected_card {
                        if self.combat.card_requires_enemy(selected) {
                            if self.combat.enemy_is_alive(index) {
                                self.perform_action(CombatAction::PlayCard {
                                    hand_index: selected,
                                    target: Some(Actor::Enemy(index)),
                                });
                            }
                        } else if index < self.combat.hand_len() {
                            self.select_or_play_card(index);
                        }
                    } else if index < self.combat.hand_len() {
                        self.select_or_play_card(index);
                    }
                }
                _ => {}
            },
            AppScreen::Result(_) => {
                if matches!(key_code, 13 | 27 | 32) {
                    self.return_to_menu();
                }
            }
        }
    }

    pub(crate) fn frame_ptr(&self) -> *const u8 {
        self.frame.as_ptr()
    }

    pub(crate) fn frame_len(&self) -> usize {
        self.frame.len()
    }

    pub(crate) fn mix_entropy(&mut self, low: u32, high: u32) {
        let incoming = ((high as u64) << 32) | low as u64;
        self.seed_entropy =
            scramble_seed(self.seed_entropy ^ incoming ^ self.boot_time_ms.to_bits() as u64);
    }

    pub(crate) fn set_debug_mode(&mut self, enabled: bool) {
        if self.debug_mode != enabled {
            self.debug_mode = enabled;
            self.refresh_hover();
            self.dirty = true;
        }
    }

    pub(crate) fn set_saved_run_available(&mut self, available: bool) {
        if self.has_saved_run != available {
            self.has_saved_run = available;
            self.refresh_hover();
            self.dirty = true;
            if self.dirty {
                self.rebuild_frame();
            }
        }
    }

    pub(crate) fn run_save_generation(&self) -> u32 {
        self.run_save_generation
    }

    pub(crate) fn run_save_ptr(&self) -> *const u8 {
        self.run_save_snapshot
            .as_ref()
            .map(|snapshot| snapshot.as_ptr())
            .unwrap_or(std::ptr::null())
    }

    pub(crate) fn run_save_len(&self) -> usize {
        self.run_save_snapshot
            .as_ref()
            .map(|snapshot| snapshot.len())
            .unwrap_or(0)
    }

    pub(crate) fn resume_request_pending(&self) -> bool {
        self.resume_request_pending
    }

    pub(crate) fn clear_resume_request(&mut self) {
        self.resume_request_pending = false;
    }

    fn activate_boot_install_action(&mut self) {
        match self.install_capability {
            InstallCapability::PromptAvailable => self.request_install(),
            InstallCapability::IosGuide => self.open_install_help(),
            InstallCapability::Unavailable | InstallCapability::Installed => {}
        }
    }

    pub(crate) fn prepare_restore_buffer(&mut self, len: usize) -> *mut u8 {
        self.restore_buffer.clear();
        self.restore_buffer.resize(len, 0);
        self.restore_buffer.as_mut_ptr()
    }

    pub(crate) fn restore_from_buffer(&mut self, len: usize) -> bool {
        if len > self.restore_buffer.len() {
            self.resume_request_pending = false;
            self.clear_run_save_snapshot();
            return false;
        }

        let raw = match std::str::from_utf8(&self.restore_buffer[..len]) {
            Ok(raw) => raw.to_owned(),
            Err(_) => {
                self.resume_request_pending = false;
                self.clear_run_save_snapshot();
                return false;
            }
        };

        let restored = self.restore_from_save_raw(&raw).is_ok();
        self.resume_request_pending = false;
        if !restored {
            self.clear_run_save_snapshot();
        }
        if self.dirty {
            self.rebuild_frame();
        }
        restored
    }

    fn activate_boot_primary_action(&mut self) {
        if self.has_saved_run {
            self.request_resume();
        } else {
            self.start_run();
        }
    }

    fn clear_boot_request_flags(&mut self) {
        self.resume_request_pending = false;
        self.install_request_pending = false;
        self.update_request_pending = false;
    }

    fn request_resume(&mut self) {
        if !self.has_saved_run {
            return;
        }

        self.ui.restart_confirm_open = false;
        self.ui.settings_open = false;
        self.ui.install_help_open = false;
        self.clear_boot_request_flags();
        self.resume_request_pending = true;
        self.refresh_hover();
        self.dirty = true;
    }

    fn open_restart_confirm(&mut self) {
        if !self.has_saved_run {
            return;
        }

        self.ui.settings_open = false;
        self.ui.install_help_open = false;
        self.ui.restart_confirm_open = true;
        self.clear_boot_request_flags();
        self.refresh_hover();
        self.dirty = true;
    }

    fn close_restart_confirm(&mut self) {
        if !self.ui.restart_confirm_open && !self.restart_confirm_visible() {
            return;
        }

        self.ui.restart_confirm_open = false;
        self.refresh_hover();
        self.dirty = true;
    }

    fn open_settings(&mut self) {
        if !matches!(self.screen, AppScreen::Boot) {
            return;
        }

        self.ui.restart_confirm_open = false;
        self.ui.install_help_open = false;
        self.ui.settings_open = true;
        self.clear_boot_request_flags();
        self.refresh_hover();
        self.dirty = true;
    }

    fn close_settings(&mut self) {
        if !self.ui.settings_open && !self.settings_visible() {
            return;
        }

        self.ui.settings_open = false;
        self.refresh_hover();
        self.dirty = true;
    }

    fn request_install(&mut self) {
        if self.install_capability != InstallCapability::PromptAvailable {
            return;
        }

        self.ui.restart_confirm_open = false;
        self.ui.settings_open = false;
        self.ui.install_help_open = false;
        self.clear_boot_request_flags();
        self.install_request_pending = true;
        self.refresh_hover();
        self.dirty = true;
    }

    fn request_update(&mut self) {
        if !matches!(self.screen, AppScreen::Boot) || !self.update_available {
            return;
        }

        self.ui.restart_confirm_open = false;
        self.ui.settings_open = false;
        self.ui.install_help_open = false;
        self.clear_boot_request_flags();
        self.update_request_pending = true;
        self.refresh_hover();
        self.dirty = true;
    }

    fn open_install_help(&mut self) {
        if !matches!(self.screen, AppScreen::Boot)
            || self.install_capability != InstallCapability::IosGuide
        {
            return;
        }

        self.ui.restart_confirm_open = false;
        self.ui.settings_open = false;
        self.ui.install_help_open = true;
        self.clear_boot_request_flags();
        self.refresh_hover();
        self.dirty = true;
    }

    fn close_install_help(&mut self) {
        if !self.ui.install_help_open && !self.install_help_visible() {
            return;
        }

        self.ui.install_help_open = false;
        self.refresh_hover();
        self.dirty = true;
    }

    fn set_language_from_boot(&mut self, language: Language) {
        self.set_language(language);
        self.ui.settings_open = false;
        self.refresh_hover();
        self.dirty = true;
    }

    fn open_run_info(&mut self) {
        if self.dungeon.is_none() {
            return;
        }

        self.ui.run_info_open = true;
        self.ui.enemy_inspect_open = false;
        self.ui.legend_open = false;
        self.refresh_hover();
        self.dirty = true;
    }

    fn close_run_info(&mut self) {
        if !self.ui.run_info_open && !self.run_info_visible() {
            return;
        }

        self.ui.run_info_open = false;
        self.refresh_hover();
        self.dirty = true;
    }

    fn toggle_run_info(&mut self) {
        if self.ui.run_info_open || self.run_info_visible() {
            self.close_run_info();
        } else {
            self.open_run_info();
        }
    }

    fn open_enemy_inspect(&mut self, enemy_index: usize) {
        if !matches!(self.screen, AppScreen::Combat) || !self.combat.enemy_is_alive(enemy_index) {
            return;
        }

        self.ui.enemy_inspect_enemy = Some(enemy_index);
        self.ui.enemy_inspect_open = true;
        self.ui.run_info_open = false;
        self.refresh_hover();
        self.dirty = true;
    }

    fn close_enemy_inspect(&mut self) {
        if !self.ui.enemy_inspect_open && !self.enemy_inspect_visible() {
            return;
        }

        self.ui.enemy_inspect_open = false;
        self.refresh_hover();
        self.dirty = true;
    }

    fn toggle_or_switch_enemy_inspect(&mut self, enemy_index: usize) {
        if self.ui.enemy_inspect_open && self.ui.enemy_inspect_enemy == Some(enemy_index) {
            self.close_enemy_inspect();
        } else {
            self.open_enemy_inspect(enemy_index);
        }
    }

    fn confirm_restart_run(&mut self) {
        self.ui.restart_confirm_open = false;
        self.ui.settings_open = false;
        self.ui.install_help_open = false;
        self.clear_boot_request_flags();
        self.clear_run_save_snapshot();
        self.dirty = true;
    }

    fn debug_clear_saved_run(&mut self) {
        if !self.debug_mode || !self.has_saved_run {
            return;
        }

        self.ui.restart_confirm_open = false;
        self.ui.settings_open = false;
        self.ui.install_help_open = false;
        self.clear_boot_request_flags();
        self.clear_run_save_snapshot();
        self.refresh_hover();
        self.dirty = true;
    }

    fn handle_boot_pointer(&mut self, x: f32, y: f32) {
        let Some(target) = self.hit_test(x, y) else {
            if self.ui.install_help_open || self.install_help_visible() {
                self.close_install_help();
                return;
            }
            if self.ui.settings_open || self.settings_visible() {
                self.close_settings();
                return;
            }
            if self.ui.restart_confirm_open || self.restart_confirm_visible() {
                self.close_restart_confirm();
            }
            return;
        };

        match target {
            HitTarget::Start => self.start_run(),
            HitTarget::Continue => self.request_resume(),
            HitTarget::Settings => self.open_settings(),
            HitTarget::Install => self.activate_boot_install_action(),
            HitTarget::Update => self.request_update(),
            HitTarget::SettingsLanguageEnglish => self.set_language_from_boot(Language::English),
            HitTarget::SettingsLanguageSpanish => self.set_language_from_boot(Language::Spanish),
            HitTarget::InstallHelpClose => self.close_install_help(),
            HitTarget::Restart => self.open_restart_confirm(),
            HitTarget::RestartConfirm => self.confirm_restart_run(),
            HitTarget::RestartCancel => self.close_restart_confirm(),
            HitTarget::DebugClearSave => self.debug_clear_saved_run(),
            HitTarget::RestartModal | HitTarget::SettingsModal | HitTarget::InstallHelpModal => {}
            _ => {}
        }
    }

    fn handle_module_select_pointer(&mut self, x: f32, y: f32) {
        let Some(target) = self.hit_test(x, y) else {
            return;
        };

        if let HitTarget::ModuleSelectCard(index) = target {
            self.claim_module_select(index);
        }
    }

    fn normalize_saved_log(log: &[String]) -> VecDeque<String> {
        let keep = log.len().saturating_sub(7);
        log.iter().skip(keep).cloned().collect()
    }

    fn set_run_save_snapshot(&mut self, snapshot: Option<String>) {
        let next_has_saved_run = snapshot.is_some();
        if self.run_save_snapshot == snapshot && self.has_saved_run == next_has_saved_run {
            return;
        }

        self.run_save_snapshot = snapshot;
        self.has_saved_run = next_has_saved_run;
        self.run_save_generation = self.run_save_generation.wrapping_add(1);
        self.refresh_hover();
        self.dirty = true;
    }

    fn clear_run_save_snapshot(&mut self) {
        self.set_run_save_snapshot(None);
    }

    fn refresh_run_save_snapshot(&mut self) {
        match self.screen {
            AppScreen::Boot => {}
            AppScreen::OpeningIntro => self.clear_run_save_snapshot(),
            AppScreen::Result(_) => self.clear_run_save_snapshot(),
            AppScreen::Map
            | AppScreen::ModuleSelect
            | AppScreen::LevelIntro
            | AppScreen::Rest
            | AppScreen::Shop
            | AppScreen::Event
            | AppScreen::Combat
            | AppScreen::Reward => {
                if let Some(snapshot) = self.serialize_current_run() {
                    self.set_run_save_snapshot(Some(snapshot));
                }
            }
        }
    }

    fn serialize_current_run(&self) -> Option<String> {
        let envelope = self.build_run_save_envelope()?;
        serialize_envelope(&envelope).ok()
    }

    fn build_run_save_envelope(&self) -> Option<RunSaveEnvelope> {
        let active_state = self.build_active_run_state()?;
        let fallback_checkpoint = self.build_fallback_checkpoint(&active_state)?;
        let log = self.log.iter().cloned().collect();
        Some(RunSaveEnvelope::new(active_state, fallback_checkpoint, log))
    }

    fn build_active_run_state(&self) -> Option<SavedRunState> {
        let dungeon = self.save_dungeon_run(self.dungeon.as_ref()?);
        match self.screen {
            AppScreen::Map => Some(SavedRunState::Map { dungeon }),
            AppScreen::ModuleSelect => Some(SavedRunState::ModuleSelect {
                dungeon,
                module_select: self.save_module_select_state(self.module_select.as_ref()?),
            }),
            AppScreen::LevelIntro => Some(SavedRunState::LevelIntro { dungeon }),
            AppScreen::Rest => Some(SavedRunState::Rest { dungeon }),
            AppScreen::Shop => Some(SavedRunState::Shop {
                dungeon,
                shop: self.save_shop_state(self.shop.as_ref()?),
            }),
            AppScreen::Event => Some(SavedRunState::Event {
                dungeon,
                event: self.save_event_state(self.event.as_ref()?),
            }),
            AppScreen::Reward => Some(SavedRunState::Reward {
                dungeon,
                reward: self.save_reward_state(self.reward.as_ref()?),
            }),
            AppScreen::Combat => Some(SavedRunState::Combat {
                dungeon,
                combat: self.save_combat_state(),
            }),
            AppScreen::Boot | AppScreen::OpeningIntro | AppScreen::Result(_) => None,
        }
    }

    fn build_fallback_checkpoint(&self, active_state: &SavedRunState) -> Option<SavedCheckpoint> {
        match active_state {
            SavedRunState::Map { dungeon } => Some(SavedCheckpoint::Map {
                dungeon: dungeon.clone(),
            }),
            SavedRunState::ModuleSelect {
                dungeon,
                module_select,
            } => Some(SavedCheckpoint::ModuleSelect {
                dungeon: dungeon.clone(),
                module_select: module_select.clone(),
            }),
            SavedRunState::LevelIntro { dungeon } => Some(SavedCheckpoint::LevelIntro {
                dungeon: dungeon.clone(),
            }),
            SavedRunState::Rest { dungeon } => Some(SavedCheckpoint::Rest {
                dungeon: dungeon.clone(),
            }),
            SavedRunState::Shop { dungeon, shop } => Some(SavedCheckpoint::Shop {
                dungeon: dungeon.clone(),
                shop: shop.clone(),
            }),
            SavedRunState::Event { dungeon, event } => Some(SavedCheckpoint::Event {
                dungeon: dungeon.clone(),
                event: event.clone(),
            }),
            SavedRunState::Reward { dungeon, reward } => Some(SavedCheckpoint::Reward {
                dungeon: dungeon.clone(),
                reward: reward.clone(),
            }),
            SavedRunState::Combat { dungeon, .. } => {
                let encounter_setup = self
                    .dungeon
                    .as_ref()
                    .and_then(DungeonRun::current_encounter_setup)
                    .map(save_encounter_setup)?;
                let source_deck = self
                    .dungeon
                    .as_ref()?
                    .deck
                    .iter()
                    .copied()
                    .map(serialize_card_id)
                    .map(str::to_string)
                    .collect();
                Some(SavedCheckpoint::EncounterStart {
                    dungeon: dungeon.clone(),
                    encounter_setup,
                    source_deck,
                })
            }
        }
    }

    fn save_dungeon_run(&self, dungeon: &DungeonRun) -> SavedDungeonRun {
        SavedDungeonRun {
            seed: dungeon.seed,
            current_level: dungeon.current_level,
            nodes: dungeon
                .nodes
                .iter()
                .map(|node| SavedDungeonNode {
                    id: node.id,
                    depth: node.depth,
                    lane: node.lane,
                    kind: serialize_room_kind(node.kind).to_string(),
                    next: node.next.clone(),
                })
                .collect(),
            current_node: dungeon.current_node,
            available_nodes: dungeon.available_nodes.clone(),
            visited_nodes: dungeon.visited_nodes.clone(),
            deck: dungeon
                .deck
                .iter()
                .copied()
                .map(serialize_card_id)
                .map(str::to_string)
                .collect(),
            modules: Some(
                dungeon
                    .modules
                    .iter()
                    .copied()
                    .map(serialize_module_id)
                    .map(str::to_string)
                    .collect(),
            ),
            player_hp: dungeon.player_hp,
            player_max_hp: dungeon.player_max_hp,
            credits: dungeon.credits,
            combats_cleared: dungeon.combats_cleared,
            elites_cleared: dungeon.elites_cleared,
            rests_completed: dungeon.rests_completed,
            bosses_cleared: dungeon.bosses_cleared,
        }
    }

    fn save_module_select_state(
        &self,
        module_select: &ModuleSelectState,
    ) -> SavedModuleSelectState {
        let (kind, boss_level) = match module_select.context {
            ModuleSelectContext::Starter => ("starter".to_string(), None),
            ModuleSelectContext::BossReward { boss_level } => {
                ("boss_reward".to_string(), Some(boss_level))
            }
        };
        SavedModuleSelectState {
            options: module_select
                .options
                .iter()
                .copied()
                .map(serialize_module_id)
                .map(str::to_string)
                .collect(),
            seed: module_select.seed,
            kind: Some(kind),
            boss_level,
        }
    }

    fn save_reward_state(&self, reward: &RewardState) -> SavedRewardState {
        SavedRewardState {
            tier: serialize_reward_tier(reward.tier).to_string(),
            options: reward
                .options
                .iter()
                .copied()
                .map(serialize_card_id)
                .map(str::to_string)
                .collect(),
            followup_completed_run: reward.followup.completed_run,
            seed: reward.seed,
        }
    }

    fn save_event_state(&self, event: &EventState) -> SavedEventState {
        SavedEventState {
            event: serialize_event_id(event.event).to_string(),
        }
    }

    fn save_shop_state(&self, shop: &ShopState) -> SavedShopState {
        SavedShopState {
            offers: shop
                .offers
                .iter()
                .map(|offer| SavedShopOffer {
                    card: serialize_card_id(offer.card).to_string(),
                    price: offer.price,
                })
                .collect(),
            seed: shop.seed,
        }
    }

    fn save_combat_state(&self) -> SavedCombatState {
        SavedCombatState {
            player: SavedPlayerState {
                fighter: SavedFighterState {
                    hp: self.combat.player.fighter.hp,
                    max_hp: self.combat.player.fighter.max_hp,
                    block: self.combat.player.fighter.block,
                    bleed: self.combat.player.fighter.statuses.bleed,
                    focus: self.combat.player.fighter.statuses.focus,
                    rhythm: self.combat.player.fighter.statuses.rhythm,
                    momentum: self.combat.player.fighter.statuses.momentum,
                },
                energy: self.combat.player.energy,
                max_energy: self.combat.player.max_energy,
            },
            enemies: self
                .combat
                .enemies
                .iter()
                .map(|enemy| SavedEnemyState {
                    fighter: SavedFighterState {
                        hp: enemy.fighter.hp,
                        max_hp: enemy.fighter.max_hp,
                        block: enemy.fighter.block,
                        bleed: enemy.fighter.statuses.bleed,
                        focus: enemy.fighter.statuses.focus,
                        rhythm: enemy.fighter.statuses.rhythm,
                        momentum: enemy.fighter.statuses.momentum,
                    },
                    profile: serialize_enemy_profile(enemy.profile).to_string(),
                    intent_index: enemy.intent_index,
                    on_hit_bleed: enemy.on_hit_bleed,
                })
                .collect(),
            deck: SavedDeckState {
                draw_pile: self
                    .combat
                    .deck
                    .draw_pile
                    .iter()
                    .copied()
                    .map(serialize_card_id)
                    .map(str::to_string)
                    .collect(),
                hand: self
                    .combat
                    .deck
                    .hand
                    .iter()
                    .copied()
                    .map(serialize_card_id)
                    .map(str::to_string)
                    .collect(),
                discard_pile: self
                    .combat
                    .deck
                    .discard_pile
                    .iter()
                    .copied()
                    .map(serialize_card_id)
                    .map(str::to_string)
                    .collect(),
            },
            phase: serialize_turn_phase(self.combat.phase).to_string(),
            turn: self.combat.turn,
            rng_state: self.combat.rng_state(),
        }
    }

    fn restore_from_save_raw(&mut self, raw: &str) -> Result<(), String> {
        let envelope = parse_run_save(raw)?;
        let restore_result = self
            .restore_active_state(&envelope.active_state, &envelope.log)
            .or_else(|_| self.restore_fallback_checkpoint(&envelope.fallback_checkpoint));

        match restore_result {
            Ok(restored_exact) => {
                if !restored_exact {
                    self.log.clear();
                    self.push_log("Run resumed from checkpoint.");
                }
                self.resume_request_pending = false;
                self.refresh_run_save_snapshot();
                self.dirty = true;
                Ok(())
            }
            Err(error) => {
                self.clear_run_save_snapshot();
                Err(error)
            }
        }
    }

    fn restore_active_state(
        &mut self,
        active_state: &SavedRunState,
        saved_log: &[String],
    ) -> Result<bool, String> {
        match active_state {
            SavedRunState::Map { dungeon } => {
                let dungeon = self.restore_saved_dungeon(dungeon)?;
                self.apply_restored_state(
                    AppScreen::Map,
                    dungeon,
                    RestoredStatePayload {
                        log: Self::normalize_saved_log(saved_log),
                        ..RestoredStatePayload::default()
                    },
                )?;
            }
            SavedRunState::ModuleSelect {
                dungeon,
                module_select,
            } => {
                let dungeon = self.restore_saved_dungeon(dungeon)?;
                let module_select = self.restore_saved_module_select_exact(module_select)?;
                self.apply_restored_state(
                    AppScreen::ModuleSelect,
                    dungeon,
                    RestoredStatePayload {
                        module_select: Some(module_select),
                        log: Self::normalize_saved_log(saved_log),
                        ..RestoredStatePayload::default()
                    },
                )?;
            }
            SavedRunState::LevelIntro { dungeon } => {
                let dungeon = self.restore_saved_dungeon(dungeon)?;
                self.apply_restored_state(
                    AppScreen::LevelIntro,
                    dungeon,
                    RestoredStatePayload {
                        log: Self::normalize_saved_log(saved_log),
                        ..RestoredStatePayload::default()
                    },
                )?;
            }
            SavedRunState::Rest { dungeon } => {
                let dungeon = self.restore_saved_dungeon(dungeon)?;
                self.apply_restored_state(
                    AppScreen::Rest,
                    dungeon,
                    RestoredStatePayload {
                        log: Self::normalize_saved_log(saved_log),
                        ..RestoredStatePayload::default()
                    },
                )?;
            }
            SavedRunState::Shop { dungeon, shop } => {
                let dungeon = self.restore_saved_dungeon(dungeon)?;
                let shop = self.restore_saved_shop_exact(shop)?;
                self.apply_restored_state(
                    AppScreen::Shop,
                    dungeon,
                    RestoredStatePayload {
                        shop: Some(shop),
                        log: Self::normalize_saved_log(saved_log),
                        ..RestoredStatePayload::default()
                    },
                )?;
            }
            SavedRunState::Event { dungeon, event } => {
                let dungeon = self.restore_saved_dungeon(dungeon)?;
                let event = self.restore_saved_event_exact(event)?;
                self.apply_restored_state(
                    AppScreen::Event,
                    dungeon,
                    RestoredStatePayload {
                        event: Some(event),
                        log: Self::normalize_saved_log(saved_log),
                        ..RestoredStatePayload::default()
                    },
                )?;
            }
            SavedRunState::Reward { dungeon, reward } => {
                let dungeon = self.restore_saved_dungeon(dungeon)?;
                let reward = self.restore_saved_reward_exact(reward)?;
                self.apply_restored_state(
                    AppScreen::Reward,
                    dungeon,
                    RestoredStatePayload {
                        reward: Some(reward),
                        log: Self::normalize_saved_log(saved_log),
                        ..RestoredStatePayload::default()
                    },
                )?;
            }
            SavedRunState::Combat { dungeon, combat } => {
                let dungeon = self.restore_saved_dungeon(dungeon)?;
                let combat = self.restore_saved_combat_exact(combat)?;
                self.apply_restored_state(
                    AppScreen::Combat,
                    dungeon,
                    RestoredStatePayload {
                        combat: Some(combat),
                        log: Self::normalize_saved_log(saved_log),
                        ..RestoredStatePayload::default()
                    },
                )?;
            }
        }

        Ok(true)
    }

    fn restore_fallback_checkpoint(
        &mut self,
        checkpoint: &SavedCheckpoint,
    ) -> Result<bool, String> {
        match checkpoint {
            SavedCheckpoint::Map { dungeon } => {
                let dungeon = self.restore_saved_dungeon(dungeon)?;
                self.apply_restored_state(
                    AppScreen::Map,
                    dungeon,
                    RestoredStatePayload::default(),
                )?;
            }
            SavedCheckpoint::ModuleSelect {
                dungeon,
                module_select,
            } => {
                let dungeon = self.restore_saved_dungeon(dungeon)?;
                let module_select = self.restore_saved_module_select_fallback(module_select)?;
                self.apply_restored_state(
                    AppScreen::ModuleSelect,
                    dungeon,
                    RestoredStatePayload {
                        module_select: Some(module_select),
                        ..RestoredStatePayload::default()
                    },
                )?;
            }
            SavedCheckpoint::LevelIntro { dungeon } => {
                let dungeon = self.restore_saved_dungeon(dungeon)?;
                self.apply_restored_state(
                    AppScreen::LevelIntro,
                    dungeon,
                    RestoredStatePayload::default(),
                )?;
            }
            SavedCheckpoint::Rest { dungeon } => {
                let dungeon = self.restore_saved_dungeon(dungeon)?;
                self.apply_restored_state(
                    AppScreen::Rest,
                    dungeon,
                    RestoredStatePayload::default(),
                )?;
            }
            SavedCheckpoint::Shop { dungeon, shop } => {
                let dungeon = self.restore_saved_dungeon(dungeon)?;
                let shop = self.restore_saved_shop_fallback(shop, dungeon.current_level())?;
                self.apply_restored_state(
                    AppScreen::Shop,
                    dungeon,
                    RestoredStatePayload {
                        shop: Some(shop),
                        ..RestoredStatePayload::default()
                    },
                )?;
            }
            SavedCheckpoint::Event { dungeon, event } => {
                let dungeon = self.restore_saved_dungeon(dungeon)?;
                let event = self.restore_saved_event_fallback(event)?;
                self.apply_restored_state(
                    AppScreen::Event,
                    dungeon,
                    RestoredStatePayload {
                        event: Some(event),
                        ..RestoredStatePayload::default()
                    },
                )?;
            }
            SavedCheckpoint::Reward { dungeon, reward } => {
                let dungeon = self.restore_saved_dungeon(dungeon)?;
                let reward = self.restore_saved_reward_fallback(reward, dungeon.current_level())?;
                self.apply_restored_state(
                    AppScreen::Reward,
                    dungeon,
                    RestoredStatePayload {
                        reward: Some(reward),
                        ..RestoredStatePayload::default()
                    },
                )?;
            }
            SavedCheckpoint::EncounterStart {
                dungeon,
                encounter_setup,
                source_deck,
            } => {
                let dungeon = self.restore_saved_dungeon(dungeon)?;
                let source_deck: Vec<CardId> = source_deck
                    .iter()
                    .map(|card| resolve_deck_card_id(card))
                    .collect();
                let encounter_setup = resolve_encounter_setup(encounter_setup)
                    .or_else(|| dungeon.current_encounter_setup())
                    .ok_or_else(|| "Missing encounter setup for checkpoint restore.".to_string())?;
                let (combat, events) = CombatState::new_with_deck(
                    self.combat_seed_for_dungeon(&dungeon),
                    encounter_setup,
                    if source_deck.is_empty() {
                        dungeon.deck.clone()
                    } else {
                        source_deck
                    },
                );
                self.apply_restored_state(
                    AppScreen::Combat,
                    dungeon,
                    RestoredStatePayload {
                        combat: Some(combat),
                        ..RestoredStatePayload::default()
                    },
                )?;
                self.handle_events(&events);
                self.apply_start_of_combat_modules();
            }
        }

        Ok(false)
    }

    fn restore_saved_dungeon(&self, saved: &SavedDungeonRun) -> Result<DungeonRun, String> {
        let nodes: Vec<_> = saved
            .nodes
            .iter()
            .map(|node| {
                Ok(crate::dungeon::DungeonNode {
                    id: node.id,
                    depth: node.depth,
                    lane: node.lane,
                    kind: resolve_room_kind(&node.kind)
                        .ok_or_else(|| format!("Unknown room kind {}.", node.kind))?,
                    next: node.next.clone(),
                })
            })
            .collect::<Result<_, String>>()?;

        let dungeon = DungeonRun {
            seed: saved.seed,
            current_level: saved.current_level,
            nodes,
            current_node: saved.current_node,
            available_nodes: saved.available_nodes.clone(),
            visited_nodes: saved.visited_nodes.clone(),
            deck: saved
                .deck
                .iter()
                .map(|card| resolve_deck_card_id(card))
                .collect(),
            modules: saved
                .modules
                .as_ref()
                .map(|modules| {
                    modules
                        .iter()
                        .filter_map(|module| resolve_module_id(module))
                        .collect()
                })
                .unwrap_or_else(|| vec![default_starter_module()]),
            player_hp: saved.player_hp,
            player_max_hp: saved.player_max_hp,
            credits: saved.credits,
            combats_cleared: saved.combats_cleared,
            elites_cleared: saved.elites_cleared,
            rests_completed: saved.rests_completed,
            bosses_cleared: saved.bosses_cleared,
        };

        if dungeon.is_structurally_valid() {
            Ok(dungeon)
        } else {
            Err("Saved dungeon structure is invalid.".to_string())
        }
    }

    fn restore_saved_module_select_context_exact(
        &self,
        saved: &SavedModuleSelectState,
    ) -> Result<ModuleSelectContext, String> {
        match saved.kind.as_deref() {
            None | Some("starter") => Ok(ModuleSelectContext::Starter),
            Some("boss_reward") => Ok(ModuleSelectContext::BossReward {
                boss_level: saved
                    .boss_level
                    .ok_or_else(|| "Saved boss module reward is missing boss_level.".to_string())?,
            }),
            Some(other) => Err(format!("Unknown module select kind {}.", other)),
        }
    }

    fn restore_saved_module_select_context_fallback(
        &self,
        saved: &SavedModuleSelectState,
    ) -> ModuleSelectContext {
        match saved.kind.as_deref() {
            Some("boss_reward") => ModuleSelectContext::BossReward {
                boss_level: saved.boss_level.unwrap_or(1).clamp(1, 2),
            },
            _ => ModuleSelectContext::Starter,
        }
    }

    fn module_select_options_for_context(&self, context: ModuleSelectContext) -> Vec<ModuleId> {
        match context {
            ModuleSelectContext::Starter => starter_module_choices(),
            ModuleSelectContext::BossReward { boss_level } => boss_module_choices(boss_level),
        }
    }

    fn restore_saved_module_select_exact(
        &self,
        saved: &SavedModuleSelectState,
    ) -> Result<ModuleSelectState, String> {
        let context = self.restore_saved_module_select_context_exact(saved)?;
        let options: Vec<_> = saved
            .options
            .iter()
            .map(|module| {
                resolve_module_id(module).ok_or_else(|| format!("Unknown module {}.", module))
            })
            .collect::<Result<_, String>>()?;
        if options.is_empty() {
            return Err("Saved module select has no options.".to_string());
        }

        Ok(ModuleSelectState {
            options,
            seed: saved.seed,
            context,
        })
    }

    fn restore_saved_module_select_fallback(
        &self,
        saved: &SavedModuleSelectState,
    ) -> Result<ModuleSelectState, String> {
        let context = self.restore_saved_module_select_context_fallback(saved);
        Ok(ModuleSelectState {
            options: self.module_select_options_for_context(context),
            seed: saved.seed,
            context,
        })
    }

    fn restore_saved_reward_exact(&self, saved: &SavedRewardState) -> Result<RewardState, String> {
        let tier = resolve_reward_tier(&saved.tier)
            .ok_or_else(|| format!("Unknown reward tier {}.", saved.tier))?;
        let options: Vec<_> = saved
            .options
            .iter()
            .map(|card| {
                resolve_card_id(card).ok_or_else(|| format!("Unknown reward card {}.", card))
            })
            .collect::<Result<_, String>>()?;
        if options.is_empty() {
            return Err("Saved reward has no options.".to_string());
        }

        Ok(RewardState {
            tier,
            options,
            followup: RewardFollowup {
                completed_run: saved.followup_completed_run,
            },
            seed: saved.seed,
        })
    }

    fn restore_saved_event_exact(&self, saved: &SavedEventState) -> Result<EventState, String> {
        Ok(EventState {
            event: resolve_event_id(&saved.event)
                .ok_or_else(|| format!("Unknown event id {}.", saved.event))?,
        })
    }

    fn restore_saved_event_fallback(&self, saved: &SavedEventState) -> Result<EventState, String> {
        self.restore_saved_event_exact(saved)
    }

    fn restore_saved_reward_fallback(
        &self,
        saved: &SavedRewardState,
        level: usize,
    ) -> Result<RewardState, String> {
        let tier = resolve_reward_tier(&saved.tier)
            .ok_or_else(|| format!("Unknown reward tier {}.", saved.tier))?;
        Ok(RewardState {
            tier,
            options: reward_choices(saved.seed, tier, level),
            followup: RewardFollowup {
                completed_run: saved.followup_completed_run,
            },
            seed: saved.seed,
        })
    }

    fn restore_saved_shop_exact(&self, saved: &SavedShopState) -> Result<ShopState, String> {
        let offers: Vec<_> = saved
            .offers
            .iter()
            .map(|offer| {
                Ok(ShopOffer {
                    card: resolve_card_id(&offer.card)
                        .ok_or_else(|| format!("Unknown shop card {}.", offer.card))?,
                    price: offer.price,
                })
            })
            .collect::<Result<_, String>>()?;
        if offers.is_empty() {
            return Err("Saved shop has no offers.".to_string());
        }

        Ok(ShopState {
            offers,
            seed: saved.seed,
        })
    }

    fn restore_saved_shop_fallback(
        &self,
        saved: &SavedShopState,
        level: usize,
    ) -> Result<ShopState, String> {
        Ok(ShopState {
            offers: shop_offers(saved.seed, level),
            seed: saved.seed,
        })
    }

    fn restore_saved_combat_exact(&self, saved: &SavedCombatState) -> Result<CombatState, String> {
        let player = PlayerState {
            fighter: FighterState {
                hp: saved.player.fighter.hp,
                max_hp: saved.player.fighter.max_hp,
                block: saved.player.fighter.block,
                statuses: StatusSet {
                    bleed: saved.player.fighter.bleed,
                    focus: saved.player.fighter.focus,
                    rhythm: saved.player.fighter.rhythm,
                    momentum: saved.player.fighter.momentum,
                },
            },
            energy: saved.player.energy,
            max_energy: saved.player.max_energy,
        };
        let enemies = saved
            .enemies
            .iter()
            .map(|enemy| {
                Ok(EnemyState {
                    fighter: FighterState {
                        hp: enemy.fighter.hp,
                        max_hp: enemy.fighter.max_hp,
                        block: enemy.fighter.block,
                        statuses: StatusSet {
                            bleed: enemy.fighter.bleed,
                            focus: enemy.fighter.focus,
                            rhythm: enemy.fighter.rhythm,
                            momentum: enemy.fighter.momentum,
                        },
                    },
                    profile: resolve_enemy_profile(&enemy.profile)
                        .ok_or_else(|| format!("Unknown enemy profile {}.", enemy.profile))?,
                    intent_index: enemy.intent_index,
                    on_hit_bleed: enemy.on_hit_bleed,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let deck = DeckState {
            draw_pile: saved
                .deck
                .draw_pile
                .iter()
                .map(|card| {
                    resolve_card_id(card).ok_or_else(|| format!("Unknown combat card {}.", card))
                })
                .collect::<Result<_, String>>()?,
            hand: saved
                .deck
                .hand
                .iter()
                .map(|card| {
                    resolve_card_id(card).ok_or_else(|| format!("Unknown combat card {}.", card))
                })
                .collect::<Result<_, String>>()?,
            discard_pile: saved
                .deck
                .discard_pile
                .iter()
                .map(|card| {
                    resolve_card_id(card).ok_or_else(|| format!("Unknown combat card {}.", card))
                })
                .collect::<Result<_, String>>()?,
        };
        let phase = resolve_turn_phase(&saved.phase)
            .ok_or_else(|| format!("Unknown combat phase {}.", saved.phase))?;

        Ok(CombatState::from_persisted_parts(
            player,
            enemies,
            deck,
            phase,
            saved.turn,
            saved.rng_state,
        ))
    }

    fn apply_restored_state(
        &mut self,
        screen: AppScreen,
        dungeon: DungeonRun,
        restored: RestoredStatePayload,
    ) -> Result<(), String> {
        let RestoredStatePayload {
            reward,
            shop,
            event,
            module_select,
            combat,
            log,
        } = restored;
        self.screen = screen;
        self.dungeon = Some(dungeon);
        self.reward = if matches!(screen, AppScreen::Reward) {
            Some(reward.ok_or_else(|| "Missing reward state.".to_string())?)
        } else {
            None
        };
        self.module_select = if matches!(screen, AppScreen::ModuleSelect) {
            Some(module_select.ok_or_else(|| "Missing module select state.".to_string())?)
        } else {
            None
        };
        self.shop = if matches!(screen, AppScreen::Shop) {
            Some(shop.ok_or_else(|| "Missing shop state.".to_string())?)
        } else {
            None
        };
        self.event = if matches!(screen, AppScreen::Event) {
            Some(event.ok_or_else(|| "Missing event state.".to_string())?)
        } else {
            None
        };
        self.rest = if matches!(screen, AppScreen::Rest) {
            let dungeon = self
                .dungeon
                .as_ref()
                .ok_or_else(|| "Missing dungeon for rest screen.".to_string())?;
            Some(RestState {
                heal_amount: dungeon.rest_heal_amount(),
                upgrade_options: dungeon.upgradable_card_indices(),
            })
        } else {
            None
        };
        self.level_intro = if matches!(screen, AppScreen::LevelIntro) {
            let dungeon = self
                .dungeon
                .as_ref()
                .ok_or_else(|| "Missing dungeon for level intro.".to_string())?;
            Some(LevelIntroState {
                level: dungeon.current_level(),
                codename: localized_level_codename(dungeon.current_level(), self.language),
                summary: localized_level_summary(dungeon.current_level(), self.language),
            })
        } else {
            None
        };
        self.opening_intro = None;
        if let Some(combat) = combat {
            self.combat = combat;
        }
        self.ui = UiState::default();
        self.pointer_pos = None;
        self.floaters.clear();
        self.pixel_shards.clear();
        self.enemy_vfx_rects.clear();
        self.enemy_defeat_vfx_started = vec![false; self.combat.enemy_count()];
        self.layout_transition = None;
        self.combat_layout_target = None;
        self.screen_transition = None;
        self.share_request = None;
        self.victory_burst_cooldown_ms = 0.0;
        self.log = log;
        self.resume_request_pending = false;
        if matches!(screen, AppScreen::Combat) {
            self.sync_combat_feedback_to_combat();
        }
        self.dirty = true;
        Ok(())
    }

    fn combat_seed_for_dungeon(&self, dungeon: &DungeonRun) -> u64 {
        shared_combat_seed_for_dungeon(dungeon)
    }

    fn start_run(&mut self) {
        let from_screen = self.screen;
        let seed = limit_run_seed(scramble_seed(
            BASE_SEED
                ^ self.seed_entropy
                ^ self.restart_count.wrapping_mul(0x9E37_79B9_7F4A_7C15)
                ^ self.boot_time_ms.to_bits() as u64,
        ));
        self.clear_boot_request_flags();
        self.restart_count = self.restart_count.wrapping_add(1);
        self.seed_entropy = scramble_seed(seed ^ 0x94D0_49BB_1331_11EB);
        self.dungeon = Some(DungeonRun::new(seed));
        self.module_select = Some(ModuleSelectState {
            options: starter_module_choices(),
            seed,
            context: ModuleSelectContext::Starter,
        });
        self.rest = None;
        self.shop = None;
        self.event = None;
        self.reward = None;
        self.level_intro = None;
        self.opening_intro = Some(OpeningIntroState::default());
        self.screen = AppScreen::OpeningIntro;
        self.ui = UiState::default();
        self.pointer_pos = None;
        self.floaters.clear();
        self.pixel_shards.clear();
        self.enemy_vfx_rects.clear();
        self.enemy_defeat_vfx_started.clear();
        self.share_request = None;
        self.victory_burst_cooldown_ms = 0.0;
        self.layout_transition = None;
        self.screen_transition = None;
        self.log.clear();
        self.clear_run_save_snapshot();
        self.begin_screen_transition(from_screen, AppScreen::OpeningIntro);
        self.refresh_run_save_snapshot();
        self.dirty = true;
    }

    fn return_to_menu(&mut self) {
        let from_screen = self.screen;
        self.screen = AppScreen::Boot;
        self.dungeon = None;
        self.rest = None;
        self.shop = None;
        self.event = None;
        self.module_select = None;
        self.reward = None;
        self.level_intro = None;
        self.opening_intro = None;
        self.ui = UiState::default();
        self.pointer_pos = None;
        self.floaters.clear();
        self.pixel_shards.clear();
        self.enemy_vfx_rects.clear();
        self.enemy_defeat_vfx_started.clear();
        self.share_request = None;
        self.victory_burst_cooldown_ms = 0.0;
        self.layout_transition = None;
        self.screen_transition = None;
        self.log.clear();
        self.begin_screen_transition(from_screen, AppScreen::Boot);
        self.refresh_run_save_snapshot();
        self.dirty = true;
    }

    fn begin_encounter(&mut self, setup: EncounterSetup) {
        let from_screen = self.screen;
        let seed = self
            .dungeon
            .as_ref()
            .map(|dungeon| self.combat_seed_for_dungeon(dungeon))
            .unwrap_or(BASE_SEED);
        let deck = self
            .dungeon
            .as_ref()
            .map(|dungeon| dungeon.deck.clone())
            .unwrap_or_else(crate::content::starter_deck);
        let (combat, events) = CombatState::new_with_deck(seed, setup, deck);
        self.combat = combat;
        self.rest = None;
        self.shop = None;
        self.event = None;
        self.module_select = None;
        self.reward = None;
        self.level_intro = None;
        self.opening_intro = None;
        self.screen = AppScreen::Combat;
        self.ui = UiState::default();
        self.pointer_pos = None;
        self.floaters.clear();
        self.pixel_shards.clear();
        self.enemy_vfx_rects.clear();
        self.enemy_defeat_vfx_started = vec![false; self.combat.enemy_count()];
        self.share_request = None;
        self.victory_burst_cooldown_ms = 0.0;
        self.layout_transition = None;
        self.screen_transition = None;
        self.log.clear();
        self.sync_combat_feedback_to_combat();
        self.handle_events(&events);
        self.apply_start_of_combat_modules();
        self.begin_screen_transition(from_screen, AppScreen::Combat);
        self.refresh_run_save_snapshot();
        self.dirty = true;
    }

    fn claim_module_select(&mut self, index: usize) {
        let Some((module, context)) = self.module_select.as_ref().and_then(|module_select| {
            module_select
                .options
                .get(index)
                .copied()
                .map(|module| (module, module_select.context))
        }) else {
            return;
        };
        if let Some(dungeon) = &mut self.dungeon {
            dungeon.add_module(module);
        }
        self.push_log(match self.language {
            Language::English => format!("Selected {}.", self.localized_module_def(module).name),
            Language::Spanish => format!("Elegiste {}.", self.localized_module_def(module).name),
        });

        if matches!(context, ModuleSelectContext::BossReward { .. }) {
            self.module_select = None;
            self.begin_level_intro();
            return;
        }

        let from_screen = self.screen;
        self.module_select = None;
        self.rest = None;
        self.shop = None;
        self.event = None;
        self.reward = None;
        self.level_intro = None;
        self.opening_intro = None;
        self.screen = AppScreen::Map;
        self.ui = UiState::default();
        self.pointer_pos = None;
        self.layout_transition = None;
        self.screen_transition = None;
        self.push_log(self.tr(
            "Route seeded. Select the first room.",
            "Ruta lista. Elige la primera sala.",
        ));
        self.begin_screen_transition(from_screen, AppScreen::Map);
        self.refresh_run_save_snapshot();
        self.dirty = true;
    }

    fn handle_map_pointer(&mut self, x: f32, y: f32) {
        let Some(target) = self.hit_test(x, y) else {
            if self.run_info_visible() || self.ui.run_info_open {
                self.close_run_info();
            } else if self.legend_visible() || self.ui.legend_open {
                self.ui.legend_open = false;
                self.refresh_hover();
                self.dirty = true;
            }
            return;
        };

        match target {
            HitTarget::Menu => self.return_to_menu(),
            HitTarget::Info => self.toggle_run_info(),
            HitTarget::Legend => {
                let open_legend = !(self.ui.legend_open || self.legend_visible());
                self.ui.legend_open = open_legend;
                if open_legend {
                    self.ui.run_info_open = false;
                }
                self.refresh_hover();
                self.dirty = true;
            }
            HitTarget::LegendPanel => {}
            HitTarget::RunInfoPanel => {}
            HitTarget::EnemyInspectPanel => {}
            HitTarget::DebugLevelDown => self.adjust_debug_level(-1),
            HitTarget::DebugLevelUp => self.adjust_debug_level(1),
            HitTarget::DebugFillDeck => self.debug_fill_deck(),
            HitTarget::MapNode(node_id) => self.select_map_node(node_id),
            HitTarget::Start
            | HitTarget::Continue
            | HitTarget::Share
            | HitTarget::Restart
            | HitTarget::RestHeal
            | HitTarget::RestCard(_)
            | HitTarget::RestConfirm
            | HitTarget::RestPagePrev
            | HitTarget::RestPageNext
            | HitTarget::ShopCard(_)
            | HitTarget::ShopLeave
            | HitTarget::EventChoice(_)
            | HitTarget::EndBattle
            | HitTarget::EndTurn
            | HitTarget::Enemy(_)
            | HitTarget::Player
            | HitTarget::RewardCard(_)
            | HitTarget::RewardSkip
            | HitTarget::ModuleSelectCard(_)
            | HitTarget::Card(_)
            | HitTarget::RestartModal
            | HitTarget::RestartConfirm
            | HitTarget::RestartCancel
            | HitTarget::Settings
            | HitTarget::SettingsModal
            | HitTarget::SettingsLanguageEnglish
            | HitTarget::SettingsLanguageSpanish
            | HitTarget::Install
            | HitTarget::Update
            | HitTarget::InstallHelpModal
            | HitTarget::InstallHelpClose
            | HitTarget::DebugClearSave => {}
        }
    }

    fn adjust_debug_level(&mut self, delta: isize) {
        if !self.debug_mode || !matches!(self.screen, AppScreen::Map) {
            return;
        }

        let changed = {
            let Some(dungeon) = self.dungeon.as_mut() else {
                return;
            };
            let target_level = (dungeon.current_level() as isize + delta).max(1) as usize;
            dungeon.debug_set_level(target_level)
        };

        if changed {
            self.refresh_hover();
            self.refresh_run_save_snapshot();
            self.dirty = true;
        }
    }

    fn debug_fill_deck(&mut self) {
        if !self.debug_mode || !matches!(self.screen, AppScreen::Map) {
            return;
        }

        let added = {
            let Some(dungeon) = self.dungeon.as_mut() else {
                return;
            };
            let mut added = 0usize;
            for &card in all_base_cards() {
                if !dungeon.deck.contains(&card) {
                    dungeon.add_card(card);
                    added += 1;
                }
            }
            added
        };

        if added > 0 {
            self.push_log(match self.language {
                Language::English => format!("Filled deck with {added} cards."),
                Language::Spanish => format!("Se llenó el mazo con {added} cartas."),
            });
        } else {
            self.push_log(self.tr(
                "Deck already contains all base cards.",
                "El mazo ya contiene todas las cartas base.",
            ));
        }

        self.refresh_hover();
        self.refresh_run_save_snapshot();
        self.dirty = true;
    }

    fn handle_reward_pointer(&mut self, x: f32, y: f32) {
        let Some(target) = self.hit_test(x, y) else {
            return;
        };

        match target {
            HitTarget::RewardCard(index) => self.claim_reward(index),
            HitTarget::RewardSkip => self.skip_reward(),
            _ => {}
        }
    }

    fn handle_shop_pointer(&mut self, x: f32, y: f32) {
        let Some(target) = self.hit_test(x, y) else {
            return;
        };

        match target {
            HitTarget::ShopCard(index) => self.claim_shop_offer(index),
            HitTarget::ShopLeave => self.leave_shop(),
            _ => {}
        }
    }

    fn handle_event_pointer(&mut self, x: f32, y: f32) {
        let Some(target) = self.hit_test(x, y) else {
            return;
        };

        if let HitTarget::EventChoice(index) = target {
            self.claim_event_choice(index);
        }
    }

    fn handle_level_intro_pointer(&mut self, x: f32, y: f32) {
        if self.hit_test(x, y) == Some(HitTarget::Continue) {
            self.continue_from_level_intro();
        }
    }

    fn handle_opening_intro_pointer(&mut self, x: f32, y: f32) {
        if self.hit_test(x, y) == Some(HitTarget::Continue) {
            self.handle_opening_intro_action();
        }
    }

    fn handle_opening_intro_action(&mut self) {
        self.continue_from_opening_intro();
    }

    fn handle_rest_pointer(&mut self, x: f32, y: f32) {
        let Some(target) = self.hit_test(x, y) else {
            return;
        };

        match target {
            HitTarget::RestHeal => self.select_rest_option(RestSelection::Heal),
            HitTarget::RestCard(index) => self.select_rest_option(RestSelection::Upgrade(index)),
            HitTarget::RestConfirm => self.confirm_rest_selection(),
            HitTarget::RestPagePrev => self.set_rest_page(self.ui.rest_page.saturating_sub(1)),
            HitTarget::RestPageNext => self.set_rest_page(self.ui.rest_page.saturating_add(1)),
            _ => {}
        }
    }

    fn rest_heal_actionable(&self) -> bool {
        self.rest
            .as_ref()
            .is_some_and(|rest| rest.heal_amount > 0 || rest.upgrade_options.is_empty())
    }

    fn rest_page_info(&self, requested_page: usize) -> Option<RestPageInfo> {
        let rest = self.rest.as_ref()?;
        let upgrade_count = rest.upgrade_options.len();
        let logical_width = self.logical_width();
        let logical_height = self.logical_height();
        let gap = HAND_MIN_GAP * 1.3;
        let preferred_columns = if logical_width < 540.0 {
            upgrade_count.clamp(1, 2)
        } else if upgrade_count >= 9 {
            5
        } else if upgrade_count >= 6 {
            4
        } else if upgrade_count >= 3 {
            3
        } else {
            upgrade_count.max(1)
        };
        let max_columns_for_width =
            (((logical_width - gap).max(0.0)) / (136.0 + gap)).floor() as usize;
        let columns = preferred_columns
            .min(max_columns_for_width.max(1))
            .min(upgrade_count.max(1));
        let rows_per_page = if logical_height < 640.0 { 2 } else { 3 };
        let page_size = (columns * rows_per_page).max(1);
        let page_count = if upgrade_count == 0 {
            0
        } else {
            upgrade_count.div_ceil(page_size)
        };
        let current_page = if page_count == 0 {
            0
        } else {
            requested_page.min(page_count - 1)
        };
        let visible_upgrade_indices = if page_count == 0 {
            Vec::new()
        } else {
            let start = current_page * page_size;
            let end = (start + page_size).min(upgrade_count);
            (start..end).collect()
        };

        Some(RestPageInfo {
            current_page,
            page_count,
            columns,
            visible_upgrade_indices,
        })
    }

    fn sync_rest_page_state(&mut self) {
        let Some(page_info) = self.rest_page_info(self.ui.rest_page) else {
            return;
        };
        self.ui.rest_page = page_info.current_page;
        if let Some(RestSelection::Upgrade(index)) = self.ui.rest_selection {
            if !page_info.visible_upgrade_indices.contains(&index) {
                self.ui.rest_selection = None;
            }
        }
    }

    fn set_rest_page(&mut self, requested_page: usize) {
        let previous_page = self.ui.rest_page;
        let previous_selection = self.ui.rest_selection;
        self.ui.rest_page = requested_page;
        self.sync_rest_page_state();
        if self.ui.rest_page != previous_page || self.ui.rest_selection != previous_selection {
            self.refresh_hover();
            self.dirty = true;
        }
    }

    fn select_rest_option(&mut self, selection: RestSelection) {
        let valid = match selection {
            RestSelection::Heal => self.rest_heal_actionable(),
            RestSelection::Upgrade(index) => self
                .rest
                .as_ref()
                .is_some_and(|rest| index < rest.upgrade_options.len()),
        };
        if !valid {
            return;
        }

        if self.ui.rest_selection != Some(selection) {
            self.ui.rest_selection = Some(selection);
            self.dirty = true;
        }
    }

    fn confirm_rest_selection(&mut self) {
        match self.ui.rest_selection {
            Some(RestSelection::Heal) => self.claim_rest_heal(),
            Some(RestSelection::Upgrade(index)) => self.claim_rest_upgrade(index),
            None => {}
        }
    }

    fn select_map_node(&mut self, node_id: usize) {
        let debug_mode = self.debug_mode;
        let selection = self.dungeon.as_mut().and_then(|dungeon| {
            if debug_mode && !dungeon.is_available(node_id) {
                dungeon.debug_select_node(node_id)
            } else {
                dungeon.select_node(node_id)
            }
        });

        match selection {
            Some(NodeSelection::Encounter(setup)) => self.begin_encounter(setup),
            Some(NodeSelection::Rest) => self.begin_rest(),
            Some(NodeSelection::Shop) => self.begin_shop(),
            Some(NodeSelection::Event(event)) => self.begin_event(event),
            None => {}
        }
    }

    fn begin_rest(&mut self) {
        let from_screen = self.screen;
        let Some(dungeon) = self.dungeon.as_ref() else {
            return;
        };
        self.rest = Some(RestState {
            heal_amount: dungeon.rest_heal_amount(),
            upgrade_options: dungeon.upgradable_card_indices(),
        });
        self.module_select = None;
        self.shop = None;
        self.event = None;
        self.reward = None;
        self.level_intro = None;
        self.share_request = None;
        self.screen = AppScreen::Rest;
        self.ui = UiState::default();
        self.pointer_pos = None;
        self.layout_transition = None;
        self.screen_transition = None;
        self.begin_screen_transition(from_screen, AppScreen::Rest);
        self.refresh_run_save_snapshot();
        self.dirty = true;
    }

    fn begin_reward(&mut self, tier: RewardTier, followup: RewardFollowup, seed: u64) {
        let from_screen = self.screen;
        let reward_level = self
            .dungeon
            .as_ref()
            .map(DungeonRun::current_level)
            .unwrap_or(1);
        self.rest = None;
        self.shop = None;
        self.event = None;
        self.module_select = None;
        self.level_intro = None;
        self.share_request = None;
        self.reward = Some(RewardState {
            tier,
            options: reward_choices(seed, tier, reward_level),
            followup,
            seed,
        });
        self.screen = AppScreen::Reward;
        self.ui = UiState::default();
        self.pointer_pos = None;
        self.layout_transition = None;
        self.screen_transition = None;
        self.begin_screen_transition(from_screen, AppScreen::Reward);
        self.refresh_run_save_snapshot();
        self.dirty = true;
    }

    fn begin_boss_module_reward(&mut self, boss_level: usize) {
        let from_screen = self.screen;
        let Some(dungeon) = self.dungeon.as_ref() else {
            return;
        };
        let options = boss_module_choices(boss_level)
            .into_iter()
            .filter(|module| !dungeon.has_module(*module))
            .collect::<Vec<_>>();
        if options.is_empty() {
            self.begin_level_intro();
            return;
        }

        let seed = dungeon.current_room_seed().unwrap_or(BASE_SEED);
        self.rest = None;
        self.shop = None;
        self.event = None;
        self.reward = None;
        self.level_intro = None;
        self.share_request = None;
        self.module_select = Some(ModuleSelectState {
            options,
            seed,
            context: ModuleSelectContext::BossReward { boss_level },
        });
        self.screen = AppScreen::ModuleSelect;
        self.ui = UiState::default();
        self.pointer_pos = None;
        self.layout_transition = None;
        self.screen_transition = None;
        self.begin_screen_transition(from_screen, AppScreen::ModuleSelect);
        self.refresh_run_save_snapshot();
        self.dirty = true;
    }

    fn begin_shop(&mut self) {
        let from_screen = self.screen;
        let Some(dungeon) = self.dungeon.as_ref() else {
            return;
        };
        let seed = dungeon.current_room_seed().unwrap_or(BASE_SEED);
        self.rest = None;
        self.module_select = None;
        self.event = None;
        self.reward = None;
        self.level_intro = None;
        self.share_request = None;
        self.shop = Some(ShopState {
            offers: shop_offers(seed, dungeon.current_level()),
            seed,
        });
        self.screen = AppScreen::Shop;
        self.ui = UiState::default();
        self.pointer_pos = None;
        self.layout_transition = None;
        self.screen_transition = None;
        self.begin_screen_transition(from_screen, AppScreen::Shop);
        self.refresh_run_save_snapshot();
        self.dirty = true;
    }

    fn begin_level_intro(&mut self) {
        let from_screen = self.screen;
        let Some(dungeon) = self.dungeon.as_ref() else {
            return;
        };
        self.rest = None;
        self.shop = None;
        self.event = None;
        self.module_select = None;
        self.reward = None;
        self.share_request = None;
        self.level_intro = Some(LevelIntroState {
            level: dungeon.current_level(),
            codename: localized_level_codename(dungeon.current_level(), self.language),
            summary: localized_level_summary(dungeon.current_level(), self.language),
        });
        self.opening_intro = None;
        self.screen = AppScreen::LevelIntro;
        self.ui = UiState::default();
        self.pointer_pos = None;
        self.layout_transition = None;
        self.screen_transition = None;
        self.begin_screen_transition(from_screen, AppScreen::LevelIntro);
        self.refresh_run_save_snapshot();
        self.dirty = true;
    }

    fn begin_event(&mut self, event: EventId) {
        let from_screen = self.screen;
        self.rest = None;
        self.shop = None;
        self.module_select = None;
        self.reward = None;
        self.level_intro = None;
        self.opening_intro = None;
        self.share_request = None;
        self.event = Some(EventState { event });
        self.screen = AppScreen::Event;
        self.ui = UiState::default();
        self.pointer_pos = None;
        self.layout_transition = None;
        self.screen_transition = None;
        self.begin_screen_transition(from_screen, AppScreen::Event);
        self.refresh_run_save_snapshot();
        self.dirty = true;
    }

    #[cfg_attr(not(test), allow(dead_code))]
    fn complete_opening_intro(&mut self) {
        let total_duration_ms = self.opening_intro_total_duration_ms();
        let Some(opening_intro) = self.opening_intro.as_mut() else {
            return;
        };
        if opening_intro.elapsed_ms >= total_duration_ms {
            return;
        }

        opening_intro.elapsed_ms = total_duration_ms;
        opening_intro.button_transition_ms = 0.0;
        self.dirty = true;
    }

    fn continue_from_opening_intro(&mut self) {
        if !matches!(self.screen, AppScreen::OpeningIntro) {
            return;
        }

        let from_screen = self.screen;
        self.screen = AppScreen::ModuleSelect;
        self.ui = UiState::default();
        self.pointer_pos = None;
        self.share_request = None;
        self.layout_transition = None;
        self.screen_transition = None;
        self.begin_screen_transition(from_screen, AppScreen::ModuleSelect);
        self.refresh_run_save_snapshot();
        self.dirty = true;
    }

    fn continue_from_level_intro(&mut self) {
        if !matches!(self.screen, AppScreen::LevelIntro) {
            return;
        }

        let from_screen = self.screen;
        self.screen = AppScreen::Map;
        self.ui = UiState::default();
        self.pointer_pos = None;
        self.share_request = None;
        self.layout_transition = None;
        self.screen_transition = None;
        self.begin_screen_transition(from_screen, AppScreen::Map);
        self.refresh_run_save_snapshot();
        self.dirty = true;
    }

    fn claim_rest_heal(&mut self) {
        let can_resolve = self
            .rest
            .as_ref()
            .is_some_and(|rest| rest.heal_amount > 0 || rest.upgrade_options.is_empty());
        if !can_resolve {
            return;
        }
        let Some((healed, progress)) = self
            .dungeon
            .as_mut()
            .and_then(|dungeon| dungeon.resolve_rest_heal())
        else {
            return;
        };

        if healed > 0 {
            self.push_log(match self.language {
                Language::English => format!("Rest site restores {healed} HP."),
                Language::Spanish => format!("El descanso restaura {healed} HP."),
            });
        } else {
            self.push_log(self.tr("Rest site complete.", "Descanso completado."));
        }
        self.finish_rest_action(progress);
    }

    fn claim_rest_upgrade(&mut self, index: usize) {
        let deck_index = self
            .rest
            .as_ref()
            .and_then(|rest| rest.upgrade_options.get(index).copied());
        let Some(deck_index) = deck_index else {
            return;
        };
        let Some((from, to, progress)) = self
            .dungeon
            .as_mut()
            .and_then(|dungeon| dungeon.resolve_rest_upgrade(deck_index))
        else {
            return;
        };

        self.push_log(format!(
            "{} {} {}.",
            self.tr("Upgraded", "Mejoraste"),
            localized_card_name(from, self.language),
            match self.language {
                Language::English => format!("to {}", localized_card_name(to, self.language)),
                Language::Spanish => format!("a {}", localized_card_name(to, self.language)),
            }
        ));
        self.finish_rest_action(progress);
    }

    fn finish_rest_action(&mut self, progress: DungeonProgress) {
        self.ui = UiState::default();
        self.pointer_pos = None;
        self.share_request = None;
        let from_screen = self.screen;
        match progress {
            DungeonProgress::Continue => {
                self.screen = AppScreen::Map;
                self.begin_screen_transition(from_screen, AppScreen::Map);
            }
            DungeonProgress::Completed => {
                self.screen = AppScreen::Result(CombatOutcome::Victory);
                self.begin_screen_transition(from_screen, self.screen);
            }
        }
        self.refresh_run_save_snapshot();
        self.dirty = true;
    }

    fn claim_event_choice(&mut self, choice_index: usize) {
        let Some(event_id) = self.event.as_ref().map(|state| state.event) else {
            return;
        };
        let Some((resolution, progress)) = self
            .dungeon
            .as_mut()
            .and_then(|dungeon| dungeon.resolve_event_choice(event_id, choice_index))
        else {
            return;
        };

        self.push_event_resolution_log(event_id, resolution);
        self.finish_event_action(progress);
    }

    fn push_event_resolution_log(&mut self, event_id: EventId, resolution: EventResolution) {
        let title = localized_event_def(event_id, self.language).title;
        match resolution {
            EventResolution::Credits {
                hp_lost: 0,
                credits_gained,
            } => self.push_log(match self.language {
                Language::English => {
                    format!(
                        "Recovered {} from {title}.",
                        credits_label(credits_gained, self.language)
                    )
                }
                Language::Spanish => {
                    format!(
                        "Recuperaste {} de {title}.",
                        credits_label(credits_gained, self.language)
                    )
                }
            }),
            EventResolution::Credits {
                hp_lost,
                credits_gained,
            } => self.push_log(match self.language {
                Language::English => format!(
                    "Lost {hp_lost} HP and recovered {} from {title}.",
                    credits_label(credits_gained, self.language)
                ),
                Language::Spanish => format!(
                    "Perdiste {hp_lost} HP y recuperaste {} de {title}.",
                    credits_label(credits_gained, self.language)
                ),
            }),
            EventResolution::Heal { healed: 0 } => self.push_log(match self.language {
                Language::English => format!("{title} completes with no repairs needed."),
                Language::Spanish => format!("{title} termina sin necesidad de reparaciones."),
            }),
            EventResolution::Heal { healed } => self.push_log(match self.language {
                Language::English => format!("{title} restores {healed} HP."),
                Language::Spanish => format!("{title} restaura {healed} HP."),
            }),
            EventResolution::MaxHp {
                hp_lost,
                max_hp_gained,
            } => self.push_log(match self.language {
                Language::English => {
                    format!("Lost {hp_lost} HP and gained {max_hp_gained} max HP at {title}.")
                }
                Language::Spanish => {
                    format!(
                        "Perdiste {hp_lost} HP y ganaste {max_hp_gained} de HP máximo en {title}."
                    )
                }
            }),
            EventResolution::Card { hp_lost: 0, card } => self.push_log(match self.language {
                Language::English => format!(
                    "Added {} to the deck from {title}.",
                    localized_card_name(card, self.language)
                ),
                Language::Spanish => format!(
                    "Añadiste {} al mazo desde {title}.",
                    localized_card_name(card, self.language)
                ),
            }),
            EventResolution::Card { hp_lost, card } => self.push_log(match self.language {
                Language::English => format!(
                    "Lost {hp_lost} HP and added {} to the deck from {title}.",
                    localized_card_name(card, self.language)
                ),
                Language::Spanish => format!(
                    "Perdiste {hp_lost} HP y añadiste {} al mazo desde {title}.",
                    localized_card_name(card, self.language)
                ),
            }),
        }
    }

    fn finish_event_action(&mut self, progress: DungeonProgress) {
        self.event = None;
        self.ui = UiState::default();
        self.pointer_pos = None;
        self.share_request = None;
        let from_screen = self.screen;
        match progress {
            DungeonProgress::Continue => {
                self.screen = AppScreen::Map;
                self.begin_screen_transition(from_screen, AppScreen::Map);
            }
            DungeonProgress::Completed => {
                self.screen = AppScreen::Result(CombatOutcome::Victory);
                self.begin_screen_transition(from_screen, self.screen);
            }
        }
        self.refresh_run_save_snapshot();
        self.dirty = true;
    }

    fn claim_shop_offer(&mut self, index: usize) {
        let Some(offer) = self
            .shop
            .as_ref()
            .and_then(|shop| shop.offers.get(index).copied())
        else {
            return;
        };
        let Some(dungeon) = self.dungeon.as_ref() else {
            return;
        };
        if !dungeon.can_afford_shop_price(offer.price) {
            return;
        }
        let Some(progress) = self
            .dungeon
            .as_mut()
            .and_then(|dungeon| dungeon.resolve_shop_purchase(offer.card, offer.price))
        else {
            return;
        };

        self.push_log(format!(
            "{} {} {}.",
            self.tr("Bought", "Compraste"),
            localized_card_name(offer.card, self.language),
            match self.language {
                Language::English => format!("for {}", credits_label(offer.price, self.language)),
                Language::Spanish => format!("por {}", credits_label(offer.price, self.language)),
            }
        ));
        self.finish_shop_action(progress);
    }

    fn leave_shop(&mut self) {
        let Some(progress) = self
            .dungeon
            .as_mut()
            .and_then(DungeonRun::resolve_shop_leave)
        else {
            return;
        };

        self.push_log(self.tr("Left shop.", "Saliste de la tienda."));
        self.finish_shop_action(progress);
    }

    fn finish_shop_action(&mut self, progress: DungeonProgress) {
        self.shop = None;
        self.ui = UiState::default();
        self.pointer_pos = None;
        self.share_request = None;
        let from_screen = self.screen;
        match progress {
            DungeonProgress::Continue => {
                self.screen = AppScreen::Map;
                self.begin_screen_transition(from_screen, AppScreen::Map);
            }
            DungeonProgress::Completed => {
                self.screen = AppScreen::Result(CombatOutcome::Victory);
                self.begin_screen_transition(from_screen, self.screen);
            }
        }
        self.refresh_run_save_snapshot();
        self.dirty = true;
    }

    fn claim_reward(&mut self, index: usize) {
        let Some(card) = self
            .reward
            .as_ref()
            .and_then(|reward| reward.options.get(index).copied())
        else {
            return;
        };
        self.resolve_reward_choice(Some(card));
    }

    fn skip_reward(&mut self) {
        if self.reward.is_none() {
            return;
        }
        self.resolve_reward_choice(None);
    }

    fn resolve_reward_choice(&mut self, selected_card: Option<CardId>) {
        let Some(reward) = self.reward.as_ref() else {
            return;
        };
        let reward_tier = reward.tier;
        let followup = reward.followup;

        if let Some(card) = selected_card {
            if let Some(dungeon) = &mut self.dungeon {
                dungeon.add_card(card);
            }
            self.push_log(match self.language {
                Language::English => {
                    format!(
                        "Added {} to the deck.",
                        localized_card_name(card, self.language)
                    )
                }
                Language::Spanish => {
                    format!(
                        "Añadiste {} al mazo.",
                        localized_card_name(card, self.language)
                    )
                }
            });
        } else {
            self.push_log(self.tr("Skipped card reward.", "Saltaste la recompensa de carta."));
        }

        self.ui = UiState::default();
        self.pointer_pos = None;
        self.share_request = None;
        self.event = None;
        if followup.completed_run {
            let from_screen = self.screen;
            self.reward = None;
            self.screen = AppScreen::Result(CombatOutcome::Victory);
            self.begin_screen_transition(from_screen, self.screen);
            self.refresh_run_save_snapshot();
        } else if matches!(reward_tier, RewardTier::Boss) {
            let boss_level = self
                .dungeon
                .as_ref()
                .map(|dungeon| dungeon.current_level().saturating_sub(1).max(1))
                .unwrap_or(1);
            self.begin_boss_module_reward(boss_level);
        } else {
            let from_screen = self.screen;
            self.reward = None;
            self.screen = AppScreen::Map;
            self.begin_screen_transition(from_screen, AppScreen::Map);
            self.refresh_run_save_snapshot();
        }
        self.dirty = true;
    }

    fn handle_combat_pointer(&mut self, x: f32, y: f32) {
        if self.combat_input_locked() {
            return;
        }

        let Some(target) = self.hit_test(x, y) else {
            if self.ui.run_info_open {
                self.close_run_info();
                return;
            }
            if self.ui.enemy_inspect_open {
                self.close_enemy_inspect();
                return;
            }
            if self.run_info_visible() {
                self.close_run_info();
                return;
            }
            if self.enemy_inspect_visible() {
                self.close_enemy_inspect();
                return;
            }
            if self.ui.selected_card.is_some() {
                self.snapshot_combat_layout_target();
            }
            if self.ui.selected_card.take().is_some() {
                self.push_log(self.tr("Selection cleared.", "Selección cancelada."));
                self.refresh_hover();
                self.dirty = true;
            }
            return;
        };

        match target {
            HitTarget::Menu => self.return_to_menu(),
            HitTarget::RunInfoPanel => {}
            HitTarget::EnemyInspectPanel => {}
            HitTarget::Card(index) => self.select_or_play_card(index),
            HitTarget::Enemy(enemy_index) => {
                if let Some(selected) = self.ui.selected_card {
                    if self.combat.card_requires_enemy(selected) {
                        self.perform_action(CombatAction::PlayCard {
                            hand_index: selected,
                            target: Some(Actor::Enemy(enemy_index)),
                        });
                    } else if self.combat.card_targets_all_enemies(selected) {
                        self.perform_action(CombatAction::PlayCard {
                            hand_index: selected,
                            target: None,
                        });
                    }
                } else {
                    self.toggle_or_switch_enemy_inspect(enemy_index);
                }
            }
            HitTarget::Player => {
                if self.ui.selected_card.is_none() {
                    if self.ui.enemy_inspect_open || self.enemy_inspect_visible() {
                        self.open_run_info();
                    } else {
                        self.toggle_run_info();
                    }
                } else if let Some(selected) = self.ui.selected_card {
                    if !self.combat.card_requires_enemy(selected)
                        && !self.combat.card_targets_all_enemies(selected)
                    {
                        self.perform_action(CombatAction::PlayCard {
                            hand_index: selected,
                            target: Some(Actor::Player),
                        });
                    }
                }
            }
            HitTarget::EndTurn => self.perform_action(CombatAction::EndTurn),
            HitTarget::EndBattle => self.debug_end_battle(),
            HitTarget::Start
            | HitTarget::Continue
            | HitTarget::DebugLevelDown
            | HitTarget::DebugLevelUp
            | HitTarget::DebugFillDeck
            | HitTarget::Share
            | HitTarget::Restart
            | HitTarget::RestHeal
            | HitTarget::RestCard(_)
            | HitTarget::RestConfirm
            | HitTarget::RestPagePrev
            | HitTarget::RestPageNext
            | HitTarget::ShopCard(_)
            | HitTarget::ShopLeave
            | HitTarget::EventChoice(_)
            | HitTarget::Legend
            | HitTarget::LegendPanel
            | HitTarget::Info
            | HitTarget::RewardCard(_)
            | HitTarget::RewardSkip
            | HitTarget::MapNode(_)
            | HitTarget::ModuleSelectCard(_)
            | HitTarget::RestartModal
            | HitTarget::RestartConfirm
            | HitTarget::RestartCancel
            | HitTarget::Settings
            | HitTarget::SettingsModal
            | HitTarget::SettingsLanguageEnglish
            | HitTarget::SettingsLanguageSpanish
            | HitTarget::Install
            | HitTarget::Update
            | HitTarget::InstallHelpModal
            | HitTarget::InstallHelpClose
            | HitTarget::DebugClearSave => {}
        }
    }

    fn select_or_play_card(&mut self, index: usize) {
        if self.combat_input_locked() {
            return;
        }

        if self.ui.selected_card == Some(index) {
            if self.combat.card_targets_all_enemies(index) {
                self.perform_action(CombatAction::PlayCard {
                    hand_index: index,
                    target: None,
                });
                return;
            }
            self.dirty = true;
            return;
        }

        self.snapshot_combat_layout_target();
        self.ui.selected_card = Some(index);
        if let Some(card) = self.combat.hand_card(index) {
            self.push_log(match self.language {
                Language::English => {
                    format!("Selected {}.", localized_card_name(card, self.language))
                }
                Language::Spanish => {
                    format!("Elegiste {}.", localized_card_name(card, self.language))
                }
            });
        }
        self.refresh_hover();
        self.dirty = true;
    }

    fn perform_action(&mut self, action: CombatAction) {
        if self.combat_input_locked() {
            return;
        }

        let previous_layout = self.layout();
        self.enemy_vfx_rects = (0..self.combat.enemy_count())
            .map(|enemy_index| previous_layout.enemy_rect(enemy_index))
            .collect();
        let previous_hand_len = self.combat.hand_len();
        let displayed_before_action = displayed_combat_stats(&self.combat);
        let intents_before_action: Vec<_> = (0..self.combat.enemy_count())
            .map(|enemy_index| self.current_displayed_intent(enemy_index))
            .collect();
        let end_turn_bursts: Vec<(Rect, CardId)> = if matches!(action, CombatAction::EndTurn) {
            previous_layout
                .hand_rects
                .iter()
                .enumerate()
                .filter_map(|(index, rect)| self.combat.hand_card(index).map(|card| (*rect, card)))
                .collect()
        } else {
            Vec::new()
        };
        let played_hand_index = match action {
            CombatAction::PlayCard { hand_index, .. } => Some(hand_index),
            CombatAction::EndTurn => None,
        };
        let played_card_rect = played_hand_index
            .and_then(|hand_index| previous_layout.hand_rects.get(hand_index).copied());
        let events = self.combat.dispatch(action);
        let played_card = events.iter().find_map(|event| match event {
            CombatEvent::CardPlayed { card } => Some(*card),
            _ => None,
        });
        let defeated_enemy_indices: Vec<_> = events
            .iter()
            .filter_map(|event| match event {
                CombatEvent::ActorDefeated {
                    actor: Actor::Enemy(enemy_index),
                } => Some(*enemy_index),
                _ => None,
            })
            .collect();
        let use_player_action_playback = matches!(action, CombatAction::PlayCard { .. })
            && events.iter().any(|event| {
                matches!(
                    event,
                    CombatEvent::BlockSpent { .. }
                        | CombatEvent::DamageDealt { .. }
                        | CombatEvent::BlockGained { .. }
                        | CombatEvent::BlockCleared { .. }
                )
            });

        if matches!(action, CombatAction::EndTurn) {
            self.begin_end_turn_playback(events, displayed_before_action, intents_before_action);
        } else if use_player_action_playback {
            self.begin_player_action_playback(
                events,
                displayed_before_action,
                intents_before_action,
            );
        } else {
            self.handle_events(&events);
            self.sync_combat_feedback_to_combat();
        }

        if let Some(rect) = played_card_rect {
            if let Some(card) = played_card {
                self.spawn_card_pixel_burst(rect, card);
            }
        }
        if matches!(action, CombatAction::EndTurn) {
            for (rect, card) in end_turn_bursts {
                self.spawn_card_pixel_burst(rect, card);
            }
        }
        if !matches!(action, CombatAction::EndTurn) && !use_player_action_playback {
            for enemy_index in defeated_enemy_indices {
                self.mark_enemy_defeat_vfx_started(enemy_index);
                if let Some(rect) = previous_layout.enemy_rect(enemy_index) {
                    self.spawn_enemy_pixel_burst(rect);
                }
            }
        }

        if matches!(
            action,
            CombatAction::PlayCard { .. } | CombatAction::EndTurn
        ) || !self.combat.is_player_turn()
        {
            self.ui.selected_card = None;
        } else if let Some(index) = self.ui.selected_card {
            if index >= self.combat.hand_len() {
                self.ui.selected_card = None;
            }
        }

        if let Some(outcome) = self.combat.outcome() {
            if use_player_action_playback && played_card.is_some() {
                self.screen = AppScreen::Combat;
                self.begin_layout_transition(previous_layout, previous_hand_len, played_hand_index);
            } else if !matches!(action, CombatAction::EndTurn) && !use_player_action_playback {
                self.finalize_combat_outcome(outcome);
            }
        } else {
            self.screen = AppScreen::Combat;
            if matches!(action, CombatAction::EndTurn) || played_card.is_some() {
                self.begin_layout_transition(previous_layout, previous_hand_len, played_hand_index);
            }
        }

        if self.combat_feedback.pending_outcome.is_none() {
            self.refresh_run_save_snapshot();
        }
        self.refresh_hover();
        self.dirty = true;
    }

    fn debug_end_battle(&mut self) {
        if !self.debug_mode || !matches!(self.screen, AppScreen::Combat) {
            return;
        }

        let previous_layout = self.layout();
        for rect in previous_layout.enemy_rects {
            self.spawn_enemy_pixel_burst(rect);
        }
        self.ui.selected_card = None;

        self.finalize_combat_outcome(CombatOutcome::Victory);

        self.refresh_run_save_snapshot();
        self.refresh_hover();
        self.dirty = true;
    }

    fn finalize_pending_combat_outcome(&mut self) {
        let Some(outcome) = self.combat_feedback.pending_outcome.take() else {
            return;
        };
        self.finalize_combat_outcome(outcome);
        self.refresh_hover();
    }

    fn finalize_combat_outcome(&mut self, outcome: CombatOutcome) {
        match outcome {
            CombatOutcome::Victory => {
                let reward_context = self.dungeon.as_ref().and_then(|dungeon| {
                    let tier = match dungeon.current_room_kind()? {
                        RoomKind::Combat => RewardTier::Combat,
                        RoomKind::Elite => RewardTier::Elite,
                        RoomKind::Boss => RewardTier::Boss,
                        RoomKind::Start | RoomKind::Rest | RoomKind::Shop | RoomKind::Event => {
                            return None;
                        }
                    };
                    let seed = dungeon.current_room_seed()?;
                    Some((tier, seed))
                });
                let victory_resolution = self.dungeon.as_mut().and_then(|dungeon| {
                    dungeon.resolve_combat_victory(self.combat.player.fighter.hp)
                });
                let (progress, credits_gained) = match victory_resolution {
                    Some((progress, credits_gained)) => (Some(progress), credits_gained),
                    None => (None, 0),
                };
                self.apply_post_victory_modules();
                if credits_gained > 0 {
                    self.push_log(match self.language {
                        Language::English => {
                            format!("Gained {}.", credits_label(credits_gained, self.language))
                        }
                        Language::Spanish => {
                            format!("Ganaste {}.", credits_label(credits_gained, self.language))
                        }
                    });
                }
                if let Some((tier, seed)) = reward_context {
                    if matches!(progress, Some(DungeonProgress::Continue)) {
                        let followup = RewardFollowup {
                            completed_run: false,
                        };
                        self.begin_reward(tier, followup, seed);
                    } else {
                        let from_screen = self.screen;
                        self.screen = AppScreen::Result(CombatOutcome::Victory);
                        self.begin_screen_transition(from_screen, self.screen);
                    }
                } else {
                    let from_screen = self.screen;
                    match progress {
                        Some(DungeonProgress::Continue) => {
                            self.screen = AppScreen::Map;
                            self.begin_screen_transition(from_screen, AppScreen::Map);
                        }
                        Some(DungeonProgress::Completed) | None => {
                            self.screen = AppScreen::Result(CombatOutcome::Victory);
                            self.begin_screen_transition(from_screen, self.screen);
                        }
                    }
                }
            }
            CombatOutcome::Defeat => {
                if let Some(dungeon) = &mut self.dungeon {
                    dungeon.resolve_combat_defeat(self.combat.player.fighter.hp);
                }
                let from_screen = self.screen;
                self.screen = AppScreen::Result(CombatOutcome::Defeat);
                self.begin_screen_transition(from_screen, self.screen);
            }
        }
        self.refresh_run_save_snapshot();
    }

    fn apply_start_of_combat_modules(&mut self) {
        let Some(dungeon) = self.dungeon.as_ref() else {
            return;
        };
        let applied_modules = self.combat.apply_start_of_combat_modules(&dungeon.modules);
        let changed = !applied_modules.is_empty();

        for module in applied_modules {
            match module {
                ModuleId::AegisDrive => {
                    self.push_log(self.tr(
                        "Aegis Drive grants 5 Shield.",
                        "Aegis Drive otorga 5 de Escudo.",
                    ));
                }
                ModuleId::TargetingRelay => {
                    self.push_log(match self.language {
                        Language::English => "Targeting Relay grants Focus +1.".to_string(),
                        Language::Spanish => "Relé de Apuntamiento otorga Enfoque +1.".to_string(),
                    });
                }
                ModuleId::Nanoforge => {}
                ModuleId::CapacitorBank => {
                    self.push_log(match self.language {
                        Language::English => "Capacitor Bank grants Momentum +1.".to_string(),
                        Language::Spanish => "Banco de Capacitores otorga Impulso +1.".to_string(),
                    });
                }
                ModuleId::PrismScope => {
                    self.push_log(match self.language {
                        Language::English => {
                            "Prism Scope applies Rhythm -1 to all enemies.".to_string()
                        }
                        Language::Spanish => {
                            "Visor Prisma aplica Ritmo -1 a todos los enemigos.".to_string()
                        }
                    });
                }
                ModuleId::SalvageLedger => {}
                ModuleId::OverclockCore => {
                    self.push_log(match self.language {
                        Language::English => "Overclock Core grants 1 extra Energy.".to_string(),
                        Language::Spanish => {
                            "Núcleo Overclock otorga 1 de Energía extra.".to_string()
                        }
                    });
                }
                ModuleId::SuppressionField => {
                    self.push_log(match self.language {
                        Language::English => {
                            "Suppression Field applies Focus -1 to all enemies.".to_string()
                        }
                        Language::Spanish => {
                            "Campo de Supresión aplica Enfoque -1 a todos los enemigos.".to_string()
                        }
                    });
                }
                ModuleId::RecoveryMatrix => {}
            }
        }

        if changed {
            self.sync_combat_feedback_to_combat();
        }
    }

    fn apply_post_victory_modules(&mut self) {
        let Some(effects) = self.dungeon.as_mut().map(apply_post_victory_module_effects) else {
            return;
        };

        if effects.nanoforge_healed > 0 {
            self.push_log(match self.language {
                Language::English => format!("Nanoforge restores {} HP.", effects.nanoforge_healed),
                Language::Spanish => format!("Nanoforge restaura {} HP.", effects.nanoforge_healed),
            });
        }
        if effects.salvage_applied {
            self.push_log(match self.language {
                Language::English => "Salvage Ledger grants 4 additional Credits.".to_string(),
                Language::Spanish => {
                    "Registro de Chatarra otorga 4 Créditos adicionales.".to_string()
                }
            });
        }
        if effects.recovery_healed > 0 {
            self.push_log(match self.language {
                Language::English => {
                    format!("Recovery Matrix restores {} HP.", effects.recovery_healed)
                }
                Language::Spanish => {
                    format!(
                        "Matriz de Recuperación restaura {} HP.",
                        effects.recovery_healed
                    )
                }
            });
        }
    }

    fn begin_screen_transition(&mut self, from_screen: AppScreen, to_screen: AppScreen) {
        self.ui.hover = None;
        if from_screen == to_screen {
            self.screen_transition = None;
            return;
        }

        let duration_ms = if matches!(from_screen, AppScreen::Combat)
            && matches!(to_screen, AppScreen::Result(_))
        {
            RESULT_SCREEN_TRANSITION_MS
        } else {
            SCREEN_TRANSITION_MS
        };

        self.screen_transition = Some(ScreenTransition {
            from_screen,
            to_screen,
            style: screen_transition_style(from_screen, to_screen),
            from_boot_has_saved_run: self.has_saved_run,
            to_boot_has_saved_run: self.has_saved_run,
            ttl_ms: duration_ms,
            total_ms: duration_ms,
        });
    }

    fn refresh_hover(&mut self) {
        self.refresh_combat_layout_transition();
        let hover = if self.screen_transition.is_some() {
            None
        } else {
            self.pointer_pos.and_then(|(x, y)| self.hit_test(x, y))
        };

        if self.ui.hover != hover {
            self.ui.hover = hover;
            self.dirty = true;
        }
    }

    fn begin_layout_transition(
        &mut self,
        from_layout: Layout,
        previous_hand_len: usize,
        removed_hand_index: Option<usize>,
    ) {
        if !matches!(self.screen, AppScreen::Combat) {
            self.layout_transition = None;
            return;
        }

        let current_hand_len = self.combat.hand_len();
        let fallback_rect = removed_hand_index
            .and_then(|index| from_layout.hand_rects.get(index).copied())
            .or_else(|| {
                previous_hand_len
                    .checked_sub(1)
                    .and_then(|index| from_layout.hand_rects.get(index).copied())
            });
        let mut hand_from_rects = vec![None; current_hand_len];

        for (new_index, from_rect) in hand_from_rects.iter_mut().enumerate() {
            let old_index = match removed_hand_index {
                Some(played_hand_index) => {
                    if new_index < played_hand_index {
                        Some(new_index)
                    } else if new_index + 1 < previous_hand_len {
                        Some(new_index + 1)
                    } else {
                        None
                    }
                }
                None => {
                    if new_index < previous_hand_len {
                        Some(new_index)
                    } else {
                        previous_hand_len.checked_sub(1)
                    }
                }
            };

            *from_rect = old_index
                .and_then(|index| from_layout.hand_rects.get(index).copied())
                .or(fallback_rect);
        }

        self.set_layout_transition(from_layout, self.base_layout(), hand_from_rects);
    }

    fn begin_hand_reveal_transition(&mut self, from_layout: Layout) {
        if !matches!(self.screen, AppScreen::Combat) {
            self.layout_transition = None;
            return;
        }

        let to_layout = self.base_layout();
        if to_layout.hand_rects.is_empty() {
            self.layout_transition = None;
            return;
        }

        let player_center_x = from_layout.player_rect.x + from_layout.player_rect.w * 0.5;
        let player_center_y = from_layout.player_rect.y + from_layout.player_rect.h * 0.5;
        let hand_mid = (to_layout.hand_rects.len().saturating_sub(1)) as f32 * 0.5;
        let hand_from_rects = to_layout
            .hand_rects
            .iter()
            .enumerate()
            .map(|(index, rect)| {
                let spread = (index as f32 - hand_mid) * (rect.w * 0.18).min(28.0);
                Some(Rect {
                    x: player_center_x - rect.w * 0.5 + spread,
                    y: player_center_y - rect.h * 0.32,
                    w: rect.w,
                    h: rect.h,
                })
            })
            .collect();

        self.set_layout_transition(from_layout, to_layout, hand_from_rects);
    }

    fn set_layout_transition(
        &mut self,
        from_layout: Layout,
        to_layout: Layout,
        hand_from_rects: Vec<Option<Rect>>,
    ) {
        self.layout_transition = Some(LayoutTransition {
            from_layout,
            to_layout,
            hand_from_rects,
            ttl_ms: LAYOUT_TRANSITION_MS,
            total_ms: LAYOUT_TRANSITION_MS,
        });
    }

    fn handle_events(&mut self, events: &[CombatEvent]) {
        for event in events {
            self.handle_combat_event(*event, false);
        }
    }

    fn push_damage_log(&mut self, source: Actor, target: Actor, amount: i32) {
        if source == target {
            match target {
                Actor::Player => self.push_log(format!("You suffer {amount}.")),
                Actor::Enemy(enemy_index) => self.push_log(format!(
                    "{} suffers {amount}.",
                    self.enemy_display_name(enemy_index)
                )),
            }
            return;
        }

        match (source, target) {
            (Actor::Player, Actor::Enemy(_)) => {
                self.push_log(format!("You strike for {amount}."));
            }
            (Actor::Enemy(enemy_index), Actor::Player) => {
                self.push_log(format!(
                    "{} hits for {amount}.",
                    self.enemy_display_name(enemy_index)
                ));
            }
            _ => {}
        }
    }

    fn push_log<T>(&mut self, line: T)
    where
        T: Into<String>,
    {
        if self.log.len() >= 7 {
            self.log.pop_front();
        }
        self.log.push_back(line.into());
    }

    fn final_victory_summary(&self) -> Option<FinalVictorySummary> {
        if !matches!(self.screen, AppScreen::Result(CombatOutcome::Victory)) {
            return None;
        }

        let dungeon = self.dungeon.as_ref()?;
        if dungeon.current_level() != dungeon.total_levels() || !dungeon.available_nodes.is_empty()
        {
            return None;
        }

        Some(FinalVictorySummary {
            act_names: (1..=dungeon.total_levels())
                .map(|level| localized_level_codename(level, self.language))
                .collect(),
            total_levels: dungeon.total_levels(),
            player_hp: dungeon.player_hp,
            player_max_hp: dungeon.player_max_hp,
            deck_count: dungeon.deck.len(),
            seed: dungeon.seed,
        })
    }

    fn defeat_summary(&self) -> Option<DefeatSummary> {
        if !matches!(self.screen, AppScreen::Result(CombatOutcome::Defeat)) {
            return None;
        }

        let dungeon = self.dungeon.as_ref()?;
        let briefing = dungeon.current_level_briefing();
        let failure_room = dungeon.current_room_kind();
        let failure_enemy = dungeon
            .current_encounter_setup()
            .and_then(|setup| {
                setup
                    .enemies
                    .first()
                    .map(|enemy| localized_enemy_name(enemy.profile, self.language))
            })
            .or_else(|| {
                failure_room.map(|room_kind| {
                    let profile = match room_kind {
                        RoomKind::Start
                        | RoomKind::Combat
                        | RoomKind::Rest
                        | RoomKind::Shop
                        | RoomKind::Event => briefing.combat_enemy,
                        RoomKind::Elite => briefing.elite_enemy,
                        RoomKind::Boss => briefing.boss_enemy,
                    };
                    localized_enemy_name(profile, self.language)
                })
            });

        Some(DefeatSummary {
            current_level: dungeon.current_level(),
            total_levels: dungeon.total_levels(),
            sector_name: localized_level_codename(dungeon.current_level(), self.language),
            failure_room,
            failure_enemy,
            combats_cleared: dungeon.combats_cleared,
            elites_cleared: dungeon.elites_cleared,
            rests_completed: dungeon.rests_completed,
            bosses_cleared: dungeon.bosses_cleared,
            player_hp: dungeon.player_hp,
            player_max_hp: dungeon.player_max_hp,
            deck_count: dungeon.deck.len(),
            seed: dungeon.seed,
        })
    }

    fn final_victory_share_capture_rect(&self) -> Option<Rect> {
        let summary = self.final_victory_summary()?;
        let logical_width = self.logical_width();
        let logical_height = self.logical_height();
        let center_x = self.logical_center_x();
        let title = self.tr("Run Complete", "Partida completada");
        let stats_line = match self.language {
            Language::English => {
                format!(
                    "{} max HP    {} card deck",
                    summary.player_max_hp, summary.deck_count
                )
            }
            Language::Spanish => {
                format!(
                    "{} HP máximo    mazo de {} cartas",
                    summary.player_max_hp, summary.deck_count
                )
            }
        };
        let seed_line = match self.language {
            Language::English => format!("Seed {}", display_seed(summary.seed)),
            Language::Spanish => format!("Semilla {}", display_seed(summary.seed)),
        };
        let version_line = visible_game_version_label();
        let logo_size = (logical_width.min(logical_height) * 0.12).clamp(72.0, 104.0);
        let title_size = fit_text_size(title, 60.0, (logical_width - 48.0).max(120.0)).max(34.0);
        let stats_size =
            fit_text_size(&stats_line, 18.0, (logical_width - 80.0).max(120.0)).max(12.0);
        let seed_size =
            fit_text_size(&seed_line, 14.0, (logical_width - 80.0).max(120.0)).max(11.0);
        let version_size =
            fit_text_size(&version_line, 14.0, (logical_width - 80.0).max(120.0)).max(11.0);
        let content_width = logo_size
            .max(text_width(title, title_size))
            .max(text_width(&stats_line, stats_size))
            .max(text_width(&seed_line, seed_size))
            .max(text_width(&version_line, version_size));
        let horizontal_pad = 24.0;
        let top = (logical_height * (156.0 / LOGICAL_HEIGHT) - 16.0).max(0.0);
        let bottom = (logical_height * (398.0 / LOGICAL_HEIGHT) + version_size * 0.42 + 16.0)
            .min(logical_height);

        Some(Rect {
            x: (center_x - content_width * 0.5 - horizontal_pad).max(0.0),
            y: top,
            w: (content_width + horizontal_pad * 2.0).min(logical_width),
            h: (bottom - top).max(1.0),
        })
    }

    fn queue_share_request(&mut self) {
        let Some(summary) = self.final_victory_summary() else {
            return;
        };
        self.share_request = Some(final_victory_share_payload(&summary, self.language));
    }

    pub(crate) fn share_request_ptr(&self) -> *const u8 {
        self.share_request
            .as_ref()
            .map(|value| value.as_ptr())
            .unwrap_or(std::ptr::null())
    }

    pub(crate) fn share_request_len(&self) -> usize {
        self.share_request
            .as_ref()
            .map(|value| value.len())
            .unwrap_or(0)
    }

    pub(crate) fn clear_share_request(&mut self) {
        self.share_request = None;
    }

    pub(crate) fn share_capture_x(&self) -> f32 {
        self.final_victory_share_capture_rect()
            .map(|rect| rect.x)
            .unwrap_or(0.0)
    }

    pub(crate) fn share_capture_y(&self) -> f32 {
        self.final_victory_share_capture_rect()
            .map(|rect| rect.y)
            .unwrap_or(0.0)
    }

    pub(crate) fn share_capture_w(&self) -> f32 {
        self.final_victory_share_capture_rect()
            .map(|rect| rect.w)
            .unwrap_or(0.0)
    }

    pub(crate) fn share_capture_h(&self) -> f32 {
        self.final_victory_share_capture_rect()
            .map(|rect| rect.h)
            .unwrap_or(0.0)
    }

    fn spawn_damage_floater(&mut self, actor: Actor, amount: i32) {
        let (x, y) = self.anchor_for(actor);
        self.floaters.push(Floater {
            text: format!("-{amount}"),
            x,
            y,
            ttl_ms: 820.0,
            total_ms: 820.0,
            color: (255, 79, 216),
        });
    }

    fn spawn_block_floater(&mut self, actor: Actor, amount: i32) {
        let (x, y) = self.anchor_for(actor);
        self.floaters.push(Floater {
            text: format!("+{amount} {}", self.tr(GUARD_LABEL, "Escudo")),
            x,
            y: y - 30.0,
            ttl_ms: 780.0,
            total_ms: 780.0,
            color: (61, 245, 255),
        });
    }

    fn spawn_status_floater(&mut self, actor: Actor, status: StatusKind, amount: i8) {
        let (x, y) = self.anchor_for(actor);
        let amount_text = match status {
            StatusKind::Bleed => amount.to_string(),
            _ => signed_axis_value(amount),
        };
        let text = format!(
            "{} {}",
            status_display_name(status, self.language),
            amount_text
        );
        let color = status_display_rgb(status);
        self.floaters.push(Floater {
            text,
            x,
            y: y - 60.0,
            ttl_ms: 900.0,
            total_ms: 900.0,
            color,
        });
    }

    fn anchor_for(&self, actor: Actor) -> (f32, f32) {
        let layout = self.layout();
        match actor {
            Actor::Player => (
                layout.player_rect.x + layout.player_rect.w * 0.5,
                layout.player_rect.y + layout.player_rect.h * 0.5,
            ),
            Actor::Enemy(enemy_index) => layout.enemy_rect(enemy_index).map_or(
                (
                    layout.player_rect.x + layout.player_rect.w * 0.5,
                    layout.player_rect.y,
                ),
                |rect| (rect.x + rect.w * 0.5, rect.y + rect.h * 0.5),
            ),
        }
    }

    fn level_intro_continue_button_rect(&self) -> Rect {
        let (pad_x, pad_y) = boot_button_tile_padding();
        let button = centered_button_rect(
            self.tr("Continue", "Continuar"),
            START_BUTTON_FONT_SIZE,
            pad_x,
            pad_y,
            self.logical_width() * 0.5,
            0.0,
        );
        Rect {
            x: button.x,
            y: (self.logical_height() - button.h - pad_y).max(0.0),
            w: button.w,
            h: button.h,
        }
    }

    fn opening_intro_lines(&self) -> [&'static str; 5] {
        [
            self.tr(
                "You walk down a narrow hallway toward a door.",
                "Avanzas por un pasillo estrecho hacia una puerta.",
            ),
            self.tr("You enter through the door.", "Cruzas la puerta."),
            self.tr(
                "The door locks behind you.",
                "La puerta se cierra con llave detrás de ti.",
            ),
            self.tr(
                "You find yourself in a cavernous room with metal walls.",
                "Te encuentras en una sala cavernosa con muros de metal.",
            ),
            self.tr("Three doors lie ahead.", "Hay tres puertas delante."),
        ]
    }

    fn opening_intro_total_duration_ms(&self) -> f32 {
        self.opening_intro_lines().iter().fold(0.0, |total, _line| {
            total + OPENING_INTRO_LINE_FADE_MS + OPENING_INTRO_LINE_PAUSE_MS
        })
    }

    fn opening_intro_progress(&self) -> OpeningIntroProgress {
        let lines = self.opening_intro_lines();
        let Some(state) = self.opening_intro.as_ref() else {
            return OpeningIntroProgress {
                line_alphas: Vec::new(),
                complete: true,
            };
        };

        let mut remaining_ms = state
            .elapsed_ms
            .clamp(0.0, self.opening_intro_total_duration_ms());
        let mut line_alphas = Vec::with_capacity(lines.len());
        for _line in lines.iter() {
            if remaining_ms < OPENING_INTRO_LINE_FADE_MS {
                line_alphas.push((remaining_ms / OPENING_INTRO_LINE_FADE_MS).clamp(0.0, 1.0));
                return OpeningIntroProgress {
                    line_alphas,
                    complete: false,
                };
            }

            line_alphas.push(1.0);
            remaining_ms -= OPENING_INTRO_LINE_FADE_MS;
            if remaining_ms < OPENING_INTRO_LINE_PAUSE_MS {
                return OpeningIntroProgress {
                    line_alphas,
                    complete: false,
                };
            }
            remaining_ms -= OPENING_INTRO_LINE_PAUSE_MS;
        }

        OpeningIntroProgress {
            line_alphas,
            complete: true,
        }
    }

    fn opening_intro_complete(&self) -> bool {
        self.opening_intro_progress().complete
    }

    fn opening_intro_button_transition_progress(&self) -> f32 {
        if !self.opening_intro_complete() {
            return 0.0;
        }

        self.opening_intro
            .as_ref()
            .map(|opening_intro| {
                (opening_intro.button_transition_ms / OPENING_INTRO_BUTTON_TRANSITION_MS)
                    .clamp(0.0, 1.0)
            })
            .unwrap_or(1.0)
    }

    fn opening_intro_action_button(&self) -> FittedPrimaryButton {
        let (pad_x, pad_y) = boot_button_tile_padding();
        let font_size = START_BUTTON_FONT_SIZE;
        let (skip_w, skip_h) = button_size(
            self.tr("Skip Intro", "Saltar intro"),
            font_size,
            pad_x,
            pad_y,
        );
        let (continue_w, continue_h) =
            button_size(self.tr("Continue", "Continuar"), font_size, pad_x, pad_y);
        let transition = self.opening_intro_button_transition_progress();
        let w = lerp_f32(skip_w, continue_w, transition);
        let h = lerp_f32(skip_h, continue_h, transition);
        FittedPrimaryButton {
            rect: Rect {
                x: self.logical_center_x() - w * 0.5,
                y: (self.logical_height() - h - pad_y).max(0.0),
                w,
                h,
            },
            font_size,
        }
    }

    fn visible_combat_hand_count(&self) -> usize {
        if self.combat_feedback.playback_kind == Some(CombatPlaybackKind::EnemyTurn) {
            0
        } else {
            self.combat.hand_len()
        }
    }

    fn base_layout(&self) -> Layout {
        let hand_count = self.visible_combat_hand_count();
        let boot_buttons = self.boot_buttons_layout(self.has_saved_run);
        let layout_context = CombatLayoutContext {
            tile_gap: HAND_MIN_GAP,
            start_button: boot_buttons.start_button,
            restart_button: boot_buttons.restart_button,
            clear_save_button: boot_buttons.clear_save_button,
        };
        let layout_plan = self.best_combat_layout_plan(hand_count, layout_context);

        self.layout_with_scale(
            &layout_plan.hand,
            &layout_plan.enemies,
            layout_plan.low_hand_layout,
            layout_plan.tile_scale,
            layout_context,
        )
    }

    fn layout_target(&self) -> Layout {
        self.base_layout()
    }

    fn refresh_combat_layout_transition(&mut self) {
        if !matches!(self.screen, AppScreen::Combat) {
            self.combat_layout_target = None;
            return;
        }

        let target_layout = self.layout_target();
        let Some(previous_target) = self.combat_layout_target.clone() else {
            self.combat_layout_target = Some(target_layout);
            return;
        };
        if combat_layouts_match(&previous_target, &target_layout) {
            self.combat_layout_target = Some(target_layout);
            return;
        }

        let stable_hand_count = previous_target.hand_rects.len() == target_layout.hand_rects.len();
        let stable_enemy_count =
            previous_target.enemy_indices.len() == target_layout.enemy_indices.len();
        if stable_hand_count && stable_enemy_count {
            let from_layout = self
                .layout_transition
                .as_ref()
                .map(interpolated_transition_layout)
                .unwrap_or_else(|| previous_target.clone());
            let hand_from_rects = target_layout
                .hand_rects
                .iter()
                .enumerate()
                .map(|(index, _)| from_layout.hand_rects.get(index).copied())
                .collect();
            self.set_layout_transition(from_layout, target_layout.clone(), hand_from_rects);
        }

        self.combat_layout_target = Some(target_layout);
    }

    fn snapshot_combat_layout_target(&mut self) {
        if matches!(self.screen, AppScreen::Combat) && self.layout_transition.is_none() {
            self.combat_layout_target = Some(self.layout_target());
        }
    }

    fn layout(&self) -> Layout {
        if !matches!(self.screen, AppScreen::Combat) {
            return self.layout_target();
        }

        let Some(transition) = self.layout_transition.as_ref() else {
            return self.layout_target();
        };
        interpolated_transition_layout(transition)
    }

    fn layout_with_scale(
        &self,
        hand_arrangement: &CombatGridArrangement,
        enemy_arrangement: &CombatGridArrangement,
        low_hand_layout: bool,
        tile_scale: f32,
        layout_context: CombatLayoutContext,
    ) -> Layout {
        let hand_count = self.visible_combat_hand_count();
        let logical_width = self.logical_width();
        let tile_gap = layout_context.tile_gap;
        let hand_band_x = tile_gap;
        let hand_band_w = logical_width - hand_band_x * 2.0;
        debug_assert_eq!(hand_arrangement.item_count(), hand_count);
        let card_w = combat_hand_card_width(hand_arrangement, low_hand_layout, tile_scale);
        let tile_insets = tile_insets_for_card_width(card_w);
        let top_button_font_size = combat_action_button_font_size(low_hand_layout, tile_scale);
        let (top_button_pad_x, top_button_pad_y) = combat_action_button_padding(tile_insets);
        let top_button_gap = tile_gap;
        let top_button_y = tile_gap;
        let menu_size = button_size(
            self.tr("Menu", "Menú"),
            top_button_font_size,
            top_button_pad_x,
            top_button_pad_y,
        );
        let end_turn_size = button_size(
            self.tr("End Turn", "Fin del turno"),
            top_button_font_size,
            top_button_pad_x,
            top_button_pad_y,
        );
        let end_battle_size = self.debug_mode.then(|| {
            button_size(
                self.tr("End Battle", "Fin de batalla"),
                top_button_font_size,
                top_button_pad_x,
                top_button_pad_y,
            )
        });
        let top_group_w = menu_size.0
            + top_button_gap
            + end_turn_size.0
            + end_battle_size
                .map(|size| top_button_gap + size.0)
                .unwrap_or(0.0);
        let top_group_x = (logical_width - top_group_w) * 0.5;
        let top_row_h = menu_size.1.max(end_turn_size.1);
        let visible_enemy_indices = self.visible_enemy_indices();
        debug_assert_eq!(enemy_arrangement.item_count(), visible_enemy_indices.len());
        let enemy_metrics: Vec<_> = visible_enemy_indices
            .iter()
            .map(|enemy_index| {
                enemy_panel_metrics(self, *enemy_index, low_hand_layout, tile_scale, tile_insets)
            })
            .collect();
        let player_metrics = player_panel_metrics(self, low_hand_layout, tile_scale, tile_insets);
        let enemy_y = top_button_y + top_row_h + tile_gap;
        let player_x = (logical_width - player_metrics.width) * 0.5;
        let card_heights: Vec<f32> = (0..hand_count)
            .map(|index| {
                self.combat
                    .hand_card(index)
                    .map(|card| {
                        let def = self.localized_card_def(card);
                        let description = self.combat_card_description(card);
                        card_content_height_with_description(def, &description, card_w)
                    })
                    .unwrap_or(card_w * (CARD_HEIGHT / CARD_WIDTH))
            })
            .collect();
        let mut enemy_rects = Vec::with_capacity(enemy_metrics.len());
        let mut enemy_item_index = 0usize;
        let mut enemy_row_top = enemy_y;
        for &row_count in &enemy_arrangement.row_counts {
            let row_end = enemy_item_index + row_count;
            let row_metrics = &enemy_metrics[enemy_item_index..row_end];
            let row_height = row_metrics
                .iter()
                .map(|metrics| metrics.height)
                .fold(0.0, f32::max);
            let row_width = row_metrics.iter().map(|metrics| metrics.width).sum::<f32>()
                + tile_gap * row_count.saturating_sub(1) as f32;
            let mut enemy_cursor_x = (logical_width - row_width) * 0.5;
            for metrics in row_metrics {
                enemy_rects.push(Rect {
                    x: enemy_cursor_x,
                    y: enemy_row_top + (row_height - metrics.height) * 0.5,
                    w: metrics.width,
                    h: metrics.height,
                });
                enemy_cursor_x += metrics.width + tile_gap;
            }
            enemy_item_index = row_end;
            enemy_row_top += row_height + tile_gap;
        }
        let enemy_bottom = if enemy_arrangement.is_empty() {
            enemy_y
        } else {
            enemy_row_top - tile_gap
        };

        let mut hand_rects = Vec::with_capacity(hand_count);
        let mut hand_item_index = 0usize;
        let mut hand_row_top = enemy_bottom + tile_gap;
        for &row_count in &hand_arrangement.row_counts {
            let row_end = hand_item_index + row_count;
            let row_heights = &card_heights[hand_item_index..row_end];
            let row_max_h = row_heights.iter().copied().fold(0.0, f32::max);
            let row_total_w =
                card_w * row_count as f32 + tile_gap * row_count.saturating_sub(1) as f32;
            let row_start_x = hand_band_x + (hand_band_w - row_total_w) * 0.5;
            for (index_in_row, card_h) in row_heights.iter().enumerate() {
                hand_rects.push(Rect {
                    x: row_start_x + (card_w + tile_gap) * index_in_row as f32,
                    y: hand_row_top + (row_max_h - *card_h) * 0.5,
                    w: card_w,
                    h: *card_h,
                });
            }
            hand_item_index = row_end;
            hand_row_top += row_max_h + tile_gap;
        }
        let hand_bottom = if hand_arrangement.is_empty() {
            enemy_bottom
        } else {
            hand_row_top - tile_gap
        };
        let player_y = hand_bottom + tile_gap;
        let (hint_message, _, _) = combat_hint_tile(self, hand_count);
        let (hint_font_size, hint_pad_x, hint_pad_y) = hand_hint_metrics(tile_scale);
        let hint_w = text_width(&hint_message, hint_font_size) + hint_pad_x * 2.0;
        let hint_h = hint_font_size + hint_pad_y * 2.0;
        let hint_rect = Some(Rect {
            x: (logical_width - hint_w) * 0.5,
            y: player_y + player_metrics.height + tile_gap,
            w: hint_w,
            h: hint_h,
        });

        let mut layout = Layout {
            start_button: layout_context.start_button,
            restart_button: layout_context.restart_button,
            clear_save_button: layout_context.clear_save_button,
            menu_button: Rect {
                x: top_group_x,
                y: top_button_y,
                w: menu_size.0,
                h: menu_size.1,
            },
            end_turn_button: Rect {
                x: top_group_x + menu_size.0 + top_button_gap,
                y: top_button_y,
                w: end_turn_size.0,
                h: end_turn_size.1,
            },
            end_battle_button: end_battle_size.map(|size| Rect {
                x: top_group_x + menu_size.0 + top_button_gap + end_turn_size.0 + top_button_gap,
                y: top_button_y,
                w: size.0,
                h: size.1,
            }),
            enemy_indices: visible_enemy_indices,
            enemy_arrangement: enemy_arrangement.clone(),
            enemy_rects,
            player_rect: Rect {
                x: player_x,
                y: player_y,
                w: player_metrics.width,
                h: player_metrics.height,
            },
            hand_arrangement: hand_arrangement.clone(),
            hand_rects,
            hint_rect,
            low_hand_layout,
            tile_scale,
            tile_insets,
        };
        let combat_bounds = combat_layout_bounds(&layout);
        let offset_y = ((self.logical_height() - combat_bounds.h) * 0.5) - combat_bounds.y;
        layout.menu_button.y += offset_y;
        layout.end_turn_button.y += offset_y;
        if let Some(rect) = &mut layout.end_battle_button {
            rect.y += offset_y;
        }
        for rect in &mut layout.enemy_rects {
            rect.y += offset_y;
        }
        layout.player_rect.y += offset_y;
        for rect in &mut layout.hand_rects {
            rect.y += offset_y;
        }
        if let Some(rect) = &mut layout.hint_rect {
            rect.y += offset_y;
        }

        layout
    }

    fn best_combat_layout_plan(
        &self,
        hand_count: usize,
        layout_context: CombatLayoutContext,
    ) -> CombatLayoutPlan {
        let enemy_count = self.visible_enemy_indices().len();
        let mut best_plan = None;

        for hand in combat_grid_arrangement_candidates(hand_count, MAX_COMBAT_HAND_ROWS) {
            let low_hand_layout = hand_count <= LOW_HAND_MAX_COUNT;
            for enemies in combat_grid_arrangement_candidates(enemy_count, MAX_COMBAT_ENEMY_ROWS) {
                let tile_scale =
                    self.combat_tile_scale(&hand, &enemies, low_hand_layout, layout_context);
                let fits = self.combat_tiles_fit(
                    &hand,
                    &enemies,
                    low_hand_layout,
                    tile_scale,
                    layout_context,
                );
                let candidate = CombatLayoutPlan {
                    hand: hand.clone(),
                    enemies: enemies.clone(),
                    low_hand_layout,
                    tile_scale,
                    score: CombatLayoutScore {
                        fits,
                        hand_card_w: combat_hand_card_width(&hand, low_hand_layout, tile_scale),
                        tile_scale,
                    },
                };
                let should_replace = match best_plan.as_ref() {
                    None => true,
                    Some(best) => combat_layout_plan_better(&candidate, hand_count, best),
                };
                if should_replace {
                    best_plan = Some(candidate);
                }
            }
        }

        best_plan.expect("combat layout should produce at least one arrangement")
    }

    fn combat_tile_scale(
        &self,
        hand_arrangement: &CombatGridArrangement,
        enemy_arrangement: &CombatGridArrangement,
        low_hand_layout: bool,
        layout_context: CombatLayoutContext,
    ) -> f32 {
        let logical_width = self.logical_width();
        let logical_height = self.logical_height();
        let fits = |scale| {
            self.combat_tiles_fit(
                hand_arrangement,
                enemy_arrangement,
                low_hand_layout,
                scale,
                layout_context,
            )
        };

        let mut low = MIN_COMBAT_TILE_SCALE;
        if !fits(low) {
            return low;
        }

        let max_search_scale = (logical_width / CARD_WIDTH).max(logical_height / CARD_HEIGHT) * 2.0;
        let available_w = logical_width - layout_context.tile_gap * 2.0;
        let available_h = logical_height - layout_context.tile_gap * 2.0;
        let initial_layout = self.layout_with_scale(
            hand_arrangement,
            enemy_arrangement,
            low_hand_layout,
            1.0,
            layout_context,
        );
        let initial_bounds = combat_layout_bounds(&initial_layout);
        let mut high = if initial_bounds.w > 0.0 && initial_bounds.h > 0.0 {
            let max_search_scale = max_search_scale.max(1.0);
            (available_w / initial_bounds.w)
                .min(available_h / initial_bounds.h)
                .clamp(1.0, max_search_scale)
        } else {
            1.0
        };
        while high < max_search_scale && fits(high) {
            low = high;
            high = (high * 1.25).min(max_search_scale);
            if (high - low).abs() < f32::EPSILON {
                return high;
            }
        }

        if fits(high) {
            return high;
        }

        for _ in 0..24 {
            let mid = (low + high) * 0.5;
            if fits(mid) {
                low = mid;
            } else {
                high = mid;
            }
        }

        low
    }

    fn boot_buttons_layout(&self, has_saved_run: bool) -> BootButtonsLayout {
        let center_x = self.logical_center_x();
        let hero = boot_hero_layout(self.logical_width(), self.logical_height());
        let (start_pad_x, start_pad_y) = boot_button_tile_padding();
        let primary_label = if has_saved_run {
            self.tr(BOOT_CONTINUE_LABEL, "Continuar")
        } else {
            self.tr("Start", "Empezar")
        };
        let start_button = centered_button_rect(
            primary_label,
            START_BUTTON_FONT_SIZE,
            start_pad_x,
            start_pad_y,
            center_x,
            hero.start_button_y,
        );
        let (restart_pad_x, restart_pad_y) = boot_button_tile_padding();
        let restart_button = centered_button_rect(
            self.tr(BOOT_RESTART_LABEL, "Reiniciar"),
            START_BUTTON_FONT_SIZE,
            restart_pad_x,
            restart_pad_y,
            center_x,
            start_button.y + start_button.h + 18.0,
        );
        let settings_button = centered_button_rect(
            self.tr(BOOT_SETTINGS_LABEL, "Ajustes"),
            START_BUTTON_FONT_SIZE,
            restart_pad_x,
            restart_pad_y,
            center_x,
            if has_saved_run {
                restart_button.y + restart_button.h + 18.0
            } else {
                start_button.y + start_button.h + 18.0
            },
        );
        let install_button = self.install_capability.button_visible().then(|| {
            centered_button_rect(
                self.tr(BOOT_INSTALL_LABEL, "Instalar"),
                START_BUTTON_FONT_SIZE,
                restart_pad_x,
                restart_pad_y,
                center_x,
                settings_button.y + settings_button.h + 18.0,
            )
        });
        let update_button = self.update_available.then(|| {
            centered_button_rect(
                self.tr(BOOT_UPDATE_LABEL, "Actualizar"),
                START_BUTTON_FONT_SIZE,
                restart_pad_x,
                restart_pad_y,
                center_x,
                install_button
                    .map(|button| button.y + button.h + 18.0)
                    .unwrap_or(settings_button.y + settings_button.h + 18.0),
            )
        });
        let clear_save_button = (self.debug_mode && has_saved_run).then(|| {
            centered_button_rect(
                self.tr(BOOT_DEBUG_CLEAR_LABEL, "Reset"),
                START_BUTTON_FONT_SIZE,
                RESET_BUTTON_PAD_X,
                RESET_BUTTON_PAD_Y,
                center_x,
                update_button
                    .map(|button| button.y + button.h + 18.0)
                    .or_else(|| install_button.map(|button| button.y + button.h + 18.0))
                    .unwrap_or(settings_button.y + settings_button.h + 18.0),
            )
        });

        BootButtonsLayout {
            start_button,
            restart_button,
            settings_button,
            install_button,
            update_button,
            clear_save_button,
        }
    }

    fn combat_tiles_fit(
        &self,
        hand_arrangement: &CombatGridArrangement,
        enemy_arrangement: &CombatGridArrangement,
        low_hand_layout: bool,
        tile_scale: f32,
        layout_context: CombatLayoutContext,
    ) -> bool {
        let layout = self.layout_with_scale(
            hand_arrangement,
            enemy_arrangement,
            low_hand_layout,
            tile_scale,
            layout_context,
        );
        let bounds = combat_layout_bounds(&layout);
        let min_edge = layout_context.tile_gap - 0.5;
        let max_x = self.logical_width() - layout_context.tile_gap + 0.5;
        let max_y = self.logical_height() - layout_context.tile_gap + 0.5;
        bounds.x >= min_edge
            && bounds.y >= min_edge
            && bounds.x + bounds.w <= max_x
            && bounds.y + bounds.h <= max_y
    }

    fn map_layout(&self) -> Option<MapLayout> {
        let dungeon = self.dungeon.as_ref()?;
        let logical_width = self.logical_width();
        let logical_height = self.logical_height();
        let tile_insets = tile_insets_for_card_width(180.0);
        let menu_font_size = 20.0;
        let top_row_y = HAND_MIN_GAP;
        let (menu_w, menu_h) = button_size(
            self.tr("Menu", "Menú"),
            menu_font_size,
            tile_insets.pad_x,
            tile_insets.top_pad,
        );
        let (info_w, info_h) = button_size(
            self.tr(MAP_INFO_LABEL, "Info"),
            menu_font_size,
            tile_insets.pad_x,
            tile_insets.top_pad,
        );
        let (legend_w, legend_h) = button_size(
            self.tr(LEGEND_BUTTON_LABEL, "Leyenda"),
            menu_font_size,
            tile_insets.pad_x,
            tile_insets.top_pad,
        );
        let top_group_w = menu_w + HAND_MIN_GAP + info_w + HAND_MIN_GAP + legend_w;
        let top_group_x = (logical_width - top_group_w) * 0.5;
        let top_bar_h = menu_h.max(info_h).max(legend_h);
        let top_bar_center_y = top_row_y + top_bar_h * 0.5;
        let debug_controls = self.debug_mode.then(|| {
            let (button_w, button_h) = button_size(
                "<",
                MAP_DEBUG_BUTTON_FONT_SIZE,
                MAP_DEBUG_BUTTON_PAD_X,
                MAP_DEBUG_BUTTON_PAD_Y,
            );
            let label_w = text_width(
                &debug_map_label(dungeon, self.language),
                MAP_DEBUG_SEED_SIZE,
            );
            let group_w = label_w + MAP_DEBUG_BUTTON_GAP + button_w * 2.0 + MAP_DEBUG_BUTTON_GAP;
            let debug_row_y = top_row_y + top_bar_h + HAND_MIN_GAP;
            let debug_center_y = debug_row_y + button_h * 0.5;
            let group_x = (logical_width - group_w) * 0.5;
            let debug_text_x = group_x + button_w + MAP_DEBUG_BUTTON_GAP + label_w * 0.5;
            let down_button = Rect {
                x: group_x,
                y: debug_row_y,
                w: button_w,
                h: button_h,
            };
            let up_button = Rect {
                x: group_x + button_w + MAP_DEBUG_BUTTON_GAP + label_w + MAP_DEBUG_BUTTON_GAP,
                y: debug_row_y,
                w: button_w,
                h: button_h,
            };
            let fill_label = self.tr(MAP_DEBUG_FILL_DECK_LABEL, "Llenar mazo");
            let (fill_button_w, fill_button_h) = button_size(
                fill_label,
                MAP_DEBUG_BUTTON_FONT_SIZE,
                MAP_DEBUG_BUTTON_PAD_X,
                MAP_DEBUG_BUTTON_PAD_Y,
            );
            let fill_button = Rect {
                x: (logical_width - fill_button_w) * 0.5,
                y: debug_row_y + button_h + HAND_MIN_GAP,
                w: fill_button_w,
                h: fill_button_h,
            };
            let group_h = button_h + HAND_MIN_GAP + fill_button_h;
            (
                down_button,
                up_button,
                (debug_text_x, debug_center_y),
                fill_button,
                group_h,
            )
        });
        let menu_button = Rect {
            x: top_group_x,
            y: top_bar_center_y - menu_h * 0.5,
            w: menu_w,
            h: menu_h,
        };
        let info_button = Rect {
            x: menu_button.x + menu_button.w + HAND_MIN_GAP,
            y: top_bar_center_y - info_h * 0.5,
            w: info_w,
            h: info_h,
        };
        let legend_button = Rect {
            x: info_button.x + info_button.w + HAND_MIN_GAP,
            y: top_bar_center_y - legend_h * 0.5,
            w: legend_w,
            h: legend_h,
        };
        let top_block_h = top_bar_h
            + debug_controls
                .map(|(_, _, _, _, group_h)| HAND_MIN_GAP + group_h)
                .unwrap_or(0.0);
        let map_top = top_row_y + top_block_h + 48.0;
        let map_bottom = logical_height - HAND_MIN_GAP - MAP_NODE_RADIUS;
        let row_count = dungeon.max_depth() + 1;
        let row_gap = if row_count > 1 {
            (map_bottom - map_top) / (row_count - 1) as f32
        } else {
            0.0
        };
        let side_pad = (logical_width * 0.12).clamp(54.0, 132.0);
        let lane_span = (logical_width - side_pad * 2.0).max(0.0);
        let lane_spacing = if dungeon.lane_count() > 1 {
            (lane_span / (dungeon.lane_count() - 1) as f32)
                .min(MAP_MAX_ADJACENT_LANE_CENTER_SPACING)
        } else {
            0.0
        };
        let (occupied_min_lane, occupied_max_lane) = dungeon
            .nodes
            .iter()
            .filter(|node| !matches!(node.kind, RoomKind::Start | RoomKind::Boss))
            .fold((usize::MAX, 0usize), |(min_lane, max_lane), node| {
                (min_lane.min(node.lane), max_lane.max(node.lane))
            });
        let occupied_lane_center = if occupied_min_lane == usize::MAX {
            0.0
        } else {
            (occupied_min_lane as f32 + occupied_max_lane as f32) * 0.5
        };

        let nodes: Vec<MapNodeLayout> = dungeon
            .nodes
            .iter()
            .map(|node| {
                let center_x = if matches!(node.kind, RoomKind::Start | RoomKind::Boss)
                    || lane_spacing <= 0.0
                {
                    logical_width * 0.5
                } else {
                    logical_width * 0.5 + (node.lane as f32 - occupied_lane_center) * lane_spacing
                };
                let center_y = map_bottom - node.depth as f32 * row_gap;
                MapNodeLayout {
                    id: node.id,
                    rect: Rect {
                        x: center_x - MAP_NODE_DIAMETER * 0.5,
                        y: center_y - MAP_NODE_DIAMETER * 0.5,
                        w: MAP_NODE_DIAMETER,
                        h: MAP_NODE_DIAMETER,
                    },
                    center_x,
                    center_y,
                }
            })
            .collect();

        let mut edges = Vec::new();
        for dungeon_node in &dungeon.nodes {
            let Some(from) = nodes.get(dungeon_node.id).copied() else {
                continue;
            };
            for next_id in &dungeon_node.next {
                let Some(to) = nodes.get(*next_id).copied() else {
                    continue;
                };
                edges.push(MapEdgeLayout {
                    from_id: dungeon_node.id,
                    to_id: *next_id,
                    from_x: from.center_x,
                    from_y: from.center_y,
                    to_x: to.center_x,
                    to_y: to.center_y,
                });
            }
        }

        let legend_insets = tile_insets_for_card_width(220.0);
        let legend_title_size = 24.0;
        let legend_label_size = 20.0;
        let legend_symbol_radius = 10.0;
        let legend_symbol_gap = 16.0;
        let legend_title_gap = 16.0;
        let legend_row_gap = 12.0;
        let legend_entries = map_legend_entries(self.language);
        let legend_label_w = legend_entries
            .iter()
            .map(|(_, label)| text_width(label, legend_label_size))
            .fold(0.0, f32::max);
        let legend_modal = Rect {
            x: (logical_width
                - (legend_insets.pad_x * 2.0
                    + legend_symbol_radius * 2.0
                    + legend_symbol_gap
                    + legend_label_w))
                * 0.5,
            y: (logical_height
                - (legend_insets.top_pad
                    + legend_title_size
                    + legend_title_gap
                    + legend_entries.len() as f32 * legend_label_size
                    + (legend_entries.len().saturating_sub(1) as f32 * legend_row_gap)
                    + legend_insets.bottom_pad))
                * 0.5,
            w: legend_insets.pad_x * 2.0
                + legend_symbol_radius * 2.0
                + legend_symbol_gap
                + legend_label_w,
            h: legend_insets.top_pad
                + legend_title_size
                + legend_title_gap
                + legend_entries.len() as f32 * legend_label_size
                + (legend_entries.len().saturating_sub(1) as f32 * legend_row_gap)
                + legend_insets.bottom_pad,
        };

        Some(MapLayout {
            menu_button,
            info_button,
            legend_button,
            legend_modal,
            debug_level_down_button: debug_controls.map(|(down, _, _, _, _)| down),
            debug_level_up_button: debug_controls.map(|(_, up, _, _, _)| up),
            debug_level_text_position: debug_controls.map(|(_, _, text, _, _)| text),
            debug_fill_deck_button: debug_controls.map(|(_, _, _, fill_button, _)| fill_button),
            nodes,
            edges,
        })
    }

    fn restart_confirm_layout(&self) -> Option<RestartConfirmLayout> {
        if !self.has_saved_run {
            return None;
        }

        let logical_width = self.logical_width();
        let pad_x = 24.0;
        let top_pad = 24.0;
        let bottom_pad = 20.0;
        let title_size = 26.0;
        let title_gap = 8.0;
        let title_max_w = (logical_width - 96.0).clamp(180.0, 360.0);
        let title_chars = ((title_max_w / (title_size * 0.62)).floor().max(10.0)) as usize;
        let title_lines = wrap_text(
            self.tr(BOOT_RESTART_CONFIRM_TITLE, "¿Seguro que quieres reiniciar?"),
            title_chars,
        );
        let title_block_w = title_lines
            .iter()
            .map(|line| text_width(line, title_size))
            .fold(0.0, f32::max);
        let title_block_h = if title_lines.is_empty() {
            title_size
        } else {
            title_lines.len() as f32 * title_size
                + title_gap * title_lines.len().saturating_sub(1) as f32
        };
        let modal_w = fit_modal_width(title_block_w + pad_x * 2.0, logical_width, 320.0);
        let button_metrics = fit_overlay_button_metrics(
            &[
                self.tr(BOOT_RESTART_CONFIRM_CANCEL_LABEL, "Cancelar"),
                self.tr(BOOT_RESTART_LABEL, "Reiniciar"),
            ],
            modal_w - pad_x * 2.0,
        );
        let modal_h = top_pad + title_block_h + 32.0 + button_metrics.block_h + bottom_pad;
        let modal_rect = Rect {
            x: (logical_width - modal_w) * 0.5,
            y: (self.logical_height() - modal_h) * 0.5,
            w: modal_w,
            h: modal_h,
        };
        let buttons = place_overlay_buttons(&button_metrics, modal_rect, bottom_pad);

        Some(RestartConfirmLayout {
            modal_rect,
            cancel_button: buttons[0],
            restart_button: buttons[1],
            title_lines,
            title_size,
        })
    }

    fn settings_layout(&self) -> SettingsLayout {
        let logical_width = self.logical_width();
        let logical_height = self.logical_height();
        let pad_x = 24.0;
        let top_pad = 24.0;
        let bottom_pad = 20.0;
        let title = self.tr("Settings", "Ajustes");
        let subtitle = self.tr("Choose the game language.", "Elige el idioma del juego.");
        let title_size = 26.0;
        let subtitle_size = 18.0;
        let title_max_w = (logical_width - 96.0).clamp(180.0, 360.0);
        let title_chars = ((title_max_w / (title_size * 0.62)).floor().max(10.0)) as usize;
        let subtitle_chars = ((title_max_w / (subtitle_size * 0.62)).floor().max(12.0)) as usize;
        let title_lines = wrap_text(title, title_chars);
        let subtitle_lines = wrap_text(subtitle, subtitle_chars);
        let title_block_w = title_lines
            .iter()
            .map(|line| text_width(line, title_size))
            .fold(0.0, f32::max);
        let subtitle_block_w = subtitle_lines
            .iter()
            .map(|line| text_width(line, subtitle_size))
            .fold(0.0, f32::max);
        let title_gap = 8.0;
        let subtitle_gap = 6.0;
        let title_block_h = if title_lines.is_empty() {
            title_size
        } else {
            title_lines.len() as f32 * title_size
                + title_gap * title_lines.len().saturating_sub(1) as f32
        };
        let subtitle_block_h = if subtitle_lines.is_empty() {
            subtitle_size
        } else {
            subtitle_lines.len() as f32 * subtitle_size
                + subtitle_gap * subtitle_lines.len().saturating_sub(1) as f32
        };
        let modal_w = fit_modal_width(
            title_block_w.max(subtitle_block_w) + pad_x * 2.0,
            logical_width,
            340.0,
        );
        let button_metrics =
            fit_overlay_button_metrics(&["English", "Español"], modal_w - pad_x * 2.0);
        let modal_h = top_pad
            + title_block_h
            + 14.0
            + subtitle_block_h
            + 32.0
            + button_metrics.block_h
            + bottom_pad;
        let modal_rect = Rect {
            x: (logical_width - modal_w) * 0.5,
            y: (logical_height - modal_h) * 0.5,
            w: modal_w,
            h: modal_h,
        };
        let buttons = place_overlay_buttons(&button_metrics, modal_rect, bottom_pad);

        SettingsLayout {
            modal_rect,
            english_button: buttons[0],
            spanish_button: buttons[1],
            title_lines,
            subtitle_lines,
            title_size,
            subtitle_size,
        }
    }

    fn install_help_layout(&self) -> InstallHelpLayout {
        let logical_width = self.logical_width();
        let logical_height = self.logical_height();
        let pad_x = 24.0;
        let top_pad = 24.0;
        let bottom_pad = 20.0;
        let title = self.tr("Install Mazocarta", "Instalar Mazocarta");
        let instructions = match self.language {
            Language::English => [
                "Open this page in Safari.",
                "Tap Share.",
                "Choose Add to Home Screen.",
            ],
            Language::Spanish => [
                "Abre esta pagina en Safari.",
                "Toca Compartir.",
                "Elige Anadir a pantalla de inicio.",
            ],
        };
        let title_size = 26.0;
        let body_size = 18.0;
        let text_max_w = (logical_width - 96.0).clamp(180.0, 420.0);
        let title_chars = ((text_max_w / (title_size * 0.62)).floor().max(10.0)) as usize;
        let body_chars = ((text_max_w / (body_size * 0.62)).floor().max(14.0)) as usize;
        let title_lines = wrap_text(title, title_chars);
        let mut body_lines = Vec::new();
        for instruction in instructions {
            body_lines.extend(wrap_text(instruction, body_chars));
        }
        let title_gap = 8.0;
        let body_gap = 6.0;
        let title_block_w = title_lines
            .iter()
            .map(|line| text_width(line, title_size))
            .fold(0.0, f32::max);
        let body_block_w = body_lines
            .iter()
            .map(|line| text_width(line, body_size))
            .fold(0.0, f32::max);
        let title_block_h = if title_lines.is_empty() {
            title_size
        } else {
            title_lines.len() as f32 * title_size
                + title_gap * title_lines.len().saturating_sub(1) as f32
        };
        let body_block_h = if body_lines.is_empty() {
            body_size
        } else {
            body_lines.len() as f32 * body_size
                + body_gap * body_lines.len().saturating_sub(1) as f32
        };
        let modal_w = fit_modal_width(
            title_block_w.max(body_block_w) + pad_x * 2.0,
            logical_width,
            340.0,
        );
        let button_metrics =
            fit_overlay_button_metrics(&[self.tr("Close", "Cerrar")], modal_w - pad_x * 2.0);
        let modal_h = top_pad
            + title_block_h
            + 16.0
            + body_block_h
            + 28.0
            + button_metrics.block_h
            + bottom_pad;
        let modal_rect = Rect {
            x: (logical_width - modal_w) * 0.5,
            y: (logical_height - modal_h) * 0.5,
            w: modal_w,
            h: modal_h,
        };
        let buttons = place_overlay_buttons(&button_metrics, modal_rect, bottom_pad);

        InstallHelpLayout {
            modal_rect,
            close_button: buttons[0],
            title_lines,
            body_lines,
            title_size,
            body_size,
        }
    }

    fn module_select_layout(&self) -> Option<ModuleSelectLayout> {
        let module_select = self.module_select.as_ref()?;
        if module_select.options.is_empty() {
            return None;
        }

        let logical_width = self.logical_width();
        let logical_height = self.logical_height();
        let title = self.module_select_title();
        let title_size = fit_text_size(title, 40.0, (logical_width - 48.0).max(120.0)).max(24.0);
        let gap = HAND_MIN_GAP;
        let side_margin = 24.0;
        let title_gap = 30.0;
        let card_w = (logical_width - side_margin * 2.0).clamp(300.0, 430.0);
        let total_cards_h = module_select
            .options
            .iter()
            .copied()
            .map(|module| module_content_height(self.localized_module_def(module), card_w))
            .fold(0.0, |height, card_h| {
                if height == 0.0 {
                    card_h
                } else {
                    height + gap + card_h
                }
            });

        let total_stack_h = title_size + title_gap + total_cards_h;
        let stack_top = (logical_height - total_stack_h) * 0.5;
        let title_y = stack_top + title_size;
        let mut row_top = stack_top + title_size + title_gap;
        let mut card_rects = Vec::with_capacity(module_select.options.len());
        for module in module_select.options.iter().copied() {
            let card_h = module_content_height(self.localized_module_def(module), card_w);
            card_rects.push(Rect {
                x: (logical_width - card_w) * 0.5,
                y: row_top,
                w: card_w,
                h: card_h,
            });
            row_top += card_h + gap;
        }

        Some(ModuleSelectLayout {
            title_y,
            card_rects,
        })
    }

    fn event_layout(&self) -> Option<EventLayout> {
        let event = self.event.as_ref()?;
        let def = localized_event_def(event.event, self.language);
        let logical_width = self.logical_width();
        let logical_height = self.logical_height();
        let gap = HAND_MIN_GAP * 1.6;
        let title_size =
            fit_text_size(def.title, 40.0, (logical_width - 48.0).max(120.0)).max(24.0);
        let flavor_size =
            fit_text_size(def.flavor, 18.0, (logical_width - 48.0).max(120.0)).max(12.0);
        let flavor_max_w = (logical_width - 64.0).clamp(220.0, 760.0);
        let flavor_chars = ((flavor_max_w / (flavor_size * 0.58)).floor() as usize).max(18);
        let flavor_lines = wrap_text(def.flavor, flavor_chars);
        let flavor_gap = 12.0;
        let flavor_line_gap = 6.0;
        let flavor_height = if flavor_lines.is_empty() {
            0.0
        } else {
            flavor_size * flavor_lines.len() as f32
                + flavor_line_gap * flavor_lines.len().saturating_sub(1) as f32
        };
        let title_block_h = title_size + flavor_gap + flavor_height + 20.0;
        let row_counts: &[usize] = if logical_width >= 760.0 {
            &[2]
        } else {
            &[1, 1]
        };
        let max_columns = row_counts.iter().copied().max().unwrap_or(1);
        let card_w = ((logical_width - gap * (max_columns as f32 + 1.0)) / max_columns as f32)
            .clamp(220.0, 360.0);

        let mut row_heights = Vec::with_capacity(row_counts.len());
        let mut choice_index = 0usize;
        for &count in row_counts {
            let row_height = (0..count)
                .filter_map(|offset| {
                    let index = choice_index + offset;
                    Some(event_choice_content_height(
                        localized_event_choice_title(event.event, index, self.language)?,
                        &localized_event_choice_body(
                            event.event,
                            index,
                            self.dungeon.as_ref()?.current_level(),
                            self.language,
                        )?,
                        card_w,
                    ))
                })
                .fold(0.0, f32::max);
            row_heights.push(row_height);
            choice_index += count;
        }

        let total_choices_h =
            row_heights.iter().sum::<f32>() + gap * row_heights.len().saturating_sub(1) as f32;
        let total_stack_h = title_block_h + total_choices_h;
        let stack_top = ((logical_height - total_stack_h) * 0.5).max(gap);
        let title_y = stack_top + title_size;
        let mut choice_rects = Vec::with_capacity(2);
        let mut row_top = stack_top + title_block_h;
        let mut choice_index = 0usize;

        for (row_index, &count) in row_counts.iter().enumerate() {
            let row_width = count as f32 * card_w + gap * count.saturating_sub(1) as f32;
            let mut x = (logical_width - row_width) * 0.5;
            for _ in 0..count {
                let title =
                    match localized_event_choice_title(event.event, choice_index, self.language) {
                        Some(title) => title,
                        None => break,
                    };
                let body = match self.dungeon.as_ref().and_then(|dungeon| {
                    localized_event_choice_body(
                        event.event,
                        choice_index,
                        dungeon.current_level(),
                        self.language,
                    )
                }) {
                    Some(body) => body,
                    None => break,
                };
                let card_h = event_choice_content_height(title, &body, card_w);
                choice_rects.push(Rect {
                    x,
                    y: row_top + (row_heights[row_index] - card_h) * 0.5,
                    w: card_w,
                    h: card_h,
                });
                x += card_w + gap;
                choice_index += 1;
            }
            row_top += row_heights[row_index] + gap;
        }

        Some(EventLayout {
            title_y,
            choice_rects,
        })
    }

    fn run_info_layout(&self) -> Option<RunInfoLayout> {
        let dungeon = self.dungeon.as_ref()?;
        let logical_width = self.logical_width();
        let logical_height = self.logical_height();
        let title_size = 24.0;
        let row_size = 18.0_f32;
        let module_name_size = 18.0;
        let module_body_size = 16.0;
        let (pad_x, pad_y) = standard_overlay_padding();
        let module_wrap_side_pad = 14.0;
        let module_bottom_pad = pad_y;
        let module_title_top_gap = 10.0;
        let line_gap = 8.0_f32;
        let module_gap = 10.0;
        let title_gap = 34.0_f32;
        let modules_gap = (title_gap - (row_size + line_gap)).max(0.0_f32);
        let content_lines = [
            match self.language {
                Language::English => format!("Level {}", dungeon.current_level()),
                Language::Spanish => format!("Nivel {}", dungeon.current_level()),
            },
            format!("HP {}/{}", dungeon.player_hp, dungeon.player_max_hp),
            credits_label(dungeon.credits, self.language),
            card_deck_label(dungeon.deck.len(), self.language),
        ];
        let widest_summary = content_lines
            .iter()
            .map(|line| text_width(line, row_size))
            .fold(
                text_width(self.tr("Run Info", "Info de la Run"), title_size),
                f32::max,
            );
        let widest_module = dungeon
            .modules
            .iter()
            .map(|module| {
                let def = self.localized_module_def(*module);
                text_width(def.name, module_name_size)
            })
            .fold(text_width("No modules online.", module_body_size), f32::max);
        let modal_w = (widest_summary.max(widest_module).max(208.0) + pad_x * 2.0)
            .clamp(232.0, (logical_width - 48.0).max(232.0));
        let inner_w = (modal_w - module_wrap_side_pad * 2.0).max(136.0);
        let modules_block_h = self.run_info_modules_block_height(
            &dungeon.modules,
            inner_w,
            module_name_size,
            module_body_size,
            module_title_top_gap,
            module_gap,
        );
        let modal_h = (pad_y
            + title_size
            + title_gap
            + content_lines.len() as f32 * row_size
            + content_lines.len().saturating_sub(1) as f32 * line_gap
            + modules_gap
            + modules_block_h
            + module_bottom_pad)
            .clamp(250.0, (logical_height - 48.0).max(250.0));
        Some(RunInfoLayout {
            modal_rect: Rect {
                x: (logical_width - modal_w) * 0.5,
                y: (logical_height - modal_h) * 0.5,
                w: modal_w,
                h: modal_h,
            },
        })
    }

    fn enemy_inspect_layout(&self) -> Option<EnemyInspectLayout> {
        if !matches!(self.screen, AppScreen::Combat) {
            return None;
        }

        let enemy_index = self.ui.enemy_inspect_enemy?;
        let enemy = self.combat.enemy(enemy_index)?;
        let logical_width = self.logical_width();
        let logical_height = self.logical_height();
        let (pad_x, pad_y) = standard_overlay_padding();
        let title_gap = 26.0_f32;
        let desired_title_size = 24.0_f32;
        let name = localized_enemy_name(enemy.profile, self.language);
        let sprite = enemy_sprite_def(enemy.profile);
        let sprite_w = sprite.width.max(1) as f32;
        let sprite_h = sprite.height.max(1) as f32;
        let max_modal_h = (logical_height - 48.0).max(180.0);
        let desired_modal_w = text_width(name, desired_title_size).max(208.0) + pad_x * 2.0;
        let modal_w = fit_modal_width(desired_modal_w, logical_width, 232.0);
        let title_size =
            fit_text_size(name, desired_title_size, (modal_w - pad_x * 2.0).max(120.0)).max(18.0);
        let max_sprite_w = (modal_w - pad_x * 2.0).max(96.0);
        let max_sprite_h = (max_modal_h - pad_y * 2.0 - title_size - title_gap).max(96.0);
        let sprite_scale = (max_sprite_w / sprite_w).min(max_sprite_h / sprite_h);
        let draw_w = sprite_w * sprite_scale;
        let draw_h = sprite_h * sprite_scale;
        let modal_h = (pad_y + title_size + title_gap + draw_h + pad_y).min(max_modal_h);
        let modal_rect = Rect {
            x: (logical_width - modal_w) * 0.5,
            y: (logical_height - modal_h) * 0.5,
            w: modal_w,
            h: modal_h,
        };
        let title_y = modal_rect.y + pad_y + title_size;
        let sprite_rect = Rect {
            x: modal_rect.x + (modal_rect.w - draw_w) * 0.5,
            y: title_y + title_gap,
            w: draw_w,
            h: draw_h,
        };

        Some(EnemyInspectLayout {
            modal_rect,
            sprite_rect,
            title_size,
            title_y,
        })
    }

    fn reward_layout(&self) -> Option<RewardLayout> {
        let reward = self.reward.as_ref()?;
        if reward.options.is_empty() {
            return None;
        }

        let logical_width = self.logical_width();
        let logical_height = self.logical_height();
        let gap = HAND_MIN_GAP * 1.4;
        let (button_pad_x, button_pad_y) = boot_button_tile_padding();
        let skip_button = centered_button_rect(
            self.tr(REWARD_SKIP_LABEL, "Saltar"),
            START_BUTTON_FONT_SIZE,
            button_pad_x,
            button_pad_y,
            logical_width * 0.5,
            0.0,
        );
        let skip_button_y = (logical_height - skip_button.h - button_pad_y).max(0.0);
        let row_counts: &[usize] = if reward.options.len() == 3 && logical_width < 760.0 {
            &[2, 1]
        } else {
            &[reward.options.len()]
        };
        let max_columns = row_counts.iter().copied().max().unwrap_or(1);
        let card_w = ((logical_width - gap * (max_columns as f32 + 1.0)) / max_columns as f32)
            .clamp(158.0, 226.0);
        let title_size = fit_text_size(
            self.tr("Add a card", "Añade una carta"),
            42.0,
            (logical_width - 48.0).max(120.0),
        )
        .max(24.0);
        let subtitle_size = fit_text_size(
            reward_tier_label(reward.tier, self.language),
            18.0,
            (logical_width - 48.0).max(120.0),
        )
        .max(12.0);
        let credits_size = fit_text_size(
            &reward_credits_label(reward.tier, self.language),
            18.0,
            (logical_width - 48.0).max(120.0),
        )
        .max(12.0);
        let title_block_h = title_size + subtitle_size + 26.0;
        let credits_block_h = credits_size + 16.0;

        let mut row_heights = Vec::with_capacity(row_counts.len());
        let mut start_index = 0usize;
        for &count in row_counts {
            let end_index = (start_index + count).min(reward.options.len());
            let row_height = reward.options[start_index..end_index]
                .iter()
                .map(|card| card_content_height(self.localized_card_def(*card), card_w))
                .fold(0.0, f32::max);
            row_heights.push(row_height);
            start_index = end_index;
        }

        let total_cards_h =
            row_heights.iter().sum::<f32>() + gap * row_heights.len().saturating_sub(1) as f32;
        let total_stack_h = title_block_h + total_cards_h + gap + credits_block_h;
        let content_bottom = (skip_button_y - gap).max(gap);
        let stack_top = ((content_bottom - total_stack_h) * 0.5).max(gap);

        let mut card_rects = Vec::with_capacity(reward.options.len());
        let mut row_top = stack_top + title_block_h;
        let mut option_index = 0usize;
        for (row_index, &count) in row_counts.iter().enumerate() {
            let row_width = count as f32 * card_w + gap * count.saturating_sub(1) as f32;
            let mut x = (logical_width - row_width) * 0.5;
            for _ in 0..count {
                if option_index >= reward.options.len() {
                    break;
                }
                let card_h = card_content_height(
                    self.localized_card_def(reward.options[option_index]),
                    card_w,
                );
                card_rects.push(Rect {
                    x,
                    y: row_top + (row_heights[row_index] - card_h) * 0.5,
                    w: card_w,
                    h: card_h,
                });
                x += card_w + gap;
                option_index += 1;
            }
            row_top += row_heights[row_index] + gap;
        }

        Some(RewardLayout {
            card_rects,
            credits_y: row_top + credits_size,
            skip_button: Rect {
                x: skip_button.x,
                y: skip_button_y,
                w: skip_button.w,
                h: skip_button.h,
            },
        })
    }

    fn shop_layout(&self) -> Option<ShopLayout> {
        let shop = self.shop.as_ref()?;
        if shop.offers.is_empty() {
            return None;
        }

        let logical_width = self.logical_width();
        let logical_height = self.logical_height();
        let gap = HAND_MIN_GAP * 1.4;
        let (button_pad_x, button_pad_y) = boot_button_tile_padding();
        let leave_button = centered_button_rect(
            self.tr(SHOP_LEAVE_LABEL, "Salir"),
            START_BUTTON_FONT_SIZE,
            button_pad_x,
            button_pad_y,
            logical_width * 0.5,
            0.0,
        );
        let leave_button_y = (logical_height - leave_button.h - button_pad_y).max(0.0);
        let row_counts: &[usize] = if shop.offers.len() == 3 && logical_width < 760.0 {
            &[2, 1]
        } else {
            &[shop.offers.len()]
        };
        let max_columns = row_counts.iter().copied().max().unwrap_or(1);
        let card_w = ((logical_width - gap * (max_columns as f32 + 1.0)) / max_columns as f32)
            .clamp(158.0, 226.0);
        let title_size = fit_text_size(
            self.tr("Shop", "Tienda"),
            42.0,
            (logical_width - 48.0).max(120.0),
        )
        .max(24.0);
        let subtitle_size = fit_text_size(
            self.tr("Buy 1 card", "Compra 1 carta"),
            18.0,
            (logical_width - 48.0).max(120.0),
        )
        .max(12.0);
        let credits_size = fit_text_size(
            self.tr("You have 99 Credits", "Tienes 99 Créditos"),
            18.0,
            (logical_width - 48.0).max(120.0),
        )
        .max(12.0);
        let price_size = fit_text_size(
            self.tr("40 Credits", "40 Créditos"),
            18.0,
            (logical_width - 48.0).max(120.0),
        )
        .max(12.0);
        let price_gap = 6.0;
        let credits_gap = 28.0;
        let title_block_h = title_size + subtitle_size + 26.0;
        let credits_block_h = credits_size + 16.0;

        let mut row_heights = Vec::with_capacity(row_counts.len());
        let mut start_index = 0usize;
        for &count in row_counts {
            let end_index = (start_index + count).min(shop.offers.len());
            let row_height = shop.offers[start_index..end_index]
                .iter()
                .map(|offer| {
                    card_content_height(self.localized_card_def(offer.card), card_w)
                        + price_gap
                        + price_size
                })
                .fold(0.0, f32::max);
            row_heights.push(row_height);
            start_index = end_index;
        }

        let total_cards_h =
            row_heights.iter().sum::<f32>() + gap * row_heights.len().saturating_sub(1) as f32;
        let total_stack_h = title_block_h + total_cards_h + credits_gap + credits_block_h;
        let content_bottom = (leave_button_y - gap).max(gap);
        let stack_top = ((content_bottom - total_stack_h) * 0.5).max(gap);

        let mut card_rects = Vec::with_capacity(shop.offers.len());
        let mut price_ys = Vec::with_capacity(shop.offers.len());
        let mut row_top = stack_top + title_block_h;
        let mut option_index = 0usize;
        for (row_index, &count) in row_counts.iter().enumerate() {
            let row_width = count as f32 * card_w + gap * count.saturating_sub(1) as f32;
            let mut x = (logical_width - row_width) * 0.5;
            for _ in 0..count {
                let Some(offer) = shop.offers.get(option_index) else {
                    break;
                };
                let card_h = card_content_height(self.localized_card_def(offer.card), card_w);
                let card_y =
                    row_top + (row_heights[row_index] - price_gap - price_size - card_h) * 0.5;
                card_rects.push(Rect {
                    x,
                    y: card_y,
                    w: card_w,
                    h: card_h,
                });
                price_ys.push(card_y + card_h + price_gap + price_size);
                x += card_w + gap;
                option_index += 1;
            }
            row_top += row_heights[row_index] + gap;
        }
        let cards_bottom = row_top - gap;

        Some(ShopLayout {
            card_rects,
            price_ys,
            credits_y: cards_bottom + credits_gap + credits_size,
            leave_button: Rect {
                x: leave_button.x,
                y: leave_button_y,
                w: leave_button.w,
                h: leave_button.h,
            },
        })
    }

    fn rest_layout(&self) -> Option<RestLayout> {
        let rest = self.rest.as_ref()?;
        let page_info = self.rest_page_info(self.ui.rest_page)?;
        let logical_width = self.logical_width();
        let logical_height = self.logical_height();
        let gap = HAND_MIN_GAP * 1.3;
        let (button_pad_x, button_pad_y) = boot_button_tile_padding();
        let heal_label = if rest.heal_amount > 0 {
            match self.language {
                Language::English => format!("Recover {} HP", rest.heal_amount),
                Language::Spanish => format!("Recupera {} HP", rest.heal_amount),
            }
        } else if rest.upgrade_options.is_empty() {
            String::from(self.tr("Continue", "Continuar"))
        } else {
            String::from(self.tr("HP Full", "HP al máximo"))
        };
        let heal_font_size = fit_text_size(
            self.tr("Recover 10 HP", "Recupera 10 HP"),
            26.0,
            (logical_width - 48.0).max(120.0),
        )
        .max(18.0);
        let heal_insets = tile_insets_for_card_width(220.0);
        let heal_w = (text_width(&heal_label, heal_font_size) + button_pad_x * 2.0)
            .clamp(110.0, (logical_width - gap * 2.0).max(110.0));
        let heal_h = heal_insets.top_pad + heal_font_size + heal_insets.bottom_pad;
        let confirm_label = self.tr("Confirm", "Confirmar");
        let confirm_font_size =
            fit_text_size(confirm_label, 26.0, (logical_width - 48.0).max(120.0)).max(18.0);
        let confirm_insets = tile_insets_for_card_width(220.0);
        let confirm_w = (text_width(confirm_label, confirm_font_size) + button_pad_x * 2.0)
            .clamp(110.0, (logical_width - gap * 2.0).max(110.0));
        let confirm_h = confirm_insets.top_pad + confirm_font_size + confirm_insets.bottom_pad;
        let title_size = fit_text_size(
            self.tr("Rest Site", "Zona de Descanso"),
            40.0,
            (logical_width - 48.0).max(120.0),
        )
        .max(24.0);
        let subtitle_size = fit_text_size(
            self.tr(
                "Recover HP or upgrade one card",
                "Recupera HP o mejora una carta",
            ),
            18.0,
            (logical_width - 48.0).max(120.0),
        )
        .max(12.0);
        let title_block_h = title_size + subtitle_size + 34.0;
        let columns = page_info.columns;
        let upgrade_count = rest.upgrade_options.len();
        let card_w = if upgrade_count == 0 {
            0.0
        } else {
            ((logical_width - gap * (columns as f32 + 1.0)) / columns as f32).clamp(136.0, 210.0)
        };

        let mut row_counts = Vec::new();
        let mut remaining = page_info.visible_upgrade_indices.len();
        while remaining > 0 {
            let row_count = remaining.min(columns);
            row_counts.push(row_count);
            remaining -= row_count;
        }

        let mut row_heights = Vec::with_capacity(row_counts.len());
        let dungeon = self.dungeon.as_ref()?;
        let mut visible_index = 0usize;
        for &count in &row_counts {
            let row_height = (0..count)
                .filter_map(|offset| {
                    let option_index = *page_info
                        .visible_upgrade_indices
                        .get(visible_index + offset)?;
                    let deck_index = *rest.upgrade_options.get(option_index)?;
                    let upgraded = upgraded_card(*dungeon.deck.get(deck_index)?)?;
                    Some(card_content_height(
                        self.localized_card_def(upgraded),
                        card_w,
                    ))
                })
                .fold(0.0, f32::max);
            row_heights.push(row_height);
            visible_index += count;
        }

        let cards_h =
            row_heights.iter().sum::<f32>() + gap * row_heights.len().saturating_sub(1) as f32;
        let upgrade_block_h = if upgrade_count > 0 { cards_h } else { 28.0 };
        let confirm_rect = Rect {
            x: (logical_width - confirm_w) * 0.5,
            y: (logical_height - gap - confirm_h).max(gap),
            w: confirm_w,
            h: confirm_h,
        };

        let prev_label = "<";
        let next_label = ">";
        let page_button_font_size = 18.0;
        let page_button_pad_x = button_pad_x * 0.58;
        let page_button_pad_y = button_pad_y * 0.58;
        let (prev_w, prev_h) = button_size(
            prev_label,
            page_button_font_size,
            page_button_pad_x,
            page_button_pad_y,
        );
        let (next_w, next_h) = button_size(
            next_label,
            page_button_font_size,
            page_button_pad_x,
            page_button_pad_y,
        );
        let page_status_label = if page_info.page_count > 1 {
            Some(format!(
                "{}/{}",
                page_info.current_page + 1,
                page_info.page_count
            ))
        } else {
            None
        };
        let page_status_size = page_status_label.as_ref().map(|label| {
            let center_max_w = (logical_width - gap * 6.0 - prev_w - next_w).max(72.0);
            fit_text_size(label, 18.0, center_max_w).max(12.0)
        });
        let page_row_h = if page_status_label.is_some() {
            prev_h
                .max(next_h)
                .max(page_status_size.unwrap_or(0.0) + 6.0)
        } else {
            0.0
        };

        let content_h = title_block_h
            + heal_h
            + gap
            + upgrade_block_h
            + if page_status_label.is_some() {
                gap + page_row_h
            } else {
                0.0
            };
        let stack_top = ((confirm_rect.y - gap - content_h) * 0.5).max(gap);
        let heal_rect = Rect {
            x: (logical_width - heal_w) * 0.5,
            y: stack_top + title_block_h,
            w: heal_w,
            h: heal_h,
        };

        let mut card_rects = Vec::with_capacity(page_info.visible_upgrade_indices.len());
        let mut row_top = heal_rect.y + heal_rect.h + gap;
        let mut visible_index = 0usize;
        for (row_index, &count) in row_counts.iter().enumerate() {
            let row_width = count as f32 * card_w + gap * count.saturating_sub(1) as f32;
            let mut x = (logical_width - row_width) * 0.5;
            for _ in 0..count {
                let Some(&option_index) = page_info.visible_upgrade_indices.get(visible_index)
                else {
                    break;
                };
                let Some(&deck_index) = rest.upgrade_options.get(option_index) else {
                    break;
                };
                let Some(&card) = dungeon.deck.get(deck_index) else {
                    break;
                };
                let Some(upgraded) = upgraded_card(card) else {
                    break;
                };
                let card_h = card_content_height(self.localized_card_def(upgraded), card_w);
                card_rects.push(Rect {
                    x,
                    y: row_top + (row_heights[row_index] - card_h) * 0.5,
                    w: card_w,
                    h: card_h,
                });
                x += card_w + gap;
                visible_index += 1;
            }
            row_top += row_heights[row_index] + gap;
        }
        let cards_bottom = if card_rects.is_empty() {
            heal_rect.y + heal_rect.h + upgrade_block_h
        } else {
            row_top - gap
        };
        let (prev_button, next_button, page_status_x, page_status_y) =
            if let (Some(page_status_label), Some(page_status_size)) =
                (page_status_label.as_ref(), page_status_size)
            {
                let page_top = cards_bottom + gap;
                let page_status_w = text_width(page_status_label, page_status_size);
                let page_gap = (HAND_MIN_GAP * 1.2).clamp(10.0, 16.0);
                let group_w = prev_w + page_gap + page_status_w + page_gap + next_w;
                let group_x = (logical_width - group_w) * 0.5;
                (
                    Some(FittedPrimaryButton {
                        rect: Rect {
                            x: group_x,
                            y: page_top + (page_row_h - prev_h) * 0.5,
                            w: prev_w,
                            h: prev_h,
                        },
                        font_size: page_button_font_size,
                    }),
                    Some(FittedPrimaryButton {
                        rect: Rect {
                            x: group_x + prev_w + page_gap + page_status_w + page_gap,
                            y: page_top + (page_row_h - next_h) * 0.5,
                            w: next_w,
                            h: next_h,
                        },
                        font_size: page_button_font_size,
                    }),
                    Some(group_x + prev_w + page_gap + page_status_w * 0.5),
                    Some(page_top + page_row_h * 0.5 + page_status_size * 0.32),
                )
            } else {
                (None, None, None, None)
            };

        Some(RestLayout {
            heal_rect,
            card_rects,
            visible_upgrade_indices: page_info.visible_upgrade_indices,
            prev_button,
            next_button,
            page_status_label,
            page_status_x,
            page_status_y,
            page_status_size,
            current_page: page_info.current_page,
            page_count: page_info.page_count,
            confirm_rect,
        })
    }

    fn hit_test(&self, x: f32, y: f32) -> Option<HitTarget> {
        match self.screen {
            AppScreen::Boot => {
                let buttons = self.boot_buttons_layout(self.has_saved_run);
                if self.ui.install_help_open || self.install_help_visible() {
                    let install_layout = self.install_help_layout();
                    if install_layout.close_button.rect.contains(x, y) {
                        return Some(HitTarget::InstallHelpClose);
                    }
                    if install_layout.modal_rect.contains(x, y) {
                        return Some(HitTarget::InstallHelpModal);
                    }
                    return None;
                }
                if self.ui.settings_open || self.settings_visible() {
                    let settings_layout = self.settings_layout();
                    if settings_layout.english_button.rect.contains(x, y) {
                        return Some(HitTarget::SettingsLanguageEnglish);
                    }
                    if settings_layout.spanish_button.rect.contains(x, y) {
                        return Some(HitTarget::SettingsLanguageSpanish);
                    }
                    if settings_layout.modal_rect.contains(x, y) {
                        return Some(HitTarget::SettingsModal);
                    }
                    return None;
                }
                if self.ui.restart_confirm_open || self.restart_confirm_visible() {
                    let restart_layout = self.restart_confirm_layout()?;
                    if restart_layout.cancel_button.rect.contains(x, y) {
                        return Some(HitTarget::RestartCancel);
                    }
                    if restart_layout.restart_button.rect.contains(x, y) {
                        return Some(HitTarget::RestartConfirm);
                    }
                    if restart_layout.modal_rect.contains(x, y) {
                        return Some(HitTarget::RestartModal);
                    }
                    return None;
                }
                if buttons.start_button.contains(x, y) {
                    return Some(if self.has_saved_run {
                        HitTarget::Continue
                    } else {
                        HitTarget::Start
                    });
                }
                if self.has_saved_run && buttons.restart_button.contains(x, y) {
                    return Some(HitTarget::Restart);
                }
                if buttons.settings_button.contains(x, y) {
                    return Some(HitTarget::Settings);
                }
                if buttons
                    .install_button
                    .is_some_and(|button| button.contains(x, y))
                {
                    return Some(HitTarget::Install);
                }
                if buttons
                    .update_button
                    .is_some_and(|button| button.contains(x, y))
                {
                    return Some(HitTarget::Update);
                }
                if self.debug_mode
                    && self.has_saved_run
                    && buttons
                        .clear_save_button
                        .is_some_and(|button| button.contains(x, y))
                {
                    return Some(HitTarget::DebugClearSave);
                }
                None
            }
            AppScreen::OpeningIntro => {
                if self.opening_intro_action_button().rect.contains(x, y) {
                    return Some(HitTarget::Continue);
                }
                None
            }
            AppScreen::Map => {
                let map_layout = self.map_layout()?;
                if map_layout.info_button.contains(x, y) {
                    return Some(HitTarget::Info);
                }
                if map_layout.legend_button.contains(x, y) {
                    return Some(HitTarget::Legend);
                }
                if self.run_info_visible() || self.ui.run_info_open {
                    let run_info_layout = self.run_info_layout()?;
                    if run_info_layout.modal_rect.contains(x, y) {
                        return Some(HitTarget::RunInfoPanel);
                    }
                    return None;
                }
                if self.legend_visible() || self.ui.legend_open {
                    if map_layout.legend_modal.contains(x, y) {
                        return Some(HitTarget::LegendPanel);
                    }
                    return None;
                }
                if map_layout.menu_button.contains(x, y) {
                    return Some(HitTarget::Menu);
                }
                if let Some(rect) = map_layout.debug_level_down_button {
                    if rect.contains(x, y) {
                        return Some(HitTarget::DebugLevelDown);
                    }
                }
                if let Some(rect) = map_layout.debug_level_up_button {
                    if rect.contains(x, y) {
                        return Some(HitTarget::DebugLevelUp);
                    }
                }
                if let Some(rect) = map_layout.debug_fill_deck_button {
                    if rect.contains(x, y) {
                        return Some(HitTarget::DebugFillDeck);
                    }
                }
                for node in map_layout.nodes.iter().rev() {
                    if point_in_circle(x, y, node.center_x, node.center_y, MAP_NODE_RADIUS) {
                        return Some(HitTarget::MapNode(node.id));
                    }
                }
                None
            }
            AppScreen::ModuleSelect => {
                let layout = self.module_select_layout()?;
                for (index, rect) in layout.card_rects.iter().enumerate().rev() {
                    if rect.contains(x, y) {
                        return Some(HitTarget::ModuleSelectCard(index));
                    }
                }
                None
            }
            AppScreen::LevelIntro => {
                if self.level_intro_continue_button_rect().contains(x, y) {
                    return Some(HitTarget::Continue);
                }
                None
            }
            AppScreen::Rest => {
                let rest_layout = self.rest_layout()?;
                if rest_layout.heal_rect.contains(x, y) && self.rest_heal_actionable() {
                    return Some(HitTarget::RestHeal);
                }
                if rest_layout.confirm_rect.contains(x, y) && self.ui.rest_selection.is_some() {
                    return Some(HitTarget::RestConfirm);
                }
                if let Some(button) = rest_layout.prev_button {
                    if button.rect.contains(x, y) && rest_layout.current_page > 0 {
                        return Some(HitTarget::RestPagePrev);
                    }
                }
                if let Some(button) = rest_layout.next_button {
                    if button.rect.contains(x, y)
                        && rest_layout.current_page + 1 < rest_layout.page_count
                    {
                        return Some(HitTarget::RestPageNext);
                    }
                }
                for (slot, rect) in rest_layout.card_rects.iter().enumerate().rev() {
                    if rect.contains(x, y) {
                        if let Some(&option_index) = rest_layout.visible_upgrade_indices.get(slot) {
                            return Some(HitTarget::RestCard(option_index));
                        }
                    }
                }
                None
            }
            AppScreen::Shop => {
                let shop_layout = self.shop_layout()?;
                if shop_layout.leave_button.contains(x, y) {
                    return Some(HitTarget::ShopLeave);
                }
                let credits = self
                    .dungeon
                    .as_ref()
                    .map(|dungeon| dungeon.credits)
                    .unwrap_or(0);
                for (index, rect) in shop_layout.card_rects.iter().enumerate().rev() {
                    let Some(offer) = self.shop.as_ref().and_then(|shop| shop.offers.get(index))
                    else {
                        continue;
                    };
                    if credits >= offer.price && rect.contains(x, y) {
                        return Some(HitTarget::ShopCard(index));
                    }
                }
                None
            }
            AppScreen::Event => {
                let event_layout = self.event_layout()?;
                for (index, rect) in event_layout.choice_rects.iter().enumerate().rev() {
                    if rect.contains(x, y) {
                        return Some(HitTarget::EventChoice(index));
                    }
                }
                None
            }
            AppScreen::Reward => {
                let reward_layout = self.reward_layout()?;
                if reward_layout.skip_button.contains(x, y) {
                    return Some(HitTarget::RewardSkip);
                }
                for (index, rect) in reward_layout.card_rects.iter().enumerate().rev() {
                    if rect.contains(x, y) {
                        return Some(HitTarget::RewardCard(index));
                    }
                }
                None
            }
            AppScreen::Combat => {
                if self.combat_input_locked() {
                    return None;
                }
                let layout = self.layout();
                if self.ui.run_info_open {
                    let run_info_layout = self.run_info_layout()?;
                    if run_info_layout.modal_rect.contains(x, y) {
                        return Some(HitTarget::RunInfoPanel);
                    }
                    for (&enemy_index, rect) in layout
                        .enemy_indices
                        .iter()
                        .zip(layout.enemy_rects.iter())
                        .rev()
                    {
                        if rect.contains(x, y) {
                            return Some(HitTarget::Enemy(enemy_index));
                        }
                    }
                    if layout.player_rect.contains(x, y) {
                        return Some(HitTarget::Player);
                    }
                    return None;
                }
                if self.ui.enemy_inspect_open {
                    let enemy_inspect_layout = self.enemy_inspect_layout()?;
                    if enemy_inspect_layout.modal_rect.contains(x, y) {
                        return Some(HitTarget::EnemyInspectPanel);
                    }
                    for (&enemy_index, rect) in layout
                        .enemy_indices
                        .iter()
                        .zip(layout.enemy_rects.iter())
                        .rev()
                    {
                        if rect.contains(x, y) {
                            return Some(HitTarget::Enemy(enemy_index));
                        }
                    }
                    if layout.player_rect.contains(x, y) {
                        return Some(HitTarget::Player);
                    }
                    return None;
                }
                if self.run_info_visible() {
                    let run_info_layout = self.run_info_layout()?;
                    if run_info_layout.modal_rect.contains(x, y) {
                        return Some(HitTarget::RunInfoPanel);
                    }
                    for (&enemy_index, rect) in layout
                        .enemy_indices
                        .iter()
                        .zip(layout.enemy_rects.iter())
                        .rev()
                    {
                        if rect.contains(x, y) {
                            return Some(HitTarget::Enemy(enemy_index));
                        }
                    }
                    if layout.player_rect.contains(x, y) {
                        return Some(HitTarget::Player);
                    }
                    return None;
                }
                if self.enemy_inspect_visible() {
                    let enemy_inspect_layout = self.enemy_inspect_layout()?;
                    if enemy_inspect_layout.modal_rect.contains(x, y) {
                        return Some(HitTarget::EnemyInspectPanel);
                    }
                    for (&enemy_index, rect) in layout
                        .enemy_indices
                        .iter()
                        .zip(layout.enemy_rects.iter())
                        .rev()
                    {
                        if rect.contains(x, y) {
                            return Some(HitTarget::Enemy(enemy_index));
                        }
                    }
                    if layout.player_rect.contains(x, y) {
                        return Some(HitTarget::Player);
                    }
                    return None;
                }
                for (index, rect) in layout.hand_rects.iter().enumerate().rev() {
                    if rect.contains(x, y) {
                        return Some(HitTarget::Card(index));
                    }
                }
                if layout.menu_button.contains(x, y) {
                    return Some(HitTarget::Menu);
                }
                if layout.end_turn_button.contains(x, y) {
                    return Some(HitTarget::EndTurn);
                }
                if let Some(rect) = layout.end_battle_button {
                    if rect.contains(x, y) {
                        return Some(HitTarget::EndBattle);
                    }
                }
                for (&enemy_index, rect) in layout
                    .enemy_indices
                    .iter()
                    .zip(layout.enemy_rects.iter())
                    .rev()
                {
                    if rect.contains(x, y) {
                        return Some(HitTarget::Enemy(enemy_index));
                    }
                }
                if layout.player_rect.contains(x, y) {
                    return Some(HitTarget::Player);
                }
                None
            }
            AppScreen::Result(_) => {
                let buttons = result_button_layout(
                    self.logical_width(),
                    self.logical_height(),
                    self.final_victory_summary().is_some(),
                    self.language,
                );
                if let Some(rect) = buttons.share_button {
                    if rect.contains(x, y) {
                        return Some(HitTarget::Share);
                    }
                }
                if buttons.menu_button.contains(x, y) {
                    return Some(HitTarget::Restart);
                }
                None
            }
        }
    }

    fn to_logical(&self, x: f32, y: f32) -> Option<(f32, f32)> {
        if x < 0.0 || y < 0.0 || x > self.logical_width() || y > self.logical_height() {
            return None;
        }

        Some((x, y))
    }

    fn rebuild_frame(&mut self) {
        let mut scene = SceneBuilder::new();
        self.render_background(&mut scene);
        if let Some(transition) = &self.screen_transition {
            let progress = 1.0 - (transition.ttl_ms / transition.total_ms).clamp(0.0, 1.0);
            let eased = ease_out_cubic(progress);
            let from_alpha = 1.0 - eased;
            let to_alpha = eased;
            let (from_offset_y, from_scale, to_offset_y, to_scale) = match transition.style {
                ScreenTransitionStyle::Motion => {
                    let travel = self.logical_height().min(72.0);
                    (
                        -travel * 0.18 * eased,
                        1.0 - eased * 0.035,
                        travel * 0.18 * (1.0 - eased),
                        0.965 + eased * 0.035,
                    )
                }
                ScreenTransitionStyle::Fade => (0.0, 1.0, 0.0, 1.0),
            };

            self.render_screen_layer(
                &mut scene,
                transition.from_screen,
                if matches!(transition.from_screen, AppScreen::Boot) {
                    Some(transition.from_boot_has_saved_run)
                } else {
                    None
                },
                from_alpha,
                from_offset_y,
                from_scale,
            );
            self.render_screen_layer(
                &mut scene,
                transition.to_screen,
                if matches!(transition.to_screen, AppScreen::Boot) {
                    Some(transition.to_boot_has_saved_run)
                } else {
                    None
                },
                to_alpha,
                to_offset_y,
                to_scale,
            );
        } else {
            self.render_screen(&mut scene, self.screen, None);
        }
        self.frame = scene.finish().into_bytes();
        self.dirty = false;
    }

    fn render_background(&self, scene: &mut SceneBuilder) {
        scene.clear("#000000");
        scene.rect(
            Rect {
                x: 0.0,
                y: 0.0,
                w: self.logical_width(),
                h: self.logical_height(),
            },
            0.0,
            COLOR_TILE_FILL,
            "transparent",
            0.0,
        );
    }

    fn render_screen_layer(
        &self,
        scene: &mut SceneBuilder,
        screen: AppScreen,
        boot_has_saved_run: Option<bool>,
        alpha: f32,
        offset_y: f32,
        scale: f32,
    ) {
        if alpha <= 0.0 {
            return;
        }

        scene.push_layer(alpha, 0.0, offset_y, scale);
        self.render_screen(scene, screen, boot_has_saved_run);
        scene.pop_layer();
    }

    fn render_screen(
        &self,
        scene: &mut SceneBuilder,
        screen: AppScreen,
        boot_has_saved_run: Option<bool>,
    ) {
        match screen {
            AppScreen::Boot => {
                self.render_boot(scene, boot_has_saved_run.unwrap_or(self.has_saved_run))
            }
            AppScreen::OpeningIntro => self.render_opening_intro(scene),
            AppScreen::Map => self.render_map(scene),
            AppScreen::ModuleSelect => self.render_module_select(scene),
            AppScreen::LevelIntro => self.render_level_intro(scene),
            AppScreen::Rest => self.render_rest(scene),
            AppScreen::Shop => self.render_shop(scene),
            AppScreen::Event => self.render_event(scene),
            AppScreen::Reward => self.render_reward(scene),
            AppScreen::Combat => {
                self.render_combat(scene);
                self.render_pixel_shards(scene);
                self.render_floaters(scene);
            }
            AppScreen::Result(outcome) => {
                self.render_result_overlay(scene, outcome);
                self.render_pixel_shards(scene);
            }
        }

        if matches!(screen, AppScreen::Map | AppScreen::Combat) {
            if matches!(screen, AppScreen::Combat) {
                if self.ui.run_info_open {
                    self.render_enemy_inspect(scene);
                    self.render_run_info(scene);
                } else {
                    self.render_run_info(scene);
                    self.render_enemy_inspect(scene);
                }
            } else {
                self.render_run_info(scene);
            }
        }
    }

    fn render_boot(&self, scene: &mut SceneBuilder, has_saved_run: bool) {
        let buttons = self.boot_buttons_layout(has_saved_run);
        let hero = boot_hero_layout(self.logical_width(), self.logical_height());
        let version_line = visible_game_version_label();
        scene.image(hero.logo_rect, LOGO_ASSET_PATH, 0.96);
        scene.text(
            self.logical_center_x(),
            hero.title_baseline_y,
            hero.title_size,
            "center",
            "#33ff66",
            "display",
            "Mazocarta",
        );

        let primary_label = if has_saved_run {
            self.tr(BOOT_CONTINUE_LABEL, "Continuar")
        } else {
            self.tr("Start", "Empezar")
        };
        render_primary_button(
            scene,
            buttons.start_button,
            matches!(self.ui.hover, Some(HitTarget::Start | HitTarget::Continue)),
            primary_label,
            self.boot_time_ms,
        );

        if has_saved_run {
            let restart_hovered = self.ui.hover == Some(HitTarget::Restart);
            render_primary_button(
                scene,
                buttons.restart_button,
                restart_hovered,
                self.tr(BOOT_RESTART_LABEL, "Reiniciar"),
                self.boot_time_ms,
            );
        }

        render_primary_button(
            scene,
            buttons.settings_button,
            self.ui.hover == Some(HitTarget::Settings),
            self.tr(BOOT_SETTINGS_LABEL, "Ajustes"),
            self.boot_time_ms,
        );

        if let Some(install_button) = buttons.install_button {
            render_primary_button(
                scene,
                install_button,
                self.ui.hover == Some(HitTarget::Install),
                self.tr(BOOT_INSTALL_LABEL, "Instalar"),
                self.boot_time_ms,
            );
        }

        if let Some(update_button) = buttons.update_button {
            render_primary_button(
                scene,
                update_button,
                self.ui.hover == Some(HitTarget::Update),
                self.tr(BOOT_UPDATE_LABEL, "Actualizar"),
                self.boot_time_ms,
            );
        }

        if let Some(clear_save_button) = buttons.clear_save_button {
            render_primary_button(
                scene,
                clear_save_button,
                self.ui.hover == Some(HitTarget::DebugClearSave),
                self.tr(BOOT_DEBUG_CLEAR_LABEL, "Reset"),
                self.boot_time_ms,
            );
        }

        scene.text(
            self.logical_center_x(),
            boot_version_baseline_y(self.logical_height()),
            boot_version_font_size(self.logical_width()),
            "center",
            TERM_GREEN_DIM,
            "body",
            &version_line,
        );

        self.render_settings_modal(scene);
        self.render_install_help_modal(scene);
        self.render_restart_confirm(scene);
    }

    fn render_restart_confirm(&self, scene: &mut SceneBuilder) {
        let progress = self.restart_confirm_eased_progress();
        if progress <= 0.0 {
            return;
        }

        let Some(layout) = self.restart_confirm_layout() else {
            return;
        };
        let backdrop_rect = Rect {
            x: 0.0,
            y: 0.0,
            w: self.logical_width(),
            h: self.logical_height(),
        };
        scene.blur_rect(backdrop_rect, 0.0, BOOT_RESTART_MODAL_BLUR * progress);
        scene.rect(
            backdrop_rect,
            0.0,
            &rgba((0, 0, 0), 0.22 * progress),
            "transparent",
            0.0,
        );
        scene.push_layer(
            progress,
            0.0,
            (1.0 - progress) * 12.0,
            0.968 + progress * 0.032,
        );
        render_ui_tile(
            scene,
            layout.modal_rect,
            BUTTON_RADIUS,
            COLOR_GREEN_STROKE_PANEL,
        );
        let title_gap = 8.0;
        let title_start_y = layout.modal_rect.y + 24.0 + layout.title_size;
        for (index, line) in layout.title_lines.iter().enumerate() {
            scene.text(
                layout.modal_rect.x + layout.modal_rect.w * 0.5,
                title_start_y + index as f32 * (layout.title_size + title_gap),
                layout.title_size,
                "center",
                TERM_GREEN_SOFT,
                "label",
                line,
            );
        }

        let cancel_hovered = self.ui.hover == Some(HitTarget::RestartCancel);
        render_primary_button_sized(
            scene,
            layout.cancel_button.rect,
            layout.cancel_button.font_size,
            cancel_hovered,
            self.tr(BOOT_RESTART_CONFIRM_CANCEL_LABEL, "Cancelar"),
            self.boot_time_ms,
        );

        let restart_hovered = self.ui.hover == Some(HitTarget::RestartConfirm);
        render_primary_button_sized(
            scene,
            layout.restart_button.rect,
            layout.restart_button.font_size,
            restart_hovered,
            self.tr(BOOT_RESTART_LABEL, "Reiniciar"),
            self.boot_time_ms,
        );
        scene.pop_layer();
    }

    fn render_settings_modal(&self, scene: &mut SceneBuilder) {
        let progress = self.settings_eased_progress();
        if progress <= 0.0 {
            return;
        }

        let layout = self.settings_layout();
        let backdrop_rect = Rect {
            x: 0.0,
            y: 0.0,
            w: self.logical_width(),
            h: self.logical_height(),
        };
        scene.blur_rect(backdrop_rect, 0.0, BOOT_RESTART_MODAL_BLUR * progress);
        scene.rect(
            backdrop_rect,
            0.0,
            &rgba((0, 0, 0), 0.22 * progress),
            "transparent",
            0.0,
        );
        scene.push_layer(
            progress,
            0.0,
            (1.0 - progress) * 12.0,
            0.968 + progress * 0.032,
        );
        render_ui_tile(
            scene,
            layout.modal_rect,
            BUTTON_RADIUS,
            COLOR_GREEN_STROKE_PANEL,
        );
        let title_gap = 8.0;
        let subtitle_gap = 6.0;
        let mut baseline_y = layout.modal_rect.y + 24.0 + layout.title_size;
        for line in &layout.title_lines {
            scene.text(
                layout.modal_rect.x + layout.modal_rect.w * 0.5,
                baseline_y,
                layout.title_size,
                "center",
                TERM_GREEN_SOFT,
                "label",
                line,
            );
            baseline_y += layout.title_size + title_gap;
        }
        baseline_y += 6.0;
        for line in &layout.subtitle_lines {
            scene.text(
                layout.modal_rect.x + layout.modal_rect.w * 0.5,
                baseline_y,
                layout.subtitle_size,
                "center",
                TERM_GREEN_TEXT,
                "body",
                line,
            );
            baseline_y += layout.subtitle_size + subtitle_gap;
        }

        let english_selected = self.language == Language::English;
        let spanish_selected = self.language == Language::Spanish;
        let english_hovered = self.ui.hover == Some(HitTarget::SettingsLanguageEnglish);
        let spanish_hovered = self.ui.hover == Some(HitTarget::SettingsLanguageSpanish);
        render_primary_button_sized(
            scene,
            layout.english_button.rect,
            layout.english_button.font_size,
            english_hovered || english_selected,
            "English",
            self.boot_time_ms,
        );
        render_primary_button_sized(
            scene,
            layout.spanish_button.rect,
            layout.spanish_button.font_size,
            spanish_hovered || spanish_selected,
            "Español",
            self.boot_time_ms,
        );
        scene.pop_layer();
    }

    fn render_install_help_modal(&self, scene: &mut SceneBuilder) {
        let progress = self.install_help_eased_progress();
        if progress <= 0.0 {
            return;
        }

        let layout = self.install_help_layout();
        let backdrop_rect = Rect {
            x: 0.0,
            y: 0.0,
            w: self.logical_width(),
            h: self.logical_height(),
        };
        scene.blur_rect(backdrop_rect, 0.0, BOOT_RESTART_MODAL_BLUR * progress);
        scene.rect(
            backdrop_rect,
            0.0,
            &rgba((0, 0, 0), 0.22 * progress),
            "transparent",
            0.0,
        );
        scene.push_layer(
            progress,
            0.0,
            (1.0 - progress) * 12.0,
            0.968 + progress * 0.032,
        );
        render_ui_tile(
            scene,
            layout.modal_rect,
            BUTTON_RADIUS,
            COLOR_GREEN_STROKE_PANEL,
        );
        let title_gap = 8.0;
        let body_gap = 6.0;
        let mut baseline_y = layout.modal_rect.y + 24.0 + layout.title_size;
        for line in &layout.title_lines {
            scene.text(
                layout.modal_rect.x + layout.modal_rect.w * 0.5,
                baseline_y,
                layout.title_size,
                "center",
                TERM_GREEN_SOFT,
                "label",
                line,
            );
            baseline_y += layout.title_size + title_gap;
        }
        baseline_y += 8.0;
        for line in &layout.body_lines {
            scene.text(
                layout.modal_rect.x + layout.modal_rect.w * 0.5,
                baseline_y,
                layout.body_size,
                "center",
                TERM_GREEN_TEXT,
                "body",
                line,
            );
            baseline_y += layout.body_size + body_gap;
        }

        render_primary_button_sized(
            scene,
            layout.close_button.rect,
            layout.close_button.font_size,
            self.ui.hover == Some(HitTarget::InstallHelpClose),
            self.tr("Close", "Cerrar"),
            self.boot_time_ms,
        );
        scene.pop_layer();
    }

    fn render_opening_intro(&self, scene: &mut SceneBuilder) {
        let progress = self.opening_intro_progress();
        let lines = self.opening_intro_lines();
        let center_x = self.logical_center_x();
        let body_max_width = (self.logical_width() - 120.0).max(220.0);
        let widest_line = lines
            .iter()
            .max_by_key(|line| line.chars().count())
            .copied()
            .unwrap_or(lines[0]);
        let body_size = fit_text_size(widest_line, 24.0, body_max_width).max(15.0);
        let body_chars = ((body_max_width / (body_size * 0.62)).floor() as usize).max(20);
        let body_line_gap = (body_size * 0.26).max(4.0);
        let section_gap = (body_size * 0.8).clamp(12.0, 20.0);
        let mut baseline_y = (self.logical_height() * 0.32).max(body_size + 32.0);

        for (index, line) in lines.iter().enumerate() {
            let alpha = progress.line_alphas.get(index).copied().unwrap_or(0.0);
            if alpha <= 0.001 {
                break;
            }
            let wrapped_lines: Vec<_> = wrap_text(line, body_chars)
                .into_iter()
                .filter(|wrapped| !wrapped.is_empty())
                .collect();
            if wrapped_lines.is_empty() {
                continue;
            }
            for wrapped in wrapped_lines {
                scene.text(
                    center_x,
                    baseline_y,
                    body_size,
                    "center",
                    &rgba((201, 255, 215), alpha),
                    "body",
                    &wrapped,
                );
                baseline_y += body_size + body_line_gap;
            }
            if index + 1 < lines.len() {
                baseline_y += section_gap;
            }
        }

        let action_button = self.opening_intro_action_button();
        let hovered = self.ui.hover == Some(HitTarget::Continue);
        let label_alpha = if hovered {
            1.0
        } else {
            primary_button_pulse(self.boot_time_ms)
        };
        let button_transition = self.opening_intro_button_transition_progress();
        render_ui_tile(
            scene,
            action_button.rect,
            BUTTON_RADIUS,
            COLOR_GREEN_STROKE_START,
        );
        let text_y = button_text_baseline(action_button.rect, action_button.font_size);
        scene.text(
            action_button.rect.x + action_button.rect.w * 0.5,
            text_y,
            action_button.font_size,
            "center",
            &rgba((51, 255, 102), label_alpha * (1.0 - button_transition)),
            "label",
            self.tr("Skip Intro", "Saltar intro"),
        );
        scene.text(
            action_button.rect.x + action_button.rect.w * 0.5,
            text_y,
            action_button.font_size,
            "center",
            &rgba((51, 255, 102), label_alpha * button_transition),
            "label",
            self.tr("Continue", "Continuar"),
        );
    }

    fn render_level_intro(&self, scene: &mut SceneBuilder) {
        let Some(level_intro) = self.level_intro.as_ref() else {
            return;
        };
        let center_x = self.logical_center_x();
        let title = match self.language {
            Language::English => format!("Level {}", level_intro.level),
            Language::Spanish => format!("Nivel {}", level_intro.level),
        };
        let title_size =
            fit_text_size(&title, 24.0, (self.logical_width() - 48.0).max(120.0)).max(16.0);
        let codename_size = fit_text_size(
            level_intro.codename,
            56.0,
            (self.logical_width() - 48.0).max(120.0),
        )
        .max(30.0);
        let summary_size = fit_text_size(
            level_intro.summary,
            18.0,
            (self.logical_width() - 96.0).max(180.0),
        )
        .max(12.0);
        let summary_chars =
            ((self.logical_width() - 96.0).max(180.0) / (summary_size * 0.62)).floor() as usize;
        let summary_lines = wrap_text(level_intro.summary, summary_chars.max(20));

        scene.text(
            center_x,
            self.logical_height() * (272.0 / LOGICAL_HEIGHT),
            title_size,
            "center",
            TERM_CYAN,
            "label",
            &title,
        );
        scene.text(
            center_x,
            self.logical_height() * (338.0 / LOGICAL_HEIGHT),
            codename_size,
            "center",
            TERM_GREEN_SOFT,
            "display",
            level_intro.codename,
        );
        let mut summary_y = self.logical_height() * (386.0 / LOGICAL_HEIGHT);
        for line in summary_lines {
            scene.text(
                center_x,
                summary_y,
                summary_size,
                "center",
                TERM_GREEN_TEXT,
                "body",
                &line,
            );
            summary_y += summary_size + 6.0;
        }
        render_primary_button(
            scene,
            self.level_intro_continue_button_rect(),
            self.ui.hover == Some(HitTarget::Continue),
            self.tr("Continue", "Continuar"),
            self.boot_time_ms,
        );
    }

    fn render_module_select(&self, scene: &mut SceneBuilder) {
        let Some(module_select) = self.module_select.as_ref() else {
            return;
        };
        let Some(layout) = self.module_select_layout() else {
            return;
        };

        let logical_width = self.logical_width();
        let title = self.module_select_title();
        let title_size = fit_text_size(title, 40.0, (logical_width - 48.0).max(120.0)).max(24.0);

        scene.text(
            logical_width * 0.5,
            layout.title_y,
            title_size,
            "center",
            TERM_GREEN_SOFT,
            "display",
            title,
        );

        for (index, rect) in layout.card_rects.iter().enumerate() {
            let Some(module) = module_select.options.get(index).copied() else {
                continue;
            };
            let hovered = self.ui.hover == Some(HitTarget::ModuleSelectCard(index));
            let stroke = if hovered {
                String::from(COLOR_GREEN_STROKE_STRONG)
            } else {
                module_stroke(module)
            };
            self.render_module_tile(scene, *rect, module, &stroke);
        }
    }

    fn render_event(&self, scene: &mut SceneBuilder) {
        let Some(event) = self.event.as_ref() else {
            return;
        };
        let Some(layout) = self.event_layout() else {
            return;
        };

        let def = localized_event_def(event.event, self.language);
        let logical_width = self.logical_width();
        let title_size =
            fit_text_size(def.title, 40.0, (logical_width - 48.0).max(120.0)).max(24.0);
        let flavor_size =
            fit_text_size(def.flavor, 18.0, (logical_width - 48.0).max(120.0)).max(12.0);
        let flavor_max_w = (logical_width - 64.0).clamp(220.0, 760.0);
        let flavor_chars = ((flavor_max_w / (flavor_size * 0.58)).floor() as usize).max(18);
        let flavor_lines = wrap_text(def.flavor, flavor_chars);
        let flavor_gap = 12.0;
        let flavor_line_gap = 6.0;

        scene.text(
            logical_width * 0.5,
            layout.title_y,
            title_size,
            "center",
            TERM_BLUE_SOFT,
            "display",
            def.title,
        );
        let mut flavor_y = layout.title_y + flavor_gap + flavor_size;
        for line in flavor_lines {
            scene.text(
                logical_width * 0.5,
                flavor_y,
                flavor_size,
                "center",
                TERM_GREEN_TEXT,
                "label",
                &line,
            );
            flavor_y += flavor_size + flavor_line_gap;
        }

        for (index, rect) in layout.choice_rects.iter().enumerate() {
            let Some(title) = localized_event_choice_title(event.event, index, self.language)
            else {
                continue;
            };
            let Some(body) = self.dungeon.as_ref().and_then(|dungeon| {
                localized_event_choice_body(
                    event.event,
                    index,
                    dungeon.current_level(),
                    self.language,
                )
            }) else {
                continue;
            };
            let hovered = self.ui.hover == Some(HitTarget::EventChoice(index));
            self.render_event_choice_tile(
                scene,
                *rect,
                title,
                &body,
                EventChoiceTileStyle {
                    stroke: if hovered {
                        COLOR_BLUE_STROKE_STRONG
                    } else {
                        COLOR_BLUE_STROKE_IDLE
                    },
                    title_color: TERM_BLUE_SOFT,
                    body_color: TERM_GREEN_TEXT,
                },
            );
        }
    }

    fn render_run_info(&self, scene: &mut SceneBuilder) {
        let progress = self.run_info_eased_progress();
        if progress <= 0.0 {
            return;
        }

        let Some(dungeon) = self.dungeon.as_ref() else {
            return;
        };
        let Some(layout) = self.run_info_layout() else {
            return;
        };

        let backdrop_rect = Rect {
            x: 0.0,
            y: 0.0,
            w: self.logical_width(),
            h: self.logical_height(),
        };
        let rect = layout.modal_rect;
        let (_, pad_y) = standard_overlay_padding();
        let module_wrap_side_pad = 14.0;
        let title_size = 24.0;
        let row_size = 18.0_f32;
        let module_name_size = 18.0;
        let module_body_size = 16.0;
        let module_title_top_gap = 10.0;
        let module_gap = 10.0;
        let line_gap = 8.0_f32;
        let title_gap = 34.0_f32;
        let modules_gap = (title_gap - (row_size + line_gap)).max(0.0_f32);
        let inner_w = (rect.w - module_wrap_side_pad * 2.0).max(136.0);

        scene.blur_rect(backdrop_rect, 0.0, LEGEND_BACKDROP_BLUR * progress);
        scene.rect(
            backdrop_rect,
            0.0,
            &rgba((0, 0, 0), 0.18 * progress),
            "transparent",
            0.0,
        );
        scene.push_layer(
            progress,
            0.0,
            (1.0 - progress) * 12.0,
            0.965 + progress * 0.035,
        );
        render_ui_tile(scene, rect, BUTTON_RADIUS, COLOR_GREEN_STROKE_PANEL);
        scene.text(
            rect.x + rect.w * 0.5,
            rect.y + pad_y + title_size,
            title_size,
            "center",
            TERM_CYAN_SOFT,
            "label",
            self.tr("Run Info", "Info de la Run"),
        );

        let mut y = rect.y + pad_y + title_size + title_gap;
        let summary_lines = [
            match self.language {
                Language::English => format!("Level {}", dungeon.current_level()),
                Language::Spanish => format!("Nivel {}", dungeon.current_level()),
            },
            format!("HP {}/{}", dungeon.player_hp, dungeon.player_max_hp),
            credits_label(dungeon.credits, self.language),
            card_deck_label(dungeon.deck.len(), self.language),
        ];
        for (line_index, line) in summary_lines.iter().enumerate() {
            scene.text(
                rect.x + rect.w * 0.5,
                y,
                row_size,
                "center",
                TERM_GREEN_TEXT,
                "body",
                line,
            );
            y += row_size;
            if line_index + 1 < summary_lines.len() {
                y += line_gap;
            }
        }
        y += modules_gap;

        if dungeon.modules.is_empty() {
            scene.text(
                rect.x + rect.w * 0.5,
                y,
                module_body_size,
                "center",
                TERM_GREEN_DIM,
                "body",
                self.tr("No modules online.", "No hay modulos activos."),
            );
        } else {
            let module_chars = ((inner_w / (module_body_size * 0.62)).floor() as usize).max(14);
            let mut modules = dungeon.modules.clone();
            modules.sort_by_key(|module| module_sort_order(*module));
            for (module_index, module) in modules.iter().enumerate() {
                let def = self.localized_module_def(*module);
                y += module_title_top_gap;
                scene.text(
                    rect.x + rect.w * 0.5,
                    y,
                    module_name_size,
                    "center",
                    module_accent_color(*module),
                    "label",
                    def.name,
                );
                y += module_name_size + 6.0;
                let body_lines = wrap_text(def.description, module_chars);
                for (line_index, line) in body_lines.iter().enumerate() {
                    scene.text(
                        rect.x + rect.w * 0.5,
                        y,
                        module_body_size,
                        "center",
                        TERM_GREEN_TEXT,
                        "body",
                        line,
                    );
                    y += module_body_size;
                    if line_index + 1 < body_lines.len() {
                        y += 5.0;
                    }
                }
                if module_index + 1 < modules.len() {
                    y += module_gap;
                }
            }
        }
        scene.pop_layer();
    }

    fn render_enemy_inspect(&self, scene: &mut SceneBuilder) {
        let progress = self.enemy_inspect_eased_progress();
        if progress <= 0.0 {
            return;
        }

        let Some(enemy_index) = self.ui.enemy_inspect_enemy else {
            return;
        };
        let Some(enemy) = self.combat.enemy(enemy_index) else {
            return;
        };
        let Some(layout) = self.enemy_inspect_layout() else {
            return;
        };

        let backdrop_rect = Rect {
            x: 0.0,
            y: 0.0,
            w: self.logical_width(),
            h: self.logical_height(),
        };
        let rect = layout.modal_rect;
        let sprite = enemy_sprite_def(enemy.profile);
        let is_alive = enemy.fighter.hp > 0;

        scene.blur_rect(backdrop_rect, 0.0, LEGEND_BACKDROP_BLUR * progress);
        scene.rect(
            backdrop_rect,
            0.0,
            &rgba((0, 0, 0), 0.18 * progress),
            "transparent",
            0.0,
        );
        scene.push_layer(
            progress,
            0.0,
            (1.0 - progress) * 12.0,
            0.965 + progress * 0.035,
        );
        render_ui_tile(scene, rect, BUTTON_RADIUS, COLOR_GREEN_STROKE_PANEL);
        scene.text(
            rect.x + rect.w * 0.5,
            layout.title_y,
            layout.title_size,
            "center",
            TERM_CYAN_SOFT,
            "label",
            localized_enemy_name(enemy.profile, self.language),
        );
        for layer in sprite.layers {
            scene.sprite(
                layout.sprite_rect,
                layer.code,
                enemy_sprite_layer_color(enemy.profile, layer.tone, is_alive),
                enemy_panel_icon_alpha(enemy.profile, is_alive),
            );
        }
        scene.pop_layer();
    }

    fn render_rest(&self, scene: &mut SceneBuilder) {
        let Some(rest) = self.rest.as_ref() else {
            return;
        };
        let Some(layout) = self.rest_layout() else {
            return;
        };

        let logical_width = self.logical_width();
        let title = self.tr("Rest Site", "Zona de Descanso");
        let subtitle = self.tr(
            "Recover HP or upgrade one card",
            "Recupera HP o mejora una carta",
        );
        let title_size = fit_text_size(title, 40.0, (logical_width - 48.0).max(120.0)).max(24.0);
        let subtitle_size =
            fit_text_size(subtitle, 18.0, (logical_width - 48.0).max(120.0)).max(12.0);
        let title_y = (layout.heal_rect.y - 58.0).max(74.0);
        let heal_actionable = self.rest_heal_actionable();
        let heal_selected = self.ui.rest_selection == Some(RestSelection::Heal);
        let heal_hovered = heal_actionable && self.ui.hover == Some(HitTarget::RestHeal);
        let heal_label = if rest.heal_amount > 0 {
            match self.language {
                Language::English => format!("Recover {} HP", rest.heal_amount),
                Language::Spanish => format!("Recupera {} HP", rest.heal_amount),
            }
        } else if rest.upgrade_options.is_empty() {
            String::from(self.tr("Continue", "Continuar"))
        } else {
            String::from(self.tr("HP Full", "HP al máximo"))
        };
        let heal_label_size =
            fit_text_size(&heal_label, 26.0, (layout.heal_rect.w - 24.0).max(100.0)).max(18.0);

        scene.text(
            logical_width * 0.5,
            title_y,
            title_size,
            "center",
            TERM_CYAN_SOFT,
            "display",
            title,
        );
        scene.text(
            logical_width * 0.5,
            title_y + 28.0,
            subtitle_size,
            "center",
            TERM_GREEN_TEXT,
            "label",
            subtitle,
        );

        render_ui_tile(
            scene,
            layout.heal_rect,
            BUTTON_RADIUS,
            if heal_actionable {
                if heal_hovered {
                    COLOR_CYAN_STROKE_STRONG
                } else if heal_selected {
                    COLOR_CYAN_STROKE_TARGET
                } else {
                    COLOR_CYAN_STROKE_IDLE
                }
            } else {
                COLOR_GRAY_STROKE_DISABLED
            },
        );
        scene.text(
            layout.heal_rect.x + layout.heal_rect.w * 0.5,
            button_text_baseline(layout.heal_rect, heal_label_size),
            heal_label_size,
            "center",
            if heal_actionable {
                TERM_CYAN_SOFT
            } else {
                "#b0b0b0"
            },
            "label",
            &heal_label,
        );

        if layout.card_rects.is_empty() {
            scene.text(
                logical_width * 0.5,
                layout.heal_rect.y + layout.heal_rect.h + 54.0,
                18.0,
                "center",
                TERM_GREEN_DIM,
                "body",
                self.tr(
                    "All cards are already upgraded.",
                    "Todas las cartas ya estan mejoradas.",
                ),
            );
        } else {
            let Some(dungeon) = self.dungeon.as_ref() else {
                return;
            };
            for (slot, rect) in layout.card_rects.iter().enumerate() {
                let Some(&option_index) = layout.visible_upgrade_indices.get(slot) else {
                    continue;
                };
                let Some(&deck_index) = rest.upgrade_options.get(option_index) else {
                    continue;
                };
                let Some(&card) = dungeon.deck.get(deck_index) else {
                    continue;
                };
                let Some(upgraded) = upgraded_card(card) else {
                    continue;
                };
                let selected = self.ui.rest_selection == Some(RestSelection::Upgrade(option_index));
                let hovered = self.ui.hover == Some(HitTarget::RestCard(option_index));
                let stroke = if selected {
                    COLOR_LIME_STROKE_TARGET
                } else if hovered {
                    COLOR_GREEN_STROKE_STRONG
                } else {
                    COLOR_GREEN_STROKE_IDLE
                };
                self.render_selection_card(scene, *rect, upgraded, stroke);
            }
        }

        if let (
            Some(prev_button),
            Some(next_button),
            Some(page_status_label),
            Some(page_status_x),
            Some(page_status_y),
            Some(page_status_size),
        ) = (
            layout.prev_button,
            layout.next_button,
            layout.page_status_label.as_deref(),
            layout.page_status_x,
            layout.page_status_y,
            layout.page_status_size,
        ) {
            let prev_enabled = layout.current_page > 0;
            let prev_hovered = prev_enabled && self.ui.hover == Some(HitTarget::RestPagePrev);
            let next_enabled = layout.current_page + 1 < layout.page_count;
            let next_hovered = next_enabled && self.ui.hover == Some(HitTarget::RestPageNext);
            let prev_label = "<";
            let next_label = ">";

            render_ui_tile(
                scene,
                prev_button.rect,
                BUTTON_RADIUS,
                if prev_enabled {
                    if prev_hovered {
                        COLOR_GREEN_STROKE_STRONG
                    } else {
                        COLOR_GREEN_STROKE_IDLE
                    }
                } else {
                    COLOR_GRAY_STROKE_DISABLED
                },
            );
            scene.text(
                prev_button.rect.x + prev_button.rect.w * 0.5,
                button_text_baseline(prev_button.rect, prev_button.font_size),
                prev_button.font_size,
                "center",
                if prev_enabled {
                    TERM_GREEN_SOFT
                } else {
                    "#b0b0b0"
                },
                "label",
                prev_label,
            );

            render_ui_tile(
                scene,
                next_button.rect,
                BUTTON_RADIUS,
                if next_enabled {
                    if next_hovered {
                        COLOR_GREEN_STROKE_STRONG
                    } else {
                        COLOR_GREEN_STROKE_IDLE
                    }
                } else {
                    COLOR_GRAY_STROKE_DISABLED
                },
            );
            scene.text(
                next_button.rect.x + next_button.rect.w * 0.5,
                button_text_baseline(next_button.rect, next_button.font_size),
                next_button.font_size,
                "center",
                if next_enabled {
                    TERM_GREEN_SOFT
                } else {
                    "#b0b0b0"
                },
                "label",
                next_label,
            );

            scene.text(
                page_status_x,
                page_status_y,
                page_status_size,
                "center",
                TERM_GREEN_DIM,
                "body",
                page_status_label,
            );
        }

        let confirm_enabled = self.ui.rest_selection.is_some();
        let confirm_hovered = confirm_enabled && self.ui.hover == Some(HitTarget::RestConfirm);
        let confirm_font_size = fit_text_size(
            self.tr("Confirm", "Confirmar"),
            26.0,
            (layout.confirm_rect.w - 24.0).max(100.0),
        )
        .max(18.0);
        render_ui_tile(
            scene,
            layout.confirm_rect,
            BUTTON_RADIUS,
            if confirm_enabled {
                if confirm_hovered {
                    COLOR_GREEN_STROKE_STRONG
                } else {
                    COLOR_GREEN_STROKE_START
                }
            } else {
                COLOR_GRAY_STROKE_DISABLED
            },
        );
        scene.text(
            layout.confirm_rect.x + layout.confirm_rect.w * 0.5,
            button_text_baseline(layout.confirm_rect, confirm_font_size),
            confirm_font_size,
            "center",
            if confirm_enabled {
                TERM_GREEN_SOFT
            } else {
                "#b0b0b0"
            },
            "label",
            self.tr("Confirm", "Confirmar"),
        );
    }

    fn render_shop(&self, scene: &mut SceneBuilder) {
        let Some(shop) = self.shop.as_ref() else {
            return;
        };
        let Some(layout) = self.shop_layout() else {
            return;
        };

        let logical_width = self.logical_width();
        let title = self.tr("Shop", "Tienda");
        let subtitle = self.tr("Buy 1 card", "Compra 1 carta");
        let credits = self
            .dungeon
            .as_ref()
            .map(|dungeon| dungeon.credits)
            .unwrap_or(0);
        let credits_line = shop_credits_label(credits, self.language);
        let title_size = fit_text_size(title, 42.0, (logical_width - 48.0).max(120.0)).max(24.0);
        let subtitle_size =
            fit_text_size(subtitle, 18.0, (logical_width - 48.0).max(120.0)).max(12.0);
        let credits_size =
            fit_text_size(&credits_line, 18.0, (logical_width - 48.0).max(120.0)).max(12.0);
        let cards_top = layout
            .card_rects
            .iter()
            .map(|rect| rect.y)
            .fold(f32::INFINITY, f32::min);
        let title_y = (cards_top - 58.0).max(74.0);

        scene.text(
            logical_width * 0.5,
            title_y,
            title_size,
            "center",
            TERM_GREEN_SOFT,
            "display",
            title,
        );
        scene.text(
            logical_width * 0.5,
            title_y + 28.0,
            subtitle_size,
            "center",
            TERM_CYAN_SOFT,
            "label",
            subtitle,
        );
        for (index, rect) in layout.card_rects.iter().enumerate() {
            let Some(offer) = shop.offers.get(index).copied() else {
                continue;
            };
            let affordable = credits >= offer.price;
            let hovered = affordable && self.ui.hover == Some(HitTarget::ShopCard(index));
            let stroke = if !affordable {
                COLOR_GRAY_STROKE_DISABLED
            } else if hovered {
                COLOR_CYAN_STROKE_STRONG
            } else {
                COLOR_CYAN_STROKE_IDLE
            };
            self.render_shop_card(scene, *rect, offer, stroke, affordable);
            scene.text(
                rect.x + rect.w * 0.5,
                layout.price_ys[index],
                18.0,
                "center",
                if affordable {
                    TERM_CYAN_SOFT
                } else {
                    "#b0b0b0"
                },
                "label",
                &credits_label(offer.price, self.language),
            );
        }

        scene.text(
            logical_width * 0.5,
            layout.credits_y,
            credits_size,
            "center",
            TERM_GREEN_TEXT,
            "label",
            &credits_line,
        );

        let leave_hovered = self.ui.hover == Some(HitTarget::ShopLeave);
        render_ui_tile(
            scene,
            layout.leave_button,
            BUTTON_RADIUS,
            if leave_hovered {
                COLOR_GREEN_STROKE_STRONG
            } else {
                COLOR_GREEN_STROKE_START
            },
        );
        scene.text(
            layout.leave_button.x + layout.leave_button.w * 0.5,
            button_text_baseline(layout.leave_button, START_BUTTON_FONT_SIZE),
            START_BUTTON_FONT_SIZE,
            "center",
            TERM_GREEN_SOFT,
            "label",
            self.tr(SHOP_LEAVE_LABEL, "Salir"),
        );
    }

    fn render_reward(&self, scene: &mut SceneBuilder) {
        let Some(reward) = self.reward.as_ref() else {
            return;
        };
        let Some(layout) = self.reward_layout() else {
            return;
        };

        let logical_width = self.logical_width();
        let title = self.tr("Add a card", "Añade una carta");
        let subtitle = reward_tier_label(reward.tier, self.language);
        let credits_line = reward_credits_label(reward.tier, self.language);
        let title_size = fit_text_size(title, 42.0, (logical_width - 48.0).max(120.0)).max(24.0);
        let subtitle_size =
            fit_text_size(subtitle, 18.0, (logical_width - 48.0).max(120.0)).max(12.0);
        let credits_size =
            fit_text_size(&credits_line, 18.0, (logical_width - 48.0).max(120.0)).max(12.0);
        let cards_top = layout
            .card_rects
            .iter()
            .map(|rect| rect.y)
            .fold(f32::INFINITY, f32::min);
        let title_y = (cards_top - 58.0).max(74.0);

        scene.text(
            logical_width * 0.5,
            title_y,
            title_size,
            "center",
            TERM_GREEN_SOFT,
            "display",
            title,
        );
        scene.text(
            logical_width * 0.5,
            title_y + 28.0,
            subtitle_size,
            "center",
            reward_tier_color(reward.tier),
            "label",
            subtitle,
        );
        for (index, rect) in layout.card_rects.iter().enumerate() {
            let Some(card) = reward.options.get(index).copied() else {
                continue;
            };
            let hovered = self.ui.hover == Some(HitTarget::RewardCard(index));
            let stroke = if hovered {
                reward_tier_hover_stroke(reward.tier)
            } else {
                reward_tier_stroke(reward.tier)
            };
            self.render_selection_card(scene, *rect, card, stroke);
        }

        scene.text(
            logical_width * 0.5,
            layout.credits_y,
            credits_size,
            "center",
            TERM_CYAN_SOFT,
            "label",
            &credits_line,
        );

        let skip_hovered = self.ui.hover == Some(HitTarget::RewardSkip);
        render_ui_tile(
            scene,
            layout.skip_button,
            BUTTON_RADIUS,
            if skip_hovered {
                COLOR_GREEN_STROKE_STRONG
            } else {
                COLOR_GREEN_STROKE_START
            },
        );
        scene.text(
            layout.skip_button.x + layout.skip_button.w * 0.5,
            button_text_baseline(layout.skip_button, START_BUTTON_FONT_SIZE),
            START_BUTTON_FONT_SIZE,
            "center",
            TERM_GREEN_SOFT,
            "label",
            self.tr(REWARD_SKIP_LABEL, "Saltar"),
        );
    }

    fn render_selection_card(
        &self,
        scene: &mut SceneBuilder,
        rect: Rect,
        card: CardId,
        stroke: &str,
    ) {
        let def = self.localized_card_def(card);
        let metrics = card_box_metrics(rect.w);
        let title_lines = wrap_text(def.name, metrics.title_chars);

        render_ui_tile(scene, rect, CARD_RADIUS, stroke);

        let title_x = rect.x + metrics.pad_x;
        let mut title_y = rect.y + metrics.top_pad + metrics.title_size;
        for (line_index, line) in title_lines.iter().enumerate() {
            scene.text(
                title_x,
                title_y,
                metrics.title_size,
                "left",
                card_banner_color(card),
                "label",
                line,
            );
            if line_index + 1 < title_lines.len() {
                title_y += metrics.title_size + metrics.title_gap;
            }
        }
        scene.text(
            rect.x + rect.w - metrics.pad_x,
            rect.y + metrics.top_pad + metrics.cost_size * 0.82,
            metrics.cost_size,
            "right",
            TERM_CYAN,
            "display",
            &def.cost.to_string(),
        );

        let title_height = if title_lines.is_empty() {
            0.0
        } else {
            metrics.title_size * title_lines.len() as f32
                + metrics.title_gap * title_lines.len().saturating_sub(1) as f32
        };
        let header_height = title_height.max(metrics.cost_size);
        let body_y =
            rect.y + metrics.top_pad + header_height + metrics.title_body_gap + metrics.body_size;
        render_card_description(
            scene,
            title_x,
            body_y,
            metrics.body_size,
            metrics.body_gap,
            def.description,
            metrics.body_max_width,
            TERM_GREEN_TEXT,
        );
    }

    fn render_module_tile(
        &self,
        scene: &mut SceneBuilder,
        rect: Rect,
        module: ModuleId,
        stroke: &str,
    ) {
        let def = self.localized_module_def(module);
        let metrics = module_box_metrics(rect.w);
        let title_lines = wrap_text(def.name, metrics.title_chars);
        let body_lines = wrap_text(def.description, metrics.body_chars);

        render_ui_tile(scene, rect, CARD_RADIUS, stroke);

        let title_x = rect.x + metrics.pad_x;
        let mut title_y = rect.y + metrics.top_pad + metrics.title_size;
        for (line_index, line) in title_lines.iter().enumerate() {
            scene.text(
                title_x,
                title_y,
                metrics.title_size,
                "left",
                module_accent_color(module),
                "label",
                line,
            );
            if line_index + 1 < title_lines.len() {
                title_y += metrics.title_size + metrics.title_gap;
            }
        }
        let title_height = if title_lines.is_empty() {
            0.0
        } else {
            metrics.title_size * title_lines.len() as f32
                + metrics.title_gap * title_lines.len().saturating_sub(1) as f32
        };
        let header_height = title_height;
        let mut body_y =
            rect.y + metrics.top_pad + header_height + metrics.title_body_gap + metrics.body_size;
        for line in body_lines {
            scene.text(
                title_x,
                body_y,
                metrics.body_size,
                "left",
                TERM_GREEN_TEXT,
                "body",
                &line,
            );
            body_y += metrics.body_size + metrics.body_gap;
        }
    }

    fn render_event_choice_tile(
        &self,
        scene: &mut SceneBuilder,
        rect: Rect,
        title: &str,
        body: &str,
        style: EventChoiceTileStyle<'_>,
    ) {
        let metrics = event_box_metrics(rect.w);
        let title_lines = wrap_text(title, metrics.title_chars);
        let body_lines = wrap_text(body, metrics.body_chars);

        render_ui_tile(scene, rect, CARD_RADIUS, style.stroke);

        let title_x = rect.x + metrics.pad_x;
        let mut title_y = rect.y + metrics.top_pad + metrics.title_size;
        for (line_index, line) in title_lines.iter().enumerate() {
            scene.text(
                title_x,
                title_y,
                metrics.title_size,
                "left",
                style.title_color,
                "label",
                line,
            );
            if line_index + 1 < title_lines.len() {
                title_y += metrics.title_size + metrics.title_gap;
            }
        }

        let title_height = if title_lines.is_empty() {
            0.0
        } else {
            metrics.title_size * title_lines.len() as f32
                + metrics.title_gap * title_lines.len().saturating_sub(1) as f32
        };
        let mut body_y =
            rect.y + metrics.top_pad + title_height + metrics.title_body_gap + metrics.body_size;
        for line in body_lines {
            scene.text(
                title_x,
                body_y,
                metrics.body_size,
                "left",
                style.body_color,
                "body",
                &line,
            );
            body_y += metrics.body_size + metrics.body_gap;
        }
    }

    fn render_shop_card(
        &self,
        scene: &mut SceneBuilder,
        rect: Rect,
        offer: ShopOffer,
        stroke: &str,
        affordable: bool,
    ) {
        let def = self.localized_card_def(offer.card);
        let metrics = card_box_metrics(rect.w);
        let title_lines = wrap_text(def.name, metrics.title_chars);
        let title_color = if affordable {
            card_banner_color(offer.card)
        } else {
            "#b8b8b8"
        };
        let body_color = if affordable {
            TERM_GREEN_TEXT
        } else {
            "#9a9a9a"
        };
        let cost_color = if affordable { TERM_CYAN } else { "#d0d0d0" };

        render_ui_tile(scene, rect, CARD_RADIUS, stroke);

        let title_x = rect.x + metrics.pad_x;
        let mut title_y = rect.y + metrics.top_pad + metrics.title_size;
        for (line_index, line) in title_lines.iter().enumerate() {
            scene.text(
                title_x,
                title_y,
                metrics.title_size,
                "left",
                title_color,
                "label",
                line,
            );
            if line_index + 1 < title_lines.len() {
                title_y += metrics.title_size + metrics.title_gap;
            }
        }
        scene.text(
            rect.x + rect.w - metrics.pad_x,
            rect.y + metrics.top_pad + metrics.cost_size * 0.82,
            metrics.cost_size,
            "right",
            cost_color,
            "display",
            &def.cost.to_string(),
        );

        let title_height = if title_lines.is_empty() {
            0.0
        } else {
            metrics.title_size * title_lines.len() as f32
                + metrics.title_gap * title_lines.len().saturating_sub(1) as f32
        };
        let header_height = title_height.max(metrics.cost_size);
        let body_y =
            rect.y + metrics.top_pad + header_height + metrics.title_body_gap + metrics.body_size;
        render_card_description(
            scene,
            title_x,
            body_y,
            metrics.body_size,
            metrics.body_gap,
            def.description,
            metrics.body_max_width,
            body_color,
        );
    }

    fn render_map(&self, scene: &mut SceneBuilder) {
        let Some(dungeon) = &self.dungeon else {
            return;
        };
        let Some(layout) = self.map_layout() else {
            return;
        };

        let menu_hovered = self.ui.hover == Some(HitTarget::Menu);
        render_ui_tile(
            scene,
            layout.menu_button,
            BUTTON_RADIUS,
            if menu_hovered {
                COLOR_GREEN_STROKE_STRONG
            } else {
                COLOR_GREEN_STROKE_IDLE
            },
        );
        scene.text(
            layout.menu_button.x + layout.menu_button.w * 0.5,
            button_text_baseline(layout.menu_button, 20.0),
            20.0,
            "center",
            if menu_hovered {
                TERM_GREEN_SOFT
            } else {
                TERM_GREEN_TEXT
            },
            "label",
            self.tr("Menu", "Menú"),
        );

        let info_hovered = self.ui.hover == Some(HitTarget::Info);
        let info_active = self.run_info_visible() || self.ui.run_info_open;
        render_ui_tile(
            scene,
            layout.info_button,
            BUTTON_RADIUS,
            if info_hovered || info_active {
                COLOR_GREEN_STROKE_STRONG
            } else {
                COLOR_GREEN_STROKE_IDLE
            },
        );
        scene.text(
            layout.info_button.x + layout.info_button.w * 0.5,
            button_text_baseline(layout.info_button, 20.0),
            20.0,
            "center",
            if info_hovered || info_active {
                TERM_GREEN_SOFT
            } else {
                TERM_GREEN_TEXT
            },
            "label",
            MAP_INFO_LABEL,
        );

        let legend_hovered = self.ui.hover == Some(HitTarget::Legend);
        let legend_active = self.legend_visible() || self.ui.legend_open;
        render_ui_tile(
            scene,
            layout.legend_button,
            BUTTON_RADIUS,
            if legend_hovered || legend_active {
                COLOR_GREEN_STROKE_STRONG
            } else {
                COLOR_GREEN_STROKE_IDLE
            },
        );
        scene.text(
            layout.legend_button.x + layout.legend_button.w * 0.5,
            button_text_baseline(layout.legend_button, 20.0),
            20.0,
            "center",
            if legend_hovered || legend_active {
                TERM_GREEN_SOFT
            } else {
                TERM_GREEN_TEXT
            },
            "label",
            LEGEND_BUTTON_LABEL,
        );

        if self.debug_mode {
            if let Some(button) = layout.debug_level_down_button {
                let hovered = self.ui.hover == Some(HitTarget::DebugLevelDown);
                render_ui_tile(
                    scene,
                    button,
                    BUTTON_RADIUS,
                    if hovered {
                        COLOR_GREEN_STROKE_STRONG
                    } else {
                        COLOR_GREEN_STROKE_IDLE
                    },
                );
                scene.text(
                    button.x + button.w * 0.5,
                    button_text_baseline(button, MAP_DEBUG_BUTTON_FONT_SIZE),
                    MAP_DEBUG_BUTTON_FONT_SIZE,
                    "center",
                    if hovered {
                        TERM_GREEN_SOFT
                    } else {
                        TERM_GREEN_TEXT
                    },
                    "label",
                    "<",
                );
            }
            if let Some(button) = layout.debug_level_up_button {
                let hovered = self.ui.hover == Some(HitTarget::DebugLevelUp);
                render_ui_tile(
                    scene,
                    button,
                    BUTTON_RADIUS,
                    if hovered {
                        COLOR_GREEN_STROKE_STRONG
                    } else {
                        COLOR_GREEN_STROKE_IDLE
                    },
                );
                scene.text(
                    button.x + button.w * 0.5,
                    button_text_baseline(button, MAP_DEBUG_BUTTON_FONT_SIZE),
                    MAP_DEBUG_BUTTON_FONT_SIZE,
                    "center",
                    if hovered {
                        TERM_GREEN_SOFT
                    } else {
                        TERM_GREEN_TEXT
                    },
                    "label",
                    ">",
                );
            }
            if let Some((debug_text_x, debug_text_y)) = layout.debug_level_text_position {
                scene.text(
                    debug_text_x,
                    debug_text_y + MAP_DEBUG_SEED_SIZE * 0.32,
                    MAP_DEBUG_SEED_SIZE,
                    "center",
                    TERM_GREEN_DIM,
                    "body",
                    &debug_map_label(dungeon, self.language),
                );
            }
            if let Some(button) = layout.debug_fill_deck_button {
                let hovered = self.ui.hover == Some(HitTarget::DebugFillDeck);
                render_ui_tile(
                    scene,
                    button,
                    BUTTON_RADIUS,
                    if hovered {
                        COLOR_GREEN_STROKE_STRONG
                    } else {
                        COLOR_GREEN_STROKE_IDLE
                    },
                );
                scene.text(
                    button.x + button.w * 0.5,
                    button_text_baseline(button, MAP_DEBUG_BUTTON_FONT_SIZE),
                    MAP_DEBUG_BUTTON_FONT_SIZE,
                    "center",
                    if hovered {
                        TERM_GREEN_SOFT
                    } else {
                        TERM_GREEN_TEXT
                    },
                    "label",
                    self.tr(MAP_DEBUG_FILL_DECK_LABEL, "Llenar mazo"),
                );
            }
        }

        for edge in &layout.edges {
            let edge_color = if dungeon.is_visited(edge.from_id)
                && (dungeon.is_visited(edge.to_id) || dungeon.is_available(edge.to_id))
            {
                COLOR_WHITE_STROKE_PATH
            } else {
                COLOR_GRAY_STROKE_DISABLED
            };
            scene.line(
                edge.from_x,
                edge.from_y,
                edge.to_x,
                edge.to_y,
                edge_color,
                MAP_LINE_WIDTH,
            );
        }

        for node in &layout.nodes {
            let Some(dungeon_node) = dungeon.node(node.id) else {
                continue;
            };
            let available = dungeon.is_available(node.id);
            let visited = dungeon.is_visited(node.id);
            let hovered = self.ui.hover == Some(HitTarget::MapNode(node.id));
            let pulse = map_available_node_pulse(self.boot_time_ms);
            let pulse_wave = map_available_node_wave(self.boot_time_ms);
            let node_scale = if available {
                1.0 + pulse_wave * 0.08
            } else {
                1.0
            };
            let draw_rect = scale_rect_from_center(node.rect, node_scale);
            let stroke_width = if available { 2.2 + pulse * 0.9 } else { 2.0 };
            let glyph_size = if available { 18.0 + pulse * 2.0 } else { 18.0 };
            let stroke = if available && hovered {
                room_hover_stroke(dungeon_node.kind)
            } else if available {
                room_pulse_stroke(dungeon_node.kind, pulse)
            } else if visited {
                room_visited_stroke(dungeon_node.kind)
            } else {
                room_muted_stroke(dungeon_node.kind)
            };
            let text_color = if available {
                room_pulse_text_color(dungeon_node.kind, pulse)
            } else if visited {
                room_visited_text_color(dungeon_node.kind)
            } else {
                room_muted_text_color(dungeon_node.kind)
            };

            scene.rect(
                draw_rect,
                draw_rect.w * 0.5,
                COLOR_TILE_FILL,
                &stroke,
                stroke_width,
            );
            self.render_map_node_symbol(
                scene,
                dungeon_node.kind,
                dungeon.boss_symbol_sides(),
                MapNodeSymbolLayout {
                    center_x: node.center_x,
                    center_y: node.center_y,
                    radius: glyph_size * 0.42,
                },
                &text_color,
            );
        }

        if self.legend_visible() || self.ui.legend_open {
            self.render_map_legend(scene, &layout);
        }
    }

    fn render_map_node_symbol(
        &self,
        scene: &mut SceneBuilder,
        kind: RoomKind,
        boss_sides: usize,
        layout: MapNodeSymbolLayout,
        color: &str,
    ) {
        let MapNodeSymbolLayout {
            center_x,
            center_y,
            radius,
        } = layout;
        match kind {
            RoomKind::Start => scene.rect(
                Rect {
                    x: center_x - radius,
                    y: center_y - radius,
                    w: radius * 2.0,
                    h: radius * 2.0,
                },
                radius,
                color,
                "transparent",
                0.0,
            ),
            RoomKind::Combat => scene.regular_polygon(
                center_x,
                center_y,
                radius,
                4,
                45.0,
                color,
                "transparent",
                0.0,
            ),
            RoomKind::Elite => scene.regular_polygon(
                center_x,
                center_y,
                radius,
                4,
                0.0,
                color,
                "transparent",
                0.0,
            ),
            RoomKind::Rest => scene.regular_polygon(
                center_x,
                center_y,
                radius,
                3,
                0.0,
                color,
                "transparent",
                0.0,
            ),
            RoomKind::Shop => {
                let total_w = radius * std::f32::consts::SQRT_2;
                let body_h = total_w * 0.5;
                let roof_h = body_h;
                let body_top = center_y;
                scene.rect(
                    Rect {
                        x: center_x - total_w * 0.5,
                        y: body_top,
                        w: total_w,
                        h: body_h,
                    },
                    0.0,
                    color,
                    "transparent",
                    0.0,
                );
                scene.triangle(
                    center_x,
                    body_top - roof_h,
                    center_x + total_w * 0.5,
                    body_top,
                    center_x - total_w * 0.5,
                    body_top,
                    color,
                    "transparent",
                    0.0,
                );
            }
            RoomKind::Event => scene.text(
                center_x,
                center_y + radius * 0.84,
                radius * 2.1,
                "center",
                color,
                "display",
                "?",
            ),
            RoomKind::Boss => {
                let sides = boss_sides.clamp(5, 7);
                let rotation = if sides % 2 == 0 { 30.0 } else { 0.0 };
                scene.regular_polygon(
                    center_x,
                    center_y,
                    radius,
                    sides,
                    rotation,
                    color,
                    "transparent",
                    0.0,
                )
            }
        }
    }

    fn render_map_legend(&self, scene: &mut SceneBuilder, layout: &MapLayout) {
        let rect = layout.legend_modal;
        let progress = self.legend_eased_progress();
        if progress <= 0.0 {
            return;
        }
        let backdrop_rect = Rect {
            x: 0.0,
            y: 0.0,
            w: self.logical_width(),
            h: self.logical_height(),
        };
        let entries = map_legend_entries(self.language);
        let boss_sides = self
            .dungeon
            .as_ref()
            .map(|dungeon| dungeon.boss_symbol_sides())
            .unwrap_or(5);
        let insets = tile_insets_for_card_width(220.0);
        let title_size = 24.0;
        let label_size = 20.0;
        let symbol_radius = 10.0;
        let symbol_gap = 16.0;
        let title_gap = 16.0;
        let row_gap = 12.0;
        let title_y = rect.y + insets.top_pad + title_size;
        let first_row_center_y =
            rect.y + insets.top_pad + title_size + title_gap + label_size * 0.5;
        let row_step = label_size + row_gap;
        let symbol_x = rect.x + insets.pad_x + symbol_radius;
        let label_x = symbol_x + symbol_radius + symbol_gap;

        scene.blur_rect(backdrop_rect, 0.0, LEGEND_BACKDROP_BLUR * progress);
        scene.rect(
            Rect {
                x: 0.0,
                y: 0.0,
                w: self.logical_width(),
                h: self.logical_height(),
            },
            0.0,
            &rgba((0, 0, 0), 0.18 * progress),
            "transparent",
            0.0,
        );
        scene.push_layer(
            progress,
            0.0,
            (1.0 - progress) * 12.0,
            0.965 + progress * 0.035,
        );
        render_ui_tile(scene, rect, BUTTON_RADIUS, COLOR_GREEN_STROKE_PANEL);
        scene.text(
            rect.x + rect.w * 0.5,
            title_y,
            title_size,
            "center",
            TERM_GREEN_SOFT,
            "label",
            self.tr("Legend", "Leyenda"),
        );

        for (index, (kind, label)) in entries.iter().enumerate() {
            let row_center_y = first_row_center_y + index as f32 * row_step;
            let symbol_color = rgba(room_rgb(*kind), 0.92);
            self.render_map_node_symbol(
                scene,
                *kind,
                boss_sides,
                MapNodeSymbolLayout {
                    center_x: symbol_x,
                    center_y: row_center_y,
                    radius: symbol_radius,
                },
                &symbol_color,
            );
            scene.text(
                label_x,
                row_center_y + label_size * 0.32,
                label_size,
                "left",
                TERM_GREEN_TEXT,
                "body",
                label,
            );
        }
        scene.pop_layer();
    }

    fn render_combat(&self, scene: &mut SceneBuilder) {
        let layout = self.layout();
        self.render_menu_button(scene, &layout);
        self.render_end_turn_button(scene, &layout);
        self.render_end_battle_button(scene, &layout);
        for &enemy_index in &layout.enemy_indices {
            self.render_enemy_panel(scene, &layout, enemy_index);
        }
        self.render_hand(scene, &layout);
        self.render_player_panel(scene, &layout);
        self.render_hand_target_hint(scene, &layout);
        self.render_turn_banner(scene, &layout);
    }

    fn render_menu_button(&self, scene: &mut SceneBuilder, layout: &Layout) {
        let rect = layout.menu_button;
        let active = !self.combat_input_locked();
        let hovered = active && self.ui.hover == Some(HitTarget::Menu);
        let font_size = combat_action_button_font_size(layout.low_hand_layout, layout.tile_scale);
        let stroke = if !active {
            COLOR_GRAY_STROKE_DISABLED
        } else if hovered {
            COLOR_GREEN_STROKE_STRONG
        } else {
            COLOR_GREEN_STROKE_IDLE
        };
        render_ui_tile(scene, rect, BUTTON_RADIUS, stroke);
        scene.text(
            rect.x + rect.w * 0.5,
            button_text_baseline(rect, font_size),
            font_size,
            "center",
            if !active {
                TERM_GREEN_DIM
            } else if hovered {
                TERM_GREEN_SOFT
            } else {
                TERM_GREEN_TEXT
            },
            "label",
            self.tr("Menu", "Menú"),
        );
    }

    fn render_enemy_panel(&self, scene: &mut SceneBuilder, layout: &Layout, enemy_index: usize) {
        let Some(rect) = layout.enemy_rect(enemy_index) else {
            return;
        };
        let Some(enemy) = self.combat.enemy(enemy_index) else {
            return;
        };
        let text_x = rect.x + layout.tile_insets.pad_x;
        let targeted = self.ui.selected_card.is_some_and(|index| {
            (self.combat.card_requires_enemy(index) || self.combat.card_targets_all_enemies(index))
                && enemy.fighter.hp > 0
        });
        let metrics = enemy_panel_metrics(
            self,
            enemy_index,
            layout.low_hand_layout,
            layout.tile_scale,
            layout.tile_insets,
        );
        let displayed = self.displayed_actor_stats(Actor::Enemy(enemy_index));
        let top_row_y = rect.y + metrics.top_pad + metrics.stats_size;
        let status_y = top_row_y + metrics.line_gap;
        let info_body_y = status_y
            + metrics.status_row_height
            + if metrics.status_row_height > 0.0 {
                metrics.line_gap
            } else {
                0.0
            }
            + metrics.info_body_size;
        let hover_target = self.ui.hover == Some(HitTarget::Enemy(enemy_index));
        let is_alive = enemy.fighter.hp > 0;
        let enemy_status_labels =
            status_labels(enemy.fighter.statuses, enemy.on_hit_bleed, self.language);
        let status_width = status_row_width(
            &enemy_status_labels,
            metrics.status_size,
            metrics.status_gap,
        );

        let enemy_stroke = if !is_alive {
            COLOR_GRAY_STROKE_DISABLED
        } else if hover_target {
            COLOR_GREEN_STROKE_STRONG
        } else if targeted {
            COLOR_LIME_STROKE_TARGET
        } else {
            COLOR_GREEN_STROKE_PANEL
        };
        let enemy_stroke = actor_panel_flash_stroke(self, Actor::Enemy(enemy_index), enemy_stroke);
        let sprite = enemy_sprite_def(enemy.profile);
        let icon_alpha = enemy_panel_icon_alpha(enemy.profile, is_alive);
        render_ui_tile(scene, rect, ENEMY_PANEL_RADIUS, enemy_stroke);
        let mut top_row_x = text_x;
        let icon_rect = enemy_inline_sprite_rect(
            top_row_x,
            top_row_y,
            metrics.stats_size,
            sprite.width,
            sprite.height,
        );
        for layer in sprite.layers {
            scene.sprite(
                icon_rect,
                layer.code,
                enemy_sprite_layer_color(enemy.profile, layer.tone, is_alive),
                icon_alpha,
            );
        }
        top_row_x += icon_rect.w + combat_inline_group_gap(metrics.stats_size);
        self.render_actor_stats_line(
            scene,
            ActorStatsLineLayout {
                x: top_row_x,
                y: top_row_y,
                size: metrics.stats_size,
                group_gap: combat_inline_group_gap(metrics.stats_size),
            },
            Actor::Enemy(enemy_index),
            displayed.hp,
            enemy.fighter.max_hp,
            displayed.block,
        );
        self.render_status_row_left_sized(
            scene,
            StatusRowLayout {
                x: text_x,
                y: status_y,
                width: status_width,
                size: metrics.status_size,
                gap: metrics.status_gap,
            },
            enemy.fighter.statuses,
            enemy.on_hit_bleed,
        );
        let next_label = self.tr(NEXT_SIGNAL_LABEL, "Siguiente");
        let summary = self.enemy_signal_summary(enemy_index);
        let mut line_y = info_body_y;
        let intent_lines = enemy_intent_lines(next_label, summary, metrics.info_body_chars);
        scene.text(
            text_x,
            line_y,
            metrics.info_body_size,
            "left",
            TERM_GREEN,
            "label",
            &intent_lines.first_line_label,
        );
        if !intent_lines.first_line_summary.is_empty() {
            scene.text(
                text_x + text_width(&intent_lines.first_line_label, metrics.info_body_size),
                line_y,
                metrics.info_body_size,
                "left",
                if is_alive {
                    TERM_CYAN_SOFT
                } else {
                    TERM_GREEN_DIM
                },
                "body",
                &intent_lines.first_line_summary,
            );
        }
        for line in &intent_lines.continuation_lines {
            line_y += metrics.info_body_size + metrics.info_body_line_gap;
            scene.text(
                text_x,
                line_y,
                metrics.info_body_size,
                "left",
                if is_alive {
                    TERM_CYAN_SOFT
                } else {
                    TERM_GREEN_DIM
                },
                "body",
                line,
            );
        }
    }

    fn render_player_panel(&self, scene: &mut SceneBuilder, layout: &Layout) {
        let rect = layout.player_rect;
        let center_x = rect.x + rect.w * 0.5;
        let hovered = self.ui.hover == Some(HitTarget::Player);
        let targeted = self.ui.selected_card.is_some_and(|index| {
            !self.combat.card_requires_enemy(index) && !self.combat.card_targets_all_enemies(index)
        });
        let metrics = player_panel_metrics(
            self,
            layout.low_hand_layout,
            layout.tile_scale,
            layout.tile_insets,
        );
        let stroke = if targeted || hovered {
            COLOR_CYAN_STROKE_TARGET
        } else {
            COLOR_GREEN_STROKE_CARD
        };
        let stroke = actor_panel_flash_stroke(self, Actor::Player, stroke);
        let label_y = rect.y + metrics.top_pad + metrics.label_size;
        let stats_y = label_y + metrics.line_gap + metrics.stats_size;
        let status_y = stats_y + metrics.line_gap;
        let meta_y = status_y
            + metrics.status_row_height
            + if metrics.status_row_height > 0.0 {
                metrics.line_gap
            } else {
                0.0
            }
            + metrics.meta_size;
        let displayed = self.displayed_actor_stats(Actor::Player);
        let player_label = self.tr(PLAYER_NAME, "Jugador");
        let stats_line_width = combat_actor_stats_line_width(
            displayed.hp,
            self.combat.player.fighter.max_hp,
            displayed.block,
            metrics.stats_size,
            combat_inline_group_gap(metrics.stats_size),
        );
        let player_status_labels =
            status_labels(self.combat.player.fighter.statuses, 0, self.language);
        let status_line_width = if player_status_labels.is_empty() {
            0.0
        } else {
            status_row_width(
                &player_status_labels,
                metrics.status_size,
                metrics.status_gap,
            )
        };
        let meta_color = TERM_CYAN;

        render_ui_tile(scene, rect, CARD_RADIUS, stroke);
        scene.text(
            center_x,
            label_y,
            metrics.label_size,
            "center",
            TERM_GREEN_SOFT,
            "label",
            player_label,
        );
        self.render_actor_stats_line(
            scene,
            ActorStatsLineLayout {
                x: center_x - stats_line_width * 0.5,
                y: stats_y,
                size: metrics.stats_size,
                group_gap: combat_inline_group_gap(metrics.stats_size),
            },
            Actor::Player,
            displayed.hp,
            self.combat.player.fighter.max_hp,
            displayed.block,
        );
        self.render_status_row_left_sized(
            scene,
            StatusRowLayout {
                x: center_x - status_line_width * 0.5,
                y: status_y,
                width: status_line_width,
                size: metrics.status_size,
                gap: metrics.status_gap,
            },
            self.combat.player.fighter.statuses,
            0,
        );
        self.render_player_panel_meta_line(
            scene,
            TextLineLayout {
                x: center_x - combat_meta_line_width(self, metrics.meta_size) * 0.5,
                y: meta_y,
                size: metrics.meta_size,
            },
            meta_color,
        );
    }

    fn render_player_panel_meta_line(
        &self,
        scene: &mut SceneBuilder,
        layout: TextLineLayout,
        color: &str,
    ) {
        let displayed_meta = self.displayed_player_meta();
        let energy_text = format!(
            "{}/{}",
            displayed_meta.energy, self.combat.player.max_energy
        );
        let draw_text = displayed_meta.draw_pile.to_string();
        let arrow_text = "→";
        let discard_text = displayed_meta.discard_pile.to_string();
        let energy_color = animated_player_meta_color(self, CombatStat::Energy, color);
        let draw_color = animated_player_meta_color(self, CombatStat::DrawPile, color);
        let discard_color = animated_player_meta_color(self, CombatStat::DiscardPile, color);
        let mut cursor = layout.x;

        render_combat_inline_icon(
            scene,
            &mut cursor,
            layout.y,
            layout.size,
            COMBAT_ENERGY_ICON_ASSET_PATH,
            color,
        );
        scene.text(
            cursor,
            layout.y,
            layout.size,
            "left",
            energy_color,
            "body",
            &energy_text,
        );
        cursor += text_width(&energy_text, layout.size);
        cursor += combat_inline_group_gap(layout.size);
        render_combat_inline_icon(
            scene,
            &mut cursor,
            layout.y,
            layout.size,
            COMBAT_DECK_ICON_ASSET_PATH,
            color,
        );
        scene.text(
            cursor,
            layout.y,
            layout.size,
            "left",
            draw_color,
            "body",
            &draw_text,
        );
        cursor += text_width(&draw_text, layout.size);
        scene.text(
            cursor,
            layout.y,
            layout.size,
            "left",
            color,
            "body",
            arrow_text,
        );
        cursor += text_width(arrow_text, layout.size);
        scene.text(
            cursor,
            layout.y,
            layout.size,
            "left",
            discard_color,
            "body",
            &discard_text,
        );
    }

    fn render_actor_stats_line(
        &self,
        scene: &mut SceneBuilder,
        layout: ActorStatsLineLayout,
        actor: Actor,
        hp: i32,
        max_hp: i32,
        block: i32,
    ) {
        let hp_text = hp.to_string();
        let max_hp_text = format!("/{max_hp}");
        let block_text = block.to_string();
        let hp_color = animated_stat_color(self, actor, CombatStat::Hp);
        let block_color = animated_stat_color(self, actor, CombatStat::Block);
        let mut cursor = layout.x;

        render_combat_inline_icon(
            scene,
            &mut cursor,
            layout.y,
            layout.size,
            COMBAT_HEART_ICON_ASSET_PATH,
            TERM_GREEN_TEXT,
        );
        scene.text(
            cursor,
            layout.y,
            layout.size,
            "left",
            hp_color,
            "body",
            &hp_text,
        );
        cursor += text_width(&hp_text, layout.size);
        scene.text(
            cursor,
            layout.y,
            layout.size,
            "left",
            TERM_GREEN_TEXT,
            "body",
            &max_hp_text,
        );
        cursor += text_width(&max_hp_text, layout.size);
        cursor += layout.group_gap;
        render_combat_inline_icon(
            scene,
            &mut cursor,
            layout.y,
            layout.size,
            COMBAT_SHIELD_ICON_ASSET_PATH,
            TERM_GREEN_TEXT,
        );
        scene.text(
            cursor,
            layout.y,
            layout.size,
            "left",
            block_color,
            "body",
            &block_text,
        );
    }

    fn render_status_row_left_sized(
        &self,
        scene: &mut SceneBuilder,
        layout: StatusRowLayout,
        statuses: StatusSet,
        primed_bleed: u8,
    ) {
        let labels = status_labels(statuses, primed_bleed, self.language);
        if labels.is_empty() {
            return;
        }

        for (row_index, row_labels) in labels.chunks(STATUS_ROW_MAX_COLUMNS).enumerate() {
            let row_width = status_row_chunk_width(row_labels, layout.size, layout.gap);
            let mut cursor = layout.x + (layout.width - row_width).max(0.0) * 0.5;
            let row_y = layout.y
                + row_index as f32 * (layout.size + status_row_line_gap(layout.size))
                + layout.size;
            let last_index = row_labels.len().saturating_sub(1);
            for (index, (label, color)) in row_labels.iter().enumerate() {
                scene.text(cursor, row_y, layout.size, "left", color, "body", label);
                cursor += status_label_width(label, layout.size);
                if index < last_index {
                    cursor += layout.gap;
                }
            }
        }
    }

    fn render_hand(&self, scene: &mut SceneBuilder, layout: &Layout) {
        if self.combat_feedback.playback_kind == Some(CombatPlaybackKind::EnemyTurn) {
            return;
        }

        for (index, rect) in layout.hand_rects.iter().enumerate() {
            let Some(card) = self.combat.hand_card(index) else {
                continue;
            };
            let def = self.localized_card_def(card);
            let description = self.combat_card_description(card);
            let selected = self.ui.selected_card == Some(index);
            let hovered = self.ui.hover == Some(HitTarget::Card(index));
            let playable = !self.combat_input_locked() && self.combat.can_play_card(index);
            let targets_enemy = self.combat.card_requires_enemy(index);
            let targets_all_enemies = self.combat.card_targets_all_enemies(index);
            let targets_hostile = targets_enemy || targets_all_enemies;

            let stroke = if !playable && selected {
                COLOR_GRAY_STROKE_SELECTED
            } else if !playable && hovered {
                COLOR_GRAY_STROKE_HOVER
            } else if !playable {
                COLOR_GRAY_STROKE_DISABLED
            } else if selected && !targets_hostile {
                COLOR_CYAN_STROKE_TARGET
            } else if selected && targets_all_enemies {
                COLOR_GREEN_STROKE_STRONG
            } else if selected {
                COLOR_LIME_STROKE_TARGET
            } else if hovered {
                COLOR_GREEN_STROKE_STRONG
            } else {
                COLOR_GREEN_STROKE_CARD
            };
            let title_color = if playable {
                card_banner_color(card)
            } else {
                "#b8b8b8"
            };
            let body_color = if playable { TERM_GREEN_TEXT } else { "#9a9a9a" };
            let cost_color = if playable { TERM_CYAN } else { "#d0d0d0" };
            let metrics = card_box_metrics(rect.w);
            let pad_x = metrics.pad_x;
            let top_pad = metrics.top_pad;
            let title_size = metrics.title_size;
            let cost_size = metrics.cost_size;
            let body_size = metrics.body_size;
            let body_max_width = metrics.body_max_width;
            let title_gap = metrics.title_gap;
            let title_body_gap = metrics.title_body_gap;
            let body_gap = metrics.body_gap;
            let title_chars = metrics.title_chars;
            let title_lines = wrap_text(def.name, title_chars);

            render_ui_tile(scene, *rect, CARD_RADIUS, stroke);
            let title_x = rect.x + pad_x;
            let mut title_y = rect.y + top_pad + title_size;
            for (line_index, line) in title_lines.iter().enumerate() {
                scene.text(
                    title_x,
                    title_y,
                    title_size,
                    "left",
                    title_color,
                    "label",
                    line,
                );
                if line_index + 1 < title_lines.len() {
                    title_y += title_size + title_gap;
                }
            }
            scene.text(
                rect.x + rect.w - pad_x,
                rect.y + top_pad + cost_size * 0.82,
                cost_size,
                "right",
                cost_color,
                "display",
                &def.cost.to_string(),
            );

            let title_height = if title_lines.is_empty() {
                0.0
            } else {
                title_size * title_lines.len() as f32
                    + title_gap * title_lines.len().saturating_sub(1) as f32
            };
            let header_height = title_height.max(cost_size);
            let line_y = rect.y + top_pad + header_height + title_body_gap + body_size;
            render_card_description(
                scene,
                title_x,
                line_y,
                body_size,
                body_gap,
                &description,
                body_max_width,
                body_color,
            );
        }
    }

    fn render_hand_target_hint(&self, scene: &mut SceneBuilder, layout: &Layout) {
        let Some(hint_rect) = layout.hint_rect else {
            return;
        };
        let (hint_font_size, _, _) = hand_hint_metrics(layout.tile_scale);
        let (message, color, stroke) = combat_hint_tile(self, self.combat.hand_len());
        render_ui_tile(scene, hint_rect, BUTTON_RADIUS, stroke);
        scene.text(
            hint_rect.x + hint_rect.w * 0.5,
            button_text_baseline(hint_rect, hint_font_size),
            hint_font_size,
            "center",
            color,
            "label",
            &message,
        );
    }

    fn render_end_turn_button(&self, scene: &mut SceneBuilder, layout: &Layout) {
        let rect = layout.end_turn_button;
        let active = self.combat.is_player_turn() && !self.combat_input_locked();
        let hovered = self.ui.hover == Some(HitTarget::EndTurn);
        let font_size = combat_action_button_font_size(layout.low_hand_layout, layout.tile_scale);
        let stroke = if !active {
            COLOR_CYAN_STROKE_DISABLED
        } else if hovered {
            COLOR_CYAN_STROKE_STRONG
        } else {
            COLOR_CYAN_STROKE_IDLE
        };
        render_ui_tile(scene, rect, BUTTON_RADIUS, stroke);
        scene.text(
            rect.x + rect.w * 0.5,
            button_text_baseline(rect, font_size),
            font_size,
            "center",
            if active {
                TERM_CYAN_SOFT
            } else {
                TERM_GREEN_DIM
            },
            "label",
            self.tr("End Turn", "Fin del turno"),
        );
    }

    fn turn_banner_rect(&self, layout: &Layout, label: &str) -> Rect {
        let font_size = combat_top_button_font_size(layout.low_hand_layout, layout.tile_scale);
        let (width, height) = button_size(
            label,
            font_size,
            layout.tile_insets.pad_x,
            layout.tile_insets.top_pad,
        );
        let min_y = layout.menu_button.y + layout.menu_button.h + HAND_MIN_GAP * 0.5;
        let enemy_top = layout
            .enemy_rects
            .iter()
            .map(|rect| rect.y)
            .fold(layout.player_rect.y, f32::min);
        let target_y = enemy_top - height - HAND_MIN_GAP;
        Rect {
            x: self.logical_center_x() - width * 0.5,
            y: target_y.max(min_y),
            w: width,
            h: height,
        }
    }

    fn render_turn_banner(&self, scene: &mut SceneBuilder, layout: &Layout) {
        let Some(banner) = self.combat_feedback.turn_banner.as_ref() else {
            return;
        };
        let alpha = (banner.ttl_ms / banner.total_ms).clamp(0.0, 1.0);
        let progress = 1.0 - alpha;
        let text_size = combat_top_button_font_size(layout.low_hand_layout, layout.tile_scale);
        let rect = self.turn_banner_rect(layout, &banner.text);
        scene.push_layer(alpha, 0.0, -6.0 * ease_out_cubic(progress), 1.0);
        render_ui_tile_scaled(
            scene,
            rect,
            BUTTON_RADIUS,
            &rgba(banner.color, 0.78 * alpha),
            alpha,
        );
        scene.text(
            self.logical_center_x(),
            button_text_baseline(rect, text_size),
            text_size,
            "center",
            &rgba(banner.color, alpha),
            "label",
            &banner.text,
        );
        scene.pop_layer();
    }

    fn render_end_battle_button(&self, scene: &mut SceneBuilder, layout: &Layout) {
        let Some(rect) = layout.end_battle_button else {
            return;
        };
        let hovered = self.ui.hover == Some(HitTarget::EndBattle);
        let font_size = combat_action_button_font_size(layout.low_hand_layout, layout.tile_scale);
        let stroke = if hovered {
            COLOR_LIME_STROKE_TARGET
        } else {
            COLOR_GREEN_STROKE_IDLE
        };
        render_ui_tile(scene, rect, BUTTON_RADIUS, stroke);
        scene.text(
            rect.x + rect.w * 0.5,
            button_text_baseline(rect, font_size),
            font_size,
            "center",
            if hovered {
                TERM_LIME_SOFT
            } else {
                TERM_GREEN_TEXT
            },
            "label",
            self.tr("End Battle", "Fin de batalla"),
        );
    }

    fn render_result_overlay(&self, scene: &mut SceneBuilder, outcome: CombatOutcome) {
        scene.rect(
            Rect {
                x: 0.0,
                y: 0.0,
                w: self.logical_width(),
                h: self.logical_height(),
            },
            0.0,
            COLOR_TILE_FILL,
            "transparent",
            0.0,
        );
        let center_x = self.logical_center_x();
        if let Some(summary) = self.final_victory_summary() {
            let title = self.tr("Run Complete", "Partida completada");
            let stats_line = match self.language {
                Language::English => {
                    format!(
                        "{} max HP    {} card deck",
                        summary.player_max_hp, summary.deck_count
                    )
                }
                Language::Spanish => {
                    format!(
                        "{} HP máximo    mazo de {} cartas",
                        summary.player_max_hp, summary.deck_count
                    )
                }
            };
            let seed_line = match self.language {
                Language::English => format!("Seed {}", display_seed(summary.seed)),
                Language::Spanish => format!("Semilla {}", display_seed(summary.seed)),
            };
            let version_line = visible_game_version_label();
            let logo_size =
                (self.logical_width().min(self.logical_height()) * 0.12).clamp(72.0, 104.0);
            let logo_rect = Rect {
                x: center_x - logo_size * 0.5,
                y: self.logical_height() * (156.0 / LOGICAL_HEIGHT),
                w: logo_size,
                h: logo_size,
            };
            let title_size =
                fit_text_size(title, 60.0, (self.logical_width() - 48.0).max(120.0)).max(34.0);
            let stats_size =
                fit_text_size(&stats_line, 18.0, (self.logical_width() - 80.0).max(120.0))
                    .max(12.0);
            let seed_size =
                fit_text_size(&seed_line, 14.0, (self.logical_width() - 80.0).max(120.0)).max(11.0);
            let version_size = fit_text_size(
                &version_line,
                14.0,
                (self.logical_width() - 80.0).max(120.0),
            )
            .max(11.0);

            scene.image(logo_rect, LOGO_ASSET_PATH, 0.96);
            scene.text(
                center_x,
                self.logical_height() * (286.0 / LOGICAL_HEIGHT),
                title_size,
                "center",
                TERM_GREEN,
                "display",
                title,
            );
            scene.text(
                center_x,
                self.logical_height() * (340.0 / LOGICAL_HEIGHT),
                stats_size,
                "center",
                TERM_GREEN_TEXT,
                "body",
                &stats_line,
            );
            scene.text(
                center_x,
                self.logical_height() * (372.0 / LOGICAL_HEIGHT),
                seed_size,
                "center",
                TERM_GREEN_DIM,
                "body",
                &seed_line,
            );
            scene.text(
                center_x,
                self.logical_height() * (398.0 / LOGICAL_HEIGHT),
                version_size,
                "center",
                TERM_GREEN_DIM,
                "body",
                &version_line,
            );
        } else if let Some(summary) = self.defeat_summary() {
            let block_width = (self.logical_width() - 96.0).clamp(260.0, 560.0);
            let defeat_line = match self.language {
                Language::English => {
                    format!("Defeated {}", defeat_by_text(&summary, self.language))
                }
                Language::Spanish => format!("Derrota {}", defeat_by_text(&summary, self.language)),
            };
            let defeat_line_size =
                fit_text_size(&defeat_line, 24.0, (self.logical_width() - 64.0).max(120.0))
                    .max(10.0);
            let line_size = fit_text_size(
                self.tr("12 fights cleared", "12 combates superados"),
                22.0,
                block_width,
            )
            .max(12.0);
            let max_chars = ((block_width / (line_size * 0.62)).floor() as usize).max(16);
            let rows: [(&str, String); 7] = [
                (
                    TERM_GREEN_TEXT,
                    count_cleared_label(
                        summary.bosses_cleared,
                        self.language,
                        "level",
                        "levels",
                        "nivel",
                        "niveles",
                    ),
                ),
                (
                    TERM_GREEN_TEXT,
                    count_cleared_label(
                        summary.combats_cleared,
                        self.language,
                        "fight",
                        "fights",
                        "combate",
                        "combates",
                    ),
                ),
                (
                    TERM_GREEN_TEXT,
                    count_cleared_label(
                        summary.elites_cleared,
                        self.language,
                        "elite",
                        "elites",
                        "élite",
                        "élites",
                    ),
                ),
                (
                    TERM_GREEN_TEXT,
                    count_cleared_label(
                        summary.bosses_cleared,
                        self.language,
                        "boss",
                        "bosses",
                        "jefe",
                        "jefes",
                    ),
                ),
                (
                    TERM_GREEN_TEXT,
                    match self.language {
                        Language::English => format!("{} max HP", summary.player_max_hp),
                        Language::Spanish => format!("{} HP máximo", summary.player_max_hp),
                    },
                ),
                (
                    TERM_GREEN_TEXT,
                    card_deck_label(summary.deck_count, self.language),
                ),
                (
                    TERM_GREEN_DIM,
                    match self.language {
                        Language::English => format!("Seed {}", display_seed(summary.seed)),
                        Language::Spanish => format!("Semilla {}", display_seed(summary.seed)),
                    },
                ),
            ];
            let wrapped_rows: Vec<(&str, Vec<String>)> = rows
                .iter()
                .map(|(color, row)| (*color, wrap_text(row, max_chars)))
                .collect();
            let line_gap = 7.0;
            let row_gap = 10.0;
            let buttons = result_button_layout(
                self.logical_width(),
                self.logical_height(),
                self.final_victory_summary().is_some(),
                self.language,
            );
            let content_bottom = (buttons.menu_button.y - 32.0).max(0.0);
            let row_heights: Vec<f32> = wrapped_rows
                .iter()
                .map(|(_, lines)| {
                    if lines.is_empty() {
                        0.0
                    } else {
                        line_size * lines.len() as f32
                            + line_gap * lines.len().saturating_sub(1) as f32
                    }
                })
                .collect();
            let rows_height = row_heights.iter().sum::<f32>()
                + row_gap * row_heights.len().saturating_sub(1) as f32;
            let total_height = defeat_line_size + 18.0 + rows_height;
            let block_top = ((content_bottom - total_height) * 0.5).max(24.0);
            let mut baseline_y = block_top + defeat_line_size;

            scene.text(
                center_x,
                baseline_y,
                defeat_line_size,
                "center",
                TERM_PINK_SOFT,
                "body",
                &defeat_line,
            );
            baseline_y += defeat_line_size + 18.0;

            for (color, lines) in wrapped_rows {
                for (line_index, line) in lines.iter().enumerate() {
                    scene.text(
                        center_x, baseline_y, line_size, "center", color, "body", line,
                    );
                    if line_index + 1 < lines.len() {
                        baseline_y += line_size + line_gap;
                    }
                }
                baseline_y += line_size + row_gap;
            }
        } else {
            let title = match outcome {
                CombatOutcome::Victory => self.tr("Victory", "Victoria"),
                CombatOutcome::Defeat => self.tr("Defeat", "Derrota"),
            };
            let color = match outcome {
                CombatOutcome::Victory => TERM_GREEN,
                CombatOutcome::Defeat => TERM_PINK_SOFT,
            };
            let title_size =
                fit_text_size(title, 58.0, (self.logical_width() - 48.0).max(120.0)).max(32.0);
            let body = match outcome {
                CombatOutcome::Victory => self.tr("Encounter complete.", "Encuentro completado."),
                CombatOutcome::Defeat => self.tr(
                    "Signal lost. Return to the main menu to try again.",
                    "Señal perdida. Vuelve al menú principal para intentarlo de nuevo.",
                ),
            };
            let body_size =
                fit_text_size(body, 20.0, (self.logical_width() - 48.0).max(120.0)).max(12.0);

            scene.text(
                center_x,
                self.logical_height() * (286.0 / LOGICAL_HEIGHT),
                title_size,
                "center",
                color,
                "display",
                title,
            );
            scene.text(
                center_x,
                self.logical_height() * (336.0 / LOGICAL_HEIGHT),
                body_size,
                "center",
                TERM_GREEN_TEXT,
                "body",
                body,
            );
        }
        let buttons = result_button_layout(
            self.logical_width(),
            self.logical_height(),
            self.final_victory_summary().is_some(),
            self.language,
        );
        let button = buttons.menu_button;
        render_primary_button(
            scene,
            button,
            self.ui.hover == Some(HitTarget::Restart),
            self.tr(RESULT_BUTTON_LABEL, "Menú principal"),
            self.boot_time_ms,
        );
        if let Some(button) = buttons.share_button {
            render_primary_button(
                scene,
                button,
                self.ui.hover == Some(HitTarget::Share),
                self.tr("Share", "Compartir"),
                self.boot_time_ms,
            );
        }
    }

    fn render_floaters(&self, scene: &mut SceneBuilder) {
        for floater in &self.floaters {
            let alpha = (floater.ttl_ms / floater.total_ms).clamp(0.0, 1.0);
            scene.text(
                floater.x,
                floater.y,
                24.0,
                "center",
                &rgba(floater.color, alpha),
                "display",
                &floater.text,
            );
        }
    }

    fn render_pixel_shards(&self, scene: &mut SceneBuilder) {
        for shard in &self.pixel_shards {
            let alpha = (shard.ttl_ms / shard.total_ms).clamp(0.0, 1.0);
            let size = shard.size * (0.72 + alpha * 0.28);
            scene.rect(
                Rect {
                    x: shard.x - size * 0.5,
                    y: shard.y - size * 0.5,
                    w: size,
                    h: size,
                },
                0.0,
                &rgba(shard.color, alpha),
                "transparent",
                0.0,
            );
        }
    }

    fn spawn_card_style_pixel_burst(&mut self, rect: Rect, base: (u8, u8, u8)) {
        self.spawn_rect_pixel_burst(rect, base, 10, 4, 0.24);
    }

    fn spawn_card_pixel_burst(&mut self, rect: Rect, card: CardId) {
        let base = card_banner_rgb(card);
        self.spawn_card_style_pixel_burst(rect, base);
    }

    fn spawn_enemy_pixel_burst(&mut self, rect: Rect) {
        self.spawn_card_style_pixel_burst(rect, (51, 255, 102));
    }

    fn spawn_random_victory_burst(&mut self) {
        let seed = self
            .boot_time_ms
            .to_bits()
            .wrapping_add(self.pixel_shards.len() as u32 * 37)
            .wrapping_add(self.restart_count as u32 * 53);
        let width = self.logical_width();
        let height = self.logical_height();
        let burst_w = 44.0 + noise01(seed.wrapping_add(1)) * 92.0;
        let burst_h = 36.0 + noise01(seed.wrapping_add(2)) * 72.0;
        let center_x = width * (0.16 + noise01(seed.wrapping_add(3)) * 0.68);
        let center_y = height * (0.14 + noise01(seed.wrapping_add(4)) * 0.42);
        let base = match seed % 4 {
            0 => (51, 255, 102),
            1 => (61, 245, 255),
            2 => (216, 255, 61),
            _ => (255, 79, 216),
        };
        let cols = 4 + ((seed >> 3) % 3) as usize;
        let rows = 3 + ((seed >> 5) % 2) as usize;
        self.spawn_rect_pixel_burst(
            Rect {
                x: center_x - burst_w * 0.5,
                y: center_y - burst_h * 0.5,
                w: burst_w,
                h: burst_h,
            },
            base,
            cols,
            rows,
            0.26,
        );
    }

    fn spawn_rect_pixel_burst(
        &mut self,
        rect: Rect,
        base: (u8, u8, u8),
        cols: usize,
        rows: usize,
        size_ratio: f32,
    ) {
        let center_x = rect.x + rect.w * 0.5;
        let center_y = rect.y + rect.h * 0.5;
        let bright = mix_rgb(base, (255, 255, 255), 0.35);
        let pale = mix_rgb(base, (255, 255, 255), 0.6);
        let cell_w = rect.w / cols as f32;
        let cell_h = rect.h / rows as f32;
        let shard_size = (cell_w.min(cell_h) * size_ratio).clamp(1.5, 5.0);

        for row in 0..rows {
            for col in 0..cols {
                let index = row * cols + col;
                let x = rect.x + cell_w * (col as f32 + 0.5);
                let y = rect.y + cell_h * (row as f32 + 0.5);
                let dir_x = ((x - center_x) / (rect.w * 0.5).max(1.0)).clamp(-1.0, 1.0);
                let dir_y = ((y - center_y) / (rect.h * 0.5).max(1.0)).clamp(-1.0, 1.0);
                let jitter_x = noise_signed(index as u32 * 13 + 7) * 0.55;
                let jitter_y = noise_signed(index as u32 * 17 + 11) * 0.55;
                let speed = 0.10 + noise01(index as u32 * 19 + 5) * 0.10;
                let ttl_ms = 150.0 + noise01(index as u32 * 23 + 3) * 90.0;
                let color = match index % 3 {
                    0 => base,
                    1 => bright,
                    _ => pale,
                };

                self.pixel_shards.push(PixelShard {
                    x,
                    y,
                    vx: (dir_x + jitter_x) * speed,
                    vy: (dir_y + jitter_y - 0.1) * speed,
                    size: shard_size,
                    ttl_ms,
                    total_ms: ttl_ms,
                    color,
                });
            }
        }
    }
}

fn card_banner_color(card: CardId) -> &'static str {
    match card_def(card).archetype {
        CardArchetype::Pressure => TERM_GREEN,
        CardArchetype::Bulwark => TERM_BLUE_SOFT,
        CardArchetype::Momentum => TERM_CYAN,
        CardArchetype::Fabricate => TERM_ORANGE,
        CardArchetype::Sweep => TERM_LIME_SOFT,
        CardArchetype::Burst => TERM_PINK_SOFT,
    }
}

fn card_banner_rgb(card: CardId) -> (u8, u8, u8) {
    match card_def(card).archetype {
        CardArchetype::Pressure => (51, 255, 102),
        CardArchetype::Bulwark => (155, 183, 255),
        CardArchetype::Momentum => (61, 245, 255),
        CardArchetype::Fabricate => (255, 184, 82),
        CardArchetype::Sweep => (235, 255, 154),
        CardArchetype::Burst => (255, 156, 240),
    }
}

fn module_accent_rgb(module: ModuleId) -> (u8, u8, u8) {
    match module {
        ModuleId::AegisDrive => (61, 245, 255),
        ModuleId::TargetingRelay => (216, 255, 61),
        ModuleId::Nanoforge => (126, 255, 166),
        ModuleId::CapacitorBank => (255, 156, 240),
        ModuleId::PrismScope => (255, 184, 82),
        ModuleId::SalvageLedger => (255, 210, 120),
        ModuleId::OverclockCore => (255, 79, 216),
        ModuleId::SuppressionField => (155, 183, 255),
        ModuleId::RecoveryMatrix => (141, 255, 173),
    }
}

fn module_accent_color(module: ModuleId) -> &'static str {
    match module {
        ModuleId::AegisDrive => TERM_CYAN_SOFT,
        ModuleId::TargetingRelay => TERM_LIME_SOFT,
        ModuleId::Nanoforge => TERM_GREEN_SOFT,
        ModuleId::CapacitorBank => TERM_PINK_SOFT,
        ModuleId::PrismScope => TERM_ORANGE,
        ModuleId::SalvageLedger => TERM_LIME_SOFT,
        ModuleId::OverclockCore => TERM_PINK,
        ModuleId::SuppressionField => TERM_BLUE_SOFT,
        ModuleId::RecoveryMatrix => TERM_GREEN_SOFT,
    }
}

fn module_stroke(module: ModuleId) -> String {
    rgba(module_accent_rgb(module), 0.86)
}

fn module_sort_order(module: ModuleId) -> u8 {
    match module {
        ModuleId::Nanoforge => 0,
        ModuleId::AegisDrive => 1,
        ModuleId::TargetingRelay => 2,
        ModuleId::CapacitorBank => 3,
        ModuleId::PrismScope => 4,
        ModuleId::SalvageLedger => 5,
        ModuleId::OverclockCore => 6,
        ModuleId::SuppressionField => 7,
        ModuleId::RecoveryMatrix => 8,
    }
}

fn reward_tier_label(tier: RewardTier, language: Language) -> &'static str {
    match tier {
        RewardTier::Combat => localized_text(language, "Combat reward", "Recompensa de combate"),
        RewardTier::Elite => localized_text(language, "Elite reward", "Recompensa de élite"),
        RewardTier::Boss => localized_text(language, "Boss reward", "Recompensa de jefe"),
    }
}

fn reward_tier_color(tier: RewardTier) -> &'static str {
    match tier {
        RewardTier::Combat => TERM_GREEN_TEXT,
        RewardTier::Elite => TERM_LIME_SOFT,
        RewardTier::Boss => TERM_PINK_SOFT,
    }
}

fn reward_tier_stroke(tier: RewardTier) -> &'static str {
    match tier {
        RewardTier::Combat => COLOR_GREEN_STROKE_IDLE,
        RewardTier::Elite => COLOR_LIME_STROKE_TARGET,
        RewardTier::Boss => COLOR_CYAN_STROKE_TARGET,
    }
}

fn reward_tier_hover_stroke(tier: RewardTier) -> &'static str {
    match tier {
        RewardTier::Combat => COLOR_GREEN_STROKE_STRONG,
        RewardTier::Elite => COLOR_GRAY_STROKE_SELECTED,
        RewardTier::Boss => COLOR_CYAN_STROKE_STRONG,
    }
}

fn mix_rgb(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    let mix = |lhs: u8, rhs: u8| -> u8 {
        (lhs as f32 + (rhs as f32 - lhs as f32) * t)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    (mix(a.0, b.0), mix(a.1, b.1), mix(a.2, b.2))
}

fn noise01(seed: u32) -> f32 {
    let mut x = seed.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
    x = ((x >> ((x >> 28) + 4)) ^ x).wrapping_mul(277_803_737);
    (((x >> 22) ^ x) as f32) / (u32::MAX as f32)
}

fn noise_signed(seed: u32) -> f32 {
    noise01(seed) * 2.0 - 1.0
}

fn displayed_combat_stats(combat: &CombatState) -> DisplayedCombatStats {
    DisplayedCombatStats {
        player: ActorDisplayedStats {
            hp: combat.player.fighter.hp,
            block: combat.player.fighter.block,
        },
        player_meta: PlayerDisplayedMeta {
            energy: combat.player.energy as i32,
            draw_pile: combat.deck.draw_pile.len() as i32,
            discard_pile: combat.deck.discard_pile.len() as i32,
        },
        enemies: combat
            .enemies
            .iter()
            .map(|enemy| ActorDisplayedStats {
                hp: enemy.fighter.hp,
                block: enemy.fighter.block,
            })
            .collect(),
    }
}

fn displayed_enemy_intents(combat: &CombatState, language: Language) -> Vec<EnemyIntent> {
    (0..combat.enemy_count())
        .filter_map(|enemy_index| {
            combat
                .enemy(enemy_index)
                .map(|enemy| localized_enemy_intent(enemy.profile, enemy.intent_index, language))
        })
        .collect()
}

fn stat_countdown_values(from: i32, to: i32) -> Vec<i32> {
    if from == to {
        return Vec::new();
    }

    let delta = (to - from).unsigned_abs() as usize;
    let step = ((delta + COMBAT_STAT_COUNTDOWN_MAX_STEPS.saturating_sub(1))
        / COMBAT_STAT_COUNTDOWN_MAX_STEPS.max(1))
    .max(1) as i32;
    let mut values = Vec::new();
    let mut current = from;

    while current != to {
        current = if current < to {
            (current + step).min(to)
        } else {
            (current - step).max(to)
        };
        values.push(current);
    }

    values
}

fn active_countdown_tint(app: &App, actor: Actor, stat: CombatStat) -> Option<StatTint> {
    app.combat_feedback
        .active_stats
        .iter()
        .find(|active| active.actor == actor && active.stat == stat)
        .map(|active| active.tint)
}

fn first_active_actor_tint(app: &App, actor: Actor) -> Option<StatTint> {
    app.combat_feedback
        .active_stats
        .iter()
        .find(|active| active.actor == actor)
        .map(|active| active.tint)
}

fn animated_stat_color(app: &App, actor: Actor, stat: CombatStat) -> &'static str {
    match active_countdown_tint(app, actor, stat) {
        Some(tint) => match tint {
            StatTint::Damage => TERM_PINK_SOFT,
            StatTint::BlockGain => TERM_GREEN_SOFT,
            StatTint::NeutralLoss => TERM_GREEN_DIM,
        },
        _ => TERM_GREEN_TEXT,
    }
}

fn animated_player_meta_color<'a>(app: &App, stat: CombatStat, default: &'a str) -> &'a str {
    match active_countdown_tint(app, Actor::Player, stat) {
        Some(_) => TERM_CYAN_SOFT,
        _ => default,
    }
}

fn actor_panel_flash_stroke<'a>(app: &App, actor: Actor, default_stroke: &'a str) -> &'a str {
    match first_active_actor_tint(app, actor) {
        Some(tint) => match tint {
            StatTint::Damage => COLOR_PINK_STROKE_STRONG,
            StatTint::BlockGain => COLOR_GREEN_STROKE_STRONG,
            StatTint::NeutralLoss => COLOR_GRAY_STROKE_SELECTED,
        },
        _ => default_stroke,
    }
}

fn status_display_name(status: StatusKind, language: Language) -> &'static str {
    match status {
        StatusKind::Bleed => localized_text(language, "Bleed", "Sangrado"),
        StatusKind::Focus => localized_text(language, "Focus", "Enfoque"),
        StatusKind::Rhythm => localized_text(language, "Rhythm", "Ritmo"),
        StatusKind::Momentum => localized_text(language, "Momentum", "Impulso"),
    }
}

fn status_color(status: StatusKind) -> &'static str {
    match status {
        StatusKind::Bleed => TERM_PINK,
        StatusKind::Focus => TERM_GREEN,
        StatusKind::Rhythm => TERM_BLUE_SOFT,
        StatusKind::Momentum => TERM_CYAN,
    }
}

fn status_display_rgb(status: StatusKind) -> (u8, u8, u8) {
    match status {
        StatusKind::Bleed => (255, 79, 216),
        StatusKind::Focus => (51, 255, 102),
        StatusKind::Rhythm => (155, 183, 255),
        StatusKind::Momentum => (61, 245, 255),
    }
}

fn status_labels(
    statuses: StatusSet,
    primed_bleed: u8,
    language: Language,
) -> Vec<(String, &'static str)> {
    let mut labels = Vec::new();
    if statuses.focus != 0 {
        labels.push((
            axis_status_label(StatusKind::Focus, statuses.focus, language),
            status_color(StatusKind::Focus),
        ));
    }
    if statuses.rhythm != 0 {
        labels.push((
            axis_status_label(StatusKind::Rhythm, statuses.rhythm, language),
            status_color(StatusKind::Rhythm),
        ));
    }
    if statuses.momentum != 0 {
        labels.push((
            axis_status_label(StatusKind::Momentum, statuses.momentum, language),
            status_color(StatusKind::Momentum),
        ));
    }
    if statuses.bleed > 0 {
        labels.push((
            format!(
                "{} {}",
                status_display_name(StatusKind::Bleed, language),
                statuses.bleed
            ),
            status_color(StatusKind::Bleed),
        ));
    }
    if primed_bleed > 0 {
        labels.push((
            match language {
                Language::English => format!(
                    "Next hit: {} {primed_bleed}",
                    status_display_name(StatusKind::Bleed, language)
                ),
                Language::Spanish => format!(
                    "Prox. golpe: {} {primed_bleed}",
                    status_display_name(StatusKind::Bleed, language)
                ),
            },
            TERM_PINK_SOFT,
        ));
    }
    labels
}

fn axis_status_label(status: StatusKind, value: i8, language: Language) -> String {
    format!(
        "{}{}",
        status_display_name(status, language),
        signed_axis_value(value)
    )
}

fn axis_display_name(axis: AxisKind, language: Language) -> &'static str {
    match axis {
        AxisKind::Focus => localized_text(language, "Focus", "Enfoque"),
        AxisKind::Rhythm => localized_text(language, "Rhythm", "Ritmo"),
        AxisKind::Momentum => localized_text(language, "Momentum", "Impulso"),
    }
}

fn signed_axis_value(value: i8) -> String {
    if value >= 0 {
        format!("+{value}")
    } else {
        value.to_string()
    }
}

fn map_legend_entries(language: Language) -> [(RoomKind, &'static str); 7] {
    [
        (RoomKind::Start, room_kind_label(RoomKind::Start, language)),
        (
            RoomKind::Combat,
            room_kind_label(RoomKind::Combat, language),
        ),
        (RoomKind::Elite, room_kind_label(RoomKind::Elite, language)),
        (RoomKind::Rest, room_kind_label(RoomKind::Rest, language)),
        (RoomKind::Shop, room_kind_label(RoomKind::Shop, language)),
        (RoomKind::Event, room_kind_label(RoomKind::Event, language)),
        (RoomKind::Boss, room_kind_label(RoomKind::Boss, language)),
    ]
}

fn room_kind_label(kind: RoomKind, language: Language) -> &'static str {
    match kind {
        RoomKind::Start => localized_text(language, "Start", "Inicio"),
        RoomKind::Combat => localized_text(language, "Fight", "Combate"),
        RoomKind::Elite => localized_text(language, "Elite", "Élite"),
        RoomKind::Rest => localized_text(language, "Rest", "Descanso"),
        RoomKind::Shop => localized_text(language, "Shop", "Tienda"),
        RoomKind::Event => localized_text(language, "Event", "Evento"),
        RoomKind::Boss => localized_text(language, "Boss", "Jefe"),
    }
}

fn count_label(
    count: usize,
    language: Language,
    english_singular: &str,
    english_plural: &str,
    spanish_singular: &str,
    spanish_plural: &str,
) -> String {
    if count == 1 {
        format!(
            "1 {}",
            localized_text(language, english_singular, spanish_singular)
        )
    } else {
        format!(
            "{count} {}",
            localized_text(language, english_plural, spanish_plural)
        )
    }
}

fn count_cleared_label(
    count: usize,
    language: Language,
    english_singular: &str,
    english_plural: &str,
    spanish_singular: &str,
    spanish_plural: &str,
) -> String {
    match language {
        Language::English => format!(
            "{} cleared",
            count_label(
                count,
                language,
                english_singular,
                english_plural,
                spanish_singular,
                spanish_plural
            )
        ),
        Language::Spanish => format!(
            "{} superado",
            count_label(
                count,
                language,
                english_singular,
                english_plural,
                spanish_singular,
                spanish_plural
            )
        ),
    }
}

fn credits_label(credits: u32, language: Language) -> String {
    count_label(
        credits as usize,
        language,
        "Credit",
        "Credits",
        "Crédito",
        "Créditos",
    )
}

fn card_deck_label(deck_count: usize, language: Language) -> String {
    match language {
        Language::English => format!("{deck_count} Card Deck"),
        Language::Spanish => format!("Mazo de {deck_count} cartas"),
    }
}

fn shop_credits_label(credits: u32, language: Language) -> String {
    match language {
        Language::English => format!("You have {}", credits_label(credits, language)),
        Language::Spanish => format!("Tienes {}", credits_label(credits, language)),
    }
}

fn reward_credits_label(tier: RewardTier, language: Language) -> String {
    let credits = match tier {
        RewardTier::Combat => credits_reward_for_room(RoomKind::Combat),
        RewardTier::Elite => credits_reward_for_room(RoomKind::Elite),
        RewardTier::Boss => credits_reward_for_room(RoomKind::Boss),
    };
    format!("+{}", credits_label(credits, language))
}

fn defeat_by_text(summary: &DefeatSummary, language: Language) -> String {
    match (summary.failure_enemy, summary.failure_room) {
        (Some(enemy_name), _) => match language {
            Language::English => format!("by {enemy_name}"),
            Language::Spanish => format!("por {enemy_name}"),
        },
        (None, Some(room_kind)) => match language {
            Language::English => format!("by {}", room_kind_label(room_kind, language)),
            Language::Spanish => format!("por {}", room_kind_label(room_kind, language)),
        },
        (None, None) => String::from(localized_text(
            language,
            "by unknown causes",
            "por causa desconocida",
        )),
    }
}

fn combat_grid_arrangement_candidates(
    item_count: usize,
    max_rows: usize,
) -> Vec<CombatGridArrangement> {
    if item_count == 0 {
        return vec![CombatGridArrangement::empty()];
    }

    (1..=item_count.min(max_rows))
        .map(|row_count| CombatGridArrangement::balanced(item_count, row_count))
        .collect()
}

fn combat_hand_base_card_width(
    hand_arrangement: &CombatGridArrangement,
    low_hand_layout: bool,
) -> f32 {
    if hand_arrangement.row_count() > 1 {
        CARD_WIDTH * HAND_TWO_ROW_SCALE
    } else if !hand_arrangement.is_empty() && low_hand_layout {
        CARD_WIDTH * LOW_HAND_CARD_SCALE
    } else {
        CARD_WIDTH
    }
}

fn combat_hand_card_width(
    hand_arrangement: &CombatGridArrangement,
    low_hand_layout: bool,
    tile_scale: f32,
) -> f32 {
    combat_hand_base_card_width(hand_arrangement, low_hand_layout) * tile_scale
}

fn combat_layout_plan_better(
    candidate: &CombatLayoutPlan,
    hand_count: usize,
    current_best: &CombatLayoutPlan,
) -> bool {
    if candidate.score.fits != current_best.score.fits {
        return candidate.score.fits;
    }

    if hand_count > 0
        && (candidate.score.hand_card_w - current_best.score.hand_card_w).abs()
            > COMBAT_LAYOUT_SCORE_EPSILON
    {
        return candidate.score.hand_card_w > current_best.score.hand_card_w;
    }

    if (candidate.score.tile_scale - current_best.score.tile_scale).abs()
        > COMBAT_LAYOUT_SCORE_EPSILON
    {
        return candidate.score.tile_scale > current_best.score.tile_scale;
    }

    if candidate.hand.row_count() != current_best.hand.row_count() {
        return candidate.hand.row_count() < current_best.hand.row_count();
    }

    if candidate.enemies.row_count() != current_best.enemies.row_count() {
        return candidate.enemies.row_count() < current_best.enemies.row_count();
    }

    false
}

fn status_label_width(label: &str, size: f32) -> f32 {
    text_width(label, size)
}

const STATUS_ROW_MAX_COLUMNS: usize = 2;

fn status_row_line_gap(size: f32) -> f32 {
    (size * 0.4).max(4.0)
}

fn status_row_chunk_width(labels: &[(String, &'static str)], size: f32, gap: f32) -> f32 {
    labels
        .iter()
        .enumerate()
        .fold(0.0, |width, (index, (label, _))| {
            width
                + status_label_width(label, size)
                + if index + 1 < labels.len() { gap } else { 0.0 }
        })
}

fn status_row_width(labels: &[(String, &'static str)], size: f32, gap: f32) -> f32 {
    labels
        .chunks(STATUS_ROW_MAX_COLUMNS)
        .map(|row| status_row_chunk_width(row, size, gap))
        .fold(0.0, f32::max)
}

fn status_row_height(labels: &[(String, &'static str)], size: f32) -> f32 {
    if labels.is_empty() {
        return 0.0;
    }

    let rows = labels.len().div_ceil(STATUS_ROW_MAX_COLUMNS);
    size * rows as f32 + status_row_line_gap(size) * rows.saturating_sub(1) as f32
}

fn combat_layout_bounds(layout: &Layout) -> Rect {
    let mut min_x = layout.menu_button.x.min(layout.end_turn_button.x);
    let mut min_y = layout.menu_button.y.min(layout.end_turn_button.y);
    let mut max_x = (layout.menu_button.x + layout.menu_button.w)
        .max(layout.end_turn_button.x + layout.end_turn_button.w);
    let mut max_y = (layout.menu_button.y + layout.menu_button.h)
        .max(layout.end_turn_button.y + layout.end_turn_button.h);

    if let Some(rect) = layout.end_battle_button {
        min_x = min_x.min(rect.x);
        min_y = min_y.min(rect.y);
        max_x = max_x.max(rect.x + rect.w);
        max_y = max_y.max(rect.y + rect.h);
    }

    for rect in layout
        .enemy_rects
        .iter()
        .copied()
        .chain([layout.player_rect])
    {
        min_x = min_x.min(rect.x);
        min_y = min_y.min(rect.y);
        max_x = max_x.max(rect.x + rect.w);
        max_y = max_y.max(rect.y + rect.h);
    }

    for rect in &layout.hand_rects {
        min_x = min_x.min(rect.x);
        min_y = min_y.min(rect.y);
        max_x = max_x.max(rect.x + rect.w);
        max_y = max_y.max(rect.y + rect.h);
    }

    if let Some(rect) = layout.hint_rect {
        min_x = min_x.min(rect.x);
        min_y = min_y.min(rect.y);
        max_x = max_x.max(rect.x + rect.w);
        max_y = max_y.max(rect.y + rect.h);
    }

    Rect {
        x: min_x,
        y: min_y,
        w: max_x - min_x,
        h: max_y - min_y,
    }
}

fn point_in_circle(x: f32, y: f32, center_x: f32, center_y: f32, radius: f32) -> bool {
    let dx = x - center_x;
    let dy = y - center_y;
    dx * dx + dy * dy <= radius * radius
}

fn scale_rect_from_center(rect: Rect, scale: f32) -> Rect {
    let scale = scale.max(0.0);
    let next_w = rect.w * scale;
    let next_h = rect.h * scale;
    Rect {
        x: rect.x + (rect.w - next_w) * 0.5,
        y: rect.y + (rect.h - next_h) * 0.5,
        w: next_w,
        h: next_h,
    }
}

fn map_available_node_wave(time_ms: f32) -> f32 {
    (time_ms * 0.007).sin().clamp(-1.0, 1.0)
}

fn map_available_node_pulse(time_ms: f32) -> f32 {
    (map_available_node_wave(time_ms) * 0.5 + 0.5).clamp(0.0, 1.0)
}

fn room_rgb(kind: RoomKind) -> (u8, u8, u8) {
    match kind {
        RoomKind::Start => (51, 255, 102),
        RoomKind::Combat => (51, 255, 102),
        RoomKind::Elite => (216, 255, 61),
        RoomKind::Rest => (61, 245, 255),
        RoomKind::Shop => (255, 184, 82),
        RoomKind::Event => (155, 183, 255),
        RoomKind::Boss => (255, 79, 216),
    }
}

fn room_pulse_stroke(kind: RoomKind, pulse: f32) -> String {
    let alpha = 0.58 + pulse * 0.34;
    rgba(room_rgb(kind), alpha)
}

fn room_pulse_text_color(kind: RoomKind, pulse: f32) -> String {
    let alpha = 0.78 + pulse * 0.22;
    rgba(room_rgb(kind), alpha)
}

fn room_visited_stroke(kind: RoomKind) -> String {
    rgba(room_rgb(kind), 0.86)
}

fn room_visited_text_color(kind: RoomKind) -> String {
    rgba(room_rgb(kind), 0.9)
}

fn room_muted_stroke(kind: RoomKind) -> String {
    rgba(mix_rgb(room_rgb(kind), (136, 136, 136), 0.68), 0.52)
}

fn room_muted_text_color(kind: RoomKind) -> String {
    rgba(mix_rgb(room_rgb(kind), (136, 136, 136), 0.74), 0.78)
}

fn room_hover_stroke(kind: RoomKind) -> String {
    match kind {
        RoomKind::Start => String::from(COLOR_GREEN_STROKE_STRONG),
        RoomKind::Combat => String::from(COLOR_GREEN_STROKE_STRONG),
        RoomKind::Elite => rgba((216, 255, 61), 0.92),
        RoomKind::Rest => rgba((61, 245, 255), 0.92),
        RoomKind::Shop => rgba((255, 184, 82), 0.92),
        RoomKind::Event => rgba((155, 183, 255), 0.92),
        RoomKind::Boss => rgba((255, 79, 216), 0.92),
    }
}

fn interpolate_layout(
    from_layout: &Layout,
    to_layout: &Layout,
    hand_from_rects: &[Option<Rect>],
    t: f32,
) -> Layout {
    let mut layout = to_layout.clone();
    layout.start_button = lerp_rect(from_layout.start_button, to_layout.start_button, t);
    layout.restart_button = lerp_rect(from_layout.restart_button, to_layout.restart_button, t);
    layout.clear_save_button = lerp_optional_rect(
        from_layout.clear_save_button,
        to_layout.clear_save_button,
        t,
    );
    layout.menu_button = lerp_rect(from_layout.menu_button, to_layout.menu_button, t);
    layout.end_turn_button = lerp_rect(from_layout.end_turn_button, to_layout.end_turn_button, t);
    layout.end_battle_button = lerp_optional_rect(
        from_layout.end_battle_button,
        to_layout.end_battle_button,
        t,
    );
    layout.enemy_indices = to_layout.enemy_indices.clone();
    layout.enemy_rects = to_layout
        .enemy_indices
        .iter()
        .zip(to_layout.enemy_rects.iter())
        .map(|(enemy_index, to_rect)| {
            from_layout
                .enemy_rect(*enemy_index)
                .map(|from_rect| lerp_rect(from_rect, *to_rect, t))
                .unwrap_or(*to_rect)
        })
        .collect();
    layout.player_rect = lerp_rect(from_layout.player_rect, to_layout.player_rect, t);
    layout.hint_rect = lerp_optional_rect(from_layout.hint_rect, to_layout.hint_rect, t);
    layout.tile_scale = lerp_f32(from_layout.tile_scale, to_layout.tile_scale, t);
    layout.tile_insets = lerp_tile_insets(from_layout.tile_insets, to_layout.tile_insets, t);
    layout.hand_rects = to_layout
        .hand_rects
        .iter()
        .enumerate()
        .map(|(index, to_rect)| {
            hand_from_rects
                .get(index)
                .copied()
                .flatten()
                .map(|from_rect| lerp_rect(from_rect, *to_rect, t))
                .unwrap_or(*to_rect)
        })
        .collect();
    layout
}

fn interpolated_transition_layout(transition: &LayoutTransition) -> Layout {
    if transition.total_ms <= 0.0 {
        return transition.to_layout.clone();
    }

    let progress = 1.0 - (transition.ttl_ms / transition.total_ms).clamp(0.0, 1.0);
    interpolate_layout(
        &transition.from_layout,
        &transition.to_layout,
        &transition.hand_from_rects,
        ease_out_cubic(progress),
    )
}

fn lerp_optional_rect(from: Option<Rect>, to: Option<Rect>, t: f32) -> Option<Rect> {
    match (from, to) {
        (Some(from_rect), Some(to_rect)) => Some(lerp_rect(from_rect, to_rect, t)),
        (None, Some(to_rect)) => Some(to_rect),
        (Some(from_rect), None) => Some(from_rect),
        (None, None) => None,
    }
}

fn optional_rects_match(left: Option<Rect>, right: Option<Rect>) -> bool {
    match (left, right) {
        (Some(left), Some(right)) => rects_match(left, right),
        (None, None) => true,
        _ => false,
    }
}

fn rects_match(left: Rect, right: Rect) -> bool {
    (left.x - right.x).abs() <= 0.01
        && (left.y - right.y).abs() <= 0.01
        && (left.w - right.w).abs() <= 0.01
        && (left.h - right.h).abs() <= 0.01
}

fn rect_vecs_match(left: &[Rect], right: &[Rect]) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right.iter())
            .all(|(left, right)| rects_match(*left, *right))
}

fn combat_layouts_match(left: &Layout, right: &Layout) -> bool {
    rects_match(left.start_button, right.start_button)
        && rects_match(left.restart_button, right.restart_button)
        && optional_rects_match(left.clear_save_button, right.clear_save_button)
        && rects_match(left.menu_button, right.menu_button)
        && rects_match(left.end_turn_button, right.end_turn_button)
        && optional_rects_match(left.end_battle_button, right.end_battle_button)
        && left.enemy_indices == right.enemy_indices
        && rect_vecs_match(&left.enemy_rects, &right.enemy_rects)
        && rects_match(left.player_rect, right.player_rect)
        && rect_vecs_match(&left.hand_rects, &right.hand_rects)
        && optional_rects_match(left.hint_rect, right.hint_rect)
        && (left.tile_scale - right.tile_scale).abs() <= 0.01
        && (left.tile_insets.pad_x - right.tile_insets.pad_x).abs() <= 0.01
        && (left.tile_insets.top_pad - right.tile_insets.top_pad).abs() <= 0.01
        && (left.tile_insets.bottom_pad - right.tile_insets.bottom_pad).abs() <= 0.01
        && left.low_hand_layout == right.low_hand_layout
}

fn lerp_tile_insets(from: TileInsets, to: TileInsets, t: f32) -> TileInsets {
    TileInsets {
        pad_x: lerp_f32(from.pad_x, to.pad_x, t),
        top_pad: lerp_f32(from.top_pad, to.top_pad, t),
        bottom_pad: lerp_f32(from.bottom_pad, to.bottom_pad, t),
    }
}

fn lerp_rect(from: Rect, to: Rect, t: f32) -> Rect {
    Rect {
        x: lerp_f32(from.x, to.x, t),
        y: lerp_f32(from.y, to.y, t),
        w: lerp_f32(from.w, to.w, t),
        h: lerp_f32(from.h, to.h, t),
    }
}

fn lerp_f32(from: f32, to: f32, t: f32) -> f32 {
    from + (to - from) * t.clamp(0.0, 1.0)
}

fn scramble_seed(value: u64) -> u64 {
    let mut x = value ^ 0x9E37_79B9_7F4A_7C15;
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

fn limit_run_seed(seed: u64) -> u64 {
    seed & RUN_SEED_MASK
}

fn display_seed(seed: u64) -> String {
    format!("{:08X}", limit_run_seed(seed))
}

fn debug_map_label(dungeon: &DungeonRun, language: Language) -> String {
    match language {
        Language::English => format!(
            "DEBUG L{}/{} seed {}",
            dungeon.current_level(),
            dungeon.total_levels(),
            display_seed(dungeon.seed)
        ),
        Language::Spanish => format!(
            "DEBUG N{}/{} semilla {}",
            dungeon.current_level(),
            dungeon.total_levels(),
            display_seed(dungeon.seed)
        ),
    }
}

fn ease_out_cubic(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    1.0 - (1.0 - t).powi(3)
}

fn text_width(text: &str, size: f32) -> f32 {
    size * 0.62 * text.chars().count() as f32
}

fn fit_text_size(text: &str, desired_size: f32, max_width: f32) -> f32 {
    let width = text_width(text, desired_size);
    if width <= max_width || width <= 0.0 {
        desired_size
    } else {
        desired_size * (max_width / width)
    }
}

fn button_size(label: &str, font_size: f32, pad_x: f32, pad_y: f32) -> (f32, f32) {
    (
        text_width(label, font_size) + pad_x * 2.0,
        font_size + pad_y * 2.0,
    )
}

fn fit_modal_width(desired_width: f32, logical_width: f32, min_width: f32) -> f32 {
    let max_width = (logical_width - 32.0).max(180.0);
    desired_width.clamp(min_width.min(max_width), max_width)
}

fn fit_overlay_button_metrics(labels: &[&str], max_row_width: f32) -> OverlayButtonMetrics {
    let (base_pad_x, base_pad_y) = boot_button_tile_padding();
    let max_row_width = max_row_width.max(1.0);

    for step in 0..=24 {
        let t = step as f32 / 24.0;
        let font_size = lerp_f32(START_BUTTON_FONT_SIZE, OVERLAY_BUTTON_MIN_FONT_SIZE, t);
        let pad_x = lerp_f32(base_pad_x, OVERLAY_BUTTON_MIN_PAD_X, t);
        let pad_y = lerp_f32(base_pad_y, OVERLAY_BUTTON_MIN_PAD_Y, t);
        let item_gap = lerp_f32(OVERLAY_BUTTON_ROW_GAP, OVERLAY_BUTTON_MIN_ROW_GAP, t);
        let widths: Vec<f32> = labels
            .iter()
            .map(|label| button_size(label, font_size, pad_x, pad_y).0)
            .collect();
        let height = labels
            .iter()
            .map(|label| button_size(label, font_size, pad_x, pad_y).1)
            .fold(0.0, f32::max);
        let block_w = widths.iter().sum::<f32>() + item_gap * widths.len().saturating_sub(1) as f32;
        if block_w <= max_row_width {
            return OverlayButtonMetrics {
                flow: OverlayButtonFlow::Row,
                font_size,
                item_gap,
                widths,
                height,
                block_w,
                block_h: height,
            };
        }
    }

    for step in 0..=24 {
        let t = step as f32 / 24.0;
        let font_size = lerp_f32(START_BUTTON_FONT_SIZE, OVERLAY_BUTTON_MIN_FONT_SIZE, t);
        let pad_x = lerp_f32(base_pad_x, OVERLAY_BUTTON_MIN_PAD_X, t);
        let pad_y = lerp_f32(base_pad_y, OVERLAY_BUTTON_MIN_PAD_Y, t);
        let widths: Vec<f32> = labels
            .iter()
            .map(|label| button_size(label, font_size, pad_x, pad_y).0)
            .collect();
        let height = labels
            .iter()
            .map(|label| button_size(label, font_size, pad_x, pad_y).1)
            .fold(0.0, f32::max);
        let block_w = widths.iter().copied().fold(0.0, f32::max);
        if block_w <= max_row_width {
            return OverlayButtonMetrics {
                flow: OverlayButtonFlow::Stack,
                font_size,
                item_gap: OVERLAY_BUTTON_STACK_GAP,
                widths,
                height,
                block_w,
                block_h: height * labels.len() as f32
                    + OVERLAY_BUTTON_STACK_GAP * labels.len().saturating_sub(1) as f32,
            };
        }
    }

    let font_size = OVERLAY_BUTTON_MIN_FONT_SIZE;
    let pad_x = OVERLAY_BUTTON_MIN_PAD_X;
    let pad_y = OVERLAY_BUTTON_MIN_PAD_Y;
    let widths: Vec<f32> = labels
        .iter()
        .map(|label| button_size(label, font_size, pad_x, pad_y).0)
        .collect();
    let height = labels
        .iter()
        .map(|label| button_size(label, font_size, pad_x, pad_y).1)
        .fold(0.0, f32::max);

    OverlayButtonMetrics {
        flow: OverlayButtonFlow::Stack,
        font_size,
        item_gap: OVERLAY_BUTTON_STACK_GAP,
        widths: widths.clone(),
        height,
        block_w: widths.iter().copied().fold(0.0, f32::max),
        block_h: height * labels.len() as f32
            + OVERLAY_BUTTON_STACK_GAP * labels.len().saturating_sub(1) as f32,
    }
}

fn place_overlay_buttons(
    metrics: &OverlayButtonMetrics,
    modal_rect: Rect,
    bottom_pad: f32,
) -> Vec<FittedPrimaryButton> {
    match metrics.flow {
        OverlayButtonFlow::Row => {
            let mut x = modal_rect.x + (modal_rect.w - metrics.block_w) * 0.5;
            let y = modal_rect.y + modal_rect.h - bottom_pad - metrics.height;
            metrics
                .widths
                .iter()
                .map(|width| {
                    let button = FittedPrimaryButton {
                        rect: Rect {
                            x,
                            y,
                            w: *width,
                            h: metrics.height,
                        },
                        font_size: metrics.font_size,
                    };
                    x += *width + metrics.item_gap;
                    button
                })
                .collect()
        }
        OverlayButtonFlow::Stack => {
            let mut y = modal_rect.y + modal_rect.h - bottom_pad - metrics.block_h;
            metrics
                .widths
                .iter()
                .map(|width| {
                    let button = FittedPrimaryButton {
                        rect: Rect {
                            x: modal_rect.x + (modal_rect.w - *width) * 0.5,
                            y,
                            w: *width,
                            h: metrics.height,
                        },
                        font_size: metrics.font_size,
                    };
                    y += metrics.height + metrics.item_gap;
                    button
                })
                .collect()
        }
    }
}

fn centered_button_rect(
    label: &str,
    font_size: f32,
    pad_x: f32,
    pad_y: f32,
    center_x: f32,
    top_y: f32,
) -> Rect {
    let (w, h) = button_size(label, font_size, pad_x, pad_y);
    Rect {
        x: center_x - w * 0.5,
        y: top_y,
        w,
        h,
    }
}

fn button_text_baseline(rect: Rect, font_size: f32) -> f32 {
    rect.y + rect.h * 0.5 + font_size * 0.32
}

fn escape_json_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn final_victory_share_payload(summary: &FinalVictorySummary, language: Language) -> String {
    let seed = display_seed(summary.seed);
    let share_text = match language {
        Language::English => format!(
            "I cleared all {} sectors in {}. {} max HP. {} card deck. Seed {}.",
            summary.total_levels, GAME_TITLE, summary.player_max_hp, summary.deck_count, seed
        ),
        Language::Spanish => format!(
            "Completé los {} sectores en {}. {} HP máximo. Mazo de {} cartas. Semilla {}.",
            summary.total_levels, GAME_TITLE, summary.player_max_hp, summary.deck_count, seed
        ),
    };
    format!(
        r#"{{"kind":"final_victory_card","title":"{title}","max_hp":{max_hp},"deck_size":{deck_size},"seed":"{seed}","version":"{version}","share_text":"{share_text}"}}"#,
        title = GAME_TITLE,
        max_hp = summary.player_max_hp,
        deck_size = summary.deck_count,
        seed = seed,
        version = GAME_VERSION,
        share_text = escape_json_string(&share_text),
    )
}

fn result_button_layout(
    logical_width: f32,
    logical_height: f32,
    include_share: bool,
    language: Language,
) -> ResultButtons {
    let (pad_x, pad_y) = boot_button_tile_padding();
    let menu_label = localized_text(language, RESULT_BUTTON_LABEL, "Menú principal");
    let (_, menu_h) = button_size(menu_label, START_BUTTON_FONT_SIZE, pad_x, pad_y);
    let bottom_margin = pad_y;
    let menu_button = centered_button_rect(
        menu_label,
        START_BUTTON_FONT_SIZE,
        pad_x,
        pad_y,
        logical_width * 0.5,
        (logical_height - menu_h - bottom_margin).max(0.0),
    );
    if !include_share {
        return ResultButtons {
            share_button: None,
            menu_button,
        };
    }

    let (share_w, share_h) = button_size(
        localized_text(language, "Share", "Compartir"),
        START_BUTTON_FONT_SIZE,
        pad_x,
        pad_y,
    );
    let gap = 16.0;
    ResultButtons {
        share_button: Some(Rect {
            x: logical_width * 0.5 - share_w * 0.5,
            y: (menu_button.y - gap - share_h).max(0.0),
            w: share_w,
            h: share_h,
        }),
        menu_button,
    }
}

fn format_visible_game_version_label(
    channel: &str,
    version: &str,
    build_timestamp_utc: Option<&str>,
    git_sha_short: Option<&str>,
) -> String {
    if channel != "preview" {
        return format!("v{version}");
    }

    match (build_timestamp_utc, git_sha_short) {
        (Some(timestamp), Some(sha)) => format!("preview {timestamp} {sha}"),
        (Some(timestamp), None) => format!("preview {timestamp}"),
        (None, Some(sha)) => format!("preview {sha}"),
        (None, None) => "preview".to_string(),
    }
}

fn app_channel() -> &'static str {
    match BUILD_APP_CHANNEL {
        Some("preview") => "preview",
        _ => "stable",
    }
}

fn visible_game_version_label() -> String {
    format_visible_game_version_label(
        app_channel(),
        GAME_VERSION,
        APP_BUILD_TIMESTAMP_UTC,
        APP_GIT_SHA_SHORT,
    )
}

fn boot_version_font_size(logical_width: f32) -> f32 {
    let version_line = visible_game_version_label();
    fit_text_size(&version_line, 14.0, (logical_width - 48.0).max(120.0)).max(11.0)
}

fn boot_version_baseline_y(logical_height: f32) -> f32 {
    let (_, pad_y) = boot_button_tile_padding();
    (logical_height - pad_y).max(0.0)
}

fn primary_button_pulse(time_ms: f32) -> f32 {
    0.55 + 0.45 * (time_ms * 0.0025).sin().abs()
}

fn render_primary_button(
    scene: &mut SceneBuilder,
    rect: Rect,
    hovered: bool,
    label: &str,
    time_ms: f32,
) {
    render_primary_button_sized(scene, rect, START_BUTTON_FONT_SIZE, hovered, label, time_ms);
}

fn render_primary_button_sized(
    scene: &mut SceneBuilder,
    rect: Rect,
    font_size: f32,
    hovered: bool,
    label: &str,
    time_ms: f32,
) {
    render_ui_tile(scene, rect, BUTTON_RADIUS, COLOR_GREEN_STROKE_START);
    scene.text(
        rect.x + rect.w * 0.5,
        button_text_baseline(rect, font_size),
        font_size,
        "center",
        &rgba(
            (51, 255, 102),
            if hovered {
                1.0
            } else {
                primary_button_pulse(time_ms)
            },
        ),
        "label",
        label,
    );
}

fn combat_top_button_font_size(low_hand_layout: bool, tile_scale: f32) -> f32 {
    let base = if low_hand_layout {
        LOW_HAND_TOP_BUTTON_FONT_SIZE
    } else {
        TOP_BUTTON_FONT_SIZE
    };
    base * tile_scale
}

fn combat_action_button_font_size(low_hand_layout: bool, tile_scale: f32) -> f32 {
    combat_top_button_font_size(low_hand_layout, tile_scale) * COMBAT_ACTION_UI_SCALE
}

fn combat_action_button_padding(tile_insets: TileInsets) -> (f32, f32) {
    (
        tile_insets.pad_x * COMBAT_ACTION_UI_SCALE,
        tile_insets.top_pad * COMBAT_ACTION_UI_SCALE,
    )
}

fn boot_button_tile_padding() -> (f32, f32) {
    let insets = tile_insets_for_card_width(CARD_WIDTH);
    (insets.pad_x, insets.top_pad)
}

fn standard_overlay_padding() -> (f32, f32) {
    boot_button_tile_padding()
}

fn boot_hero_layout(logical_width: f32, logical_height: f32) -> BootHeroLayout {
    let title_size = fit_text_size("Mazocarta", 88.0, (logical_width - 48.0).max(120.0)).max(30.0);
    let logo_size = (logical_width.min(logical_height) * 0.18).clamp(84.0, 156.0);
    let (start_pad_x, start_pad_y) = boot_button_tile_padding();
    let (_, start_h) = button_size("Start", START_BUTTON_FONT_SIZE, start_pad_x, start_pad_y);
    let gap = (title_size * 0.42).clamp(22.0, 38.0);
    let total_h = logo_size + gap + title_size + gap + start_h;
    let stack_top = ((logical_height - total_h) * 0.5 - BOOT_HERO_SHIFT_UP).max(24.0);
    let title_top = stack_top + logo_size + gap;

    BootHeroLayout {
        logo_rect: Rect {
            x: logical_width * 0.5 - logo_size * 0.5,
            y: stack_top,
            w: logo_size,
            h: logo_size,
        },
        title_size,
        title_baseline_y: title_top + title_size * 0.82,
        start_button_y: title_top + title_size + gap,
    }
}

fn screen_transition_style(from_screen: AppScreen, to_screen: AppScreen) -> ScreenTransitionStyle {
    if matches!(from_screen, AppScreen::OpeningIntro)
        || matches!(to_screen, AppScreen::OpeningIntro)
    {
        ScreenTransitionStyle::Fade
    } else {
        ScreenTransitionStyle::Motion
    }
}

fn hand_hint_metrics(tile_scale: f32) -> (f32, f32, f32) {
    (
        16.0 * tile_scale * COMBAT_ACTION_UI_SCALE,
        14.0 * tile_scale * COMBAT_ACTION_UI_SCALE,
        8.0 * tile_scale * COMBAT_ACTION_UI_SCALE,
    )
}

fn combat_hint_tile(app: &App, hand_count: usize) -> (String, &'static str, &'static str) {
    if app.combat_feedback.playback_kind == Some(CombatPlaybackKind::EnemyTurn) {
        return (
            String::from(app.tr("Resolving enemy turn...", "Resolviendo turno enemigo...")),
            TERM_CYAN_SOFT,
            COLOR_CYAN_STROKE_IDLE,
        );
    }
    if app.combat_feedback.playback_kind == Some(CombatPlaybackKind::PlayerAction) {
        return (
            String::from(app.tr("Resolving action...", "Resolviendo accion...")),
            TERM_LIME_SOFT,
            COLOR_LIME_STROKE_TARGET,
        );
    }
    if app.combat_feedback.pending_outcome.is_some() || app.combat_input_locked() {
        return (
            String::from(app.tr("Resolving encounter...", "Resolviendo encuentro...")),
            TERM_GREEN_TEXT,
            COLOR_GREEN_STROKE_IDLE,
        );
    }

    match app.ui.selected_card {
        Some(index) if index < hand_count => {
            if app.combat.can_play_card(index) {
                let energy_cost = app
                    .combat
                    .hand_card(index)
                    .map(|card| app.localized_card_def(card).cost)
                    .unwrap_or(0);
                if app.combat.card_requires_enemy(index) {
                    (
                        match app.language {
                            Language::English => format!("Tap enemy ({} energy)", energy_cost),
                            Language::Spanish => format!("Toca enemigo ({} energía)", energy_cost),
                        },
                        TERM_LIME_SOFT,
                        COLOR_LIME_STROKE_TARGET,
                    )
                } else if app.combat.card_targets_all_enemies(index) {
                    (
                        match app.language {
                            Language::English => {
                                format!("Tap card again ({} energy)", energy_cost)
                            }
                            Language::Spanish => {
                                format!("Toca la carta otra vez ({} energía)", energy_cost)
                            }
                        },
                        TERM_GREEN_SOFT,
                        COLOR_GREEN_STROKE_STRONG,
                    )
                } else {
                    (
                        match app.language {
                            Language::English => format!("Tap player ({} energy)", energy_cost),
                            Language::Spanish => format!("Toca jugador ({} energía)", energy_cost),
                        },
                        TERM_CYAN_SOFT,
                        COLOR_CYAN_STROKE_TARGET,
                    )
                }
            } else {
                (
                    String::from(app.tr("Insufficient energy", "Energía insuficiente")),
                    "#d0d0d0",
                    COLOR_GRAY_STROKE_HINT,
                )
            }
        }
        _ => (
            String::from(app.tr("Tap card or end turn", "Toca una carta o termina el turno")),
            TERM_GREEN_TEXT,
            COLOR_GREEN_STROKE_CARD,
        ),
    }
}

fn enemy_intent_lines(label: &str, summary: &str, max_chars: usize) -> EnemyIntentLines {
    let max_chars = max_chars.max(1);
    let summary_words: Vec<&str> = summary.split_whitespace().collect();
    let first_line_summary_chars = max_chars.saturating_sub(label.len() + 1);
    let mut first_line_words = Vec::new();
    let mut used_chars = 0usize;

    if first_line_summary_chars > 0 {
        for word in &summary_words {
            let next_len = if first_line_words.is_empty() {
                word.len()
            } else {
                used_chars + 1 + word.len()
            };
            if next_len > first_line_summary_chars {
                break;
            }
            first_line_words.push(*word);
            used_chars = next_len;
        }
    }

    let first_line_summary = first_line_words.join(" ");
    let first_line_label = if first_line_summary.is_empty() {
        String::from(label)
    } else {
        format!("{label} ")
    };
    let continuation_lines = if first_line_words.len() == summary_words.len() {
        Vec::new()
    } else {
        wrap_text(
            &summary_words[first_line_words.len()..].join(" "),
            max_chars,
        )
    };

    EnemyIntentLines {
        first_line_label,
        first_line_summary,
        continuation_lines,
    }
}

fn enemy_panel_metrics(
    app: &App,
    enemy_index: usize,
    low_hand_layout: bool,
    tile_scale: f32,
    tile_insets: TileInsets,
) -> EnemyPanelMetrics {
    let scale = if low_hand_layout { 1.16 } else { 1.0 } * tile_scale;
    let info_body_size = 15.0 * scale;
    let info_body_line_gap = 5.0 * scale;
    let stats_size = 18.0 * scale;
    let status_size = 14.0 * scale;
    let info_body_breathing_room = text_bottom_breathing_room(info_body_size);
    let status_gap = 18.0 * scale;
    let content_pad_x = tile_insets.pad_x;
    let top_pad = tile_insets.top_pad;
    let line_gap = 10.0 * scale;
    let bottom_pad = tile_insets.bottom_pad;
    let enemy = app.combat.enemy(enemy_index);
    let displayed = app.displayed_actor_stats(Actor::Enemy(enemy_index));
    let next_label = app.tr(NEXT_SIGNAL_LABEL, "Siguiente");
    let summary = app.enemy_signal_summary(enemy_index);
    let stats_line_width = combat_actor_stats_line_width(
        displayed.hp,
        enemy.map(|enemy| enemy.fighter.max_hp).unwrap_or(0),
        displayed.block,
        stats_size,
        combat_inline_group_gap(stats_size),
    );
    let status_labels = enemy
        .map(|enemy| status_labels(enemy.fighter.statuses, enemy.on_hit_bleed, app.language))
        .unwrap_or_default();
    let status_row_height = status_row_height(&status_labels, status_size);
    let status_width = if status_labels.is_empty() {
        0.0
    } else {
        status_row_width(&status_labels, status_size, status_gap)
    };
    let top_row_sprite_width = enemy
        .map(|enemy| {
            let sprite = enemy_sprite_def(enemy.profile);
            enemy_inline_sprite_width(stats_size, sprite.width, sprite.height)
        })
        .unwrap_or(combat_inline_icon_height(stats_size));
    let top_row_width =
        top_row_sprite_width + combat_inline_group_gap(stats_size) + stats_line_width;
    let base_inner_width = top_row_width.max(status_width);
    let mut info_body_chars =
        ((base_inner_width / (info_body_size * 0.62)).floor() as usize).max(18);
    let mut intent_lines = enemy_intent_lines(next_label, summary, info_body_chars);
    let mut signal_width = intent_lines.max_width(info_body_size);
    while info_body_chars > 1 && signal_width > base_inner_width + 0.01 {
        info_body_chars -= 1;
        intent_lines = enemy_intent_lines(next_label, summary, info_body_chars);
        signal_width = intent_lines.max_width(info_body_size);
    }
    let inner_width = base_inner_width.max(signal_width);
    let width = inner_width + content_pad_x * 2.0;
    let info_body_lines = intent_lines.line_count();
    let height = top_pad
        + stats_size
        + line_gap
        + status_row_height
        + if status_row_height > 0.0 {
            line_gap
        } else {
            0.0
        }
        + info_body_size * info_body_lines as f32
        + info_body_line_gap * info_body_lines.saturating_sub(1) as f32
        + info_body_breathing_room
        + bottom_pad;

    EnemyPanelMetrics {
        info_body_size,
        info_body_line_gap,
        info_body_chars,
        stats_size,
        status_size,
        status_row_height,
        status_gap,
        top_pad,
        line_gap,
        width,
        height,
    }
}

fn enemy_inline_sprite_width(font_size: f32, sprite_width: u8, sprite_height: u8) -> f32 {
    let sprite_w = sprite_width.max(1) as f32;
    let sprite_h = sprite_height.max(1) as f32;

    combat_inline_icon_height(font_size) * sprite_w / sprite_h
}

fn enemy_inline_sprite_rect(
    x: f32,
    baseline_y: f32,
    font_size: f32,
    sprite_width: u8,
    sprite_height: u8,
) -> Rect {
    let draw_h = combat_inline_icon_height(font_size);
    let draw_w = enemy_inline_sprite_width(font_size, sprite_width, sprite_height);

    Rect {
        x,
        y: baseline_y - draw_h,
        w: draw_w,
        h: draw_h,
    }
}

#[derive(Clone, Copy)]
struct EnemySpritePalette {
    base: &'static str,
    detail_a: &'static str,
    detail_b: &'static str,
    detail_c: &'static str,
    detail_d: &'static str,
    detail_e: &'static str,
    dim: &'static str,
}

const ENEMY_LEVEL_ONE_SPRITE_PALETTE: EnemySpritePalette = EnemySpritePalette {
    base: TERM_GREEN_SOFT,
    detail_a: "#efff6f",
    detail_b: "#39e8ff",
    detail_c: "#7fb6ff",
    detail_d: "#1fba63",
    detail_e: "#ffe39a",
    dim: TERM_GREEN_DIM,
};

const ENEMY_LEVEL_TWO_SPRITE_PALETTE: EnemySpritePalette = EnemySpritePalette {
    base: "#c7a7ff",
    detail_a: "#ff9df3",
    detail_b: "#7f89ff",
    detail_c: "#79e7ff",
    detail_d: "#b65cff",
    detail_e: "#ffe9b8",
    dim: "#7f719b",
};

const ENEMY_LEVEL_THREE_SPRITE_PALETTE: EnemySpritePalette = EnemySpritePalette {
    base: TERM_ORANGE,
    detail_a: "#ffe07a",
    detail_b: "#ff6438",
    detail_c: "#ff4f8a",
    detail_d: "#fff27a",
    detail_e: "#9fe7ff",
    dim: "#9a7657",
};

fn enemy_sprite_palette(profile: EnemyProfileId) -> EnemySpritePalette {
    match enemy_profile_level(profile) {
        1 => ENEMY_LEVEL_ONE_SPRITE_PALETTE,
        2 => ENEMY_LEVEL_TWO_SPRITE_PALETTE,
        _ => ENEMY_LEVEL_THREE_SPRITE_PALETTE,
    }
}

fn enemy_panel_icon_alpha(profile: EnemyProfileId, is_alive: bool) -> f32 {
    match (enemy_profile_level(profile), is_alive) {
        (1, true) => ENEMY_PANEL_ICON_ALPHA - 0.04,
        (2, true) => ENEMY_PANEL_ICON_ALPHA,
        (_, true) => ENEMY_PANEL_ICON_ALPHA + 0.05,
        (1, false) => ENEMY_PANEL_ICON_DISABLED_ALPHA - 0.08,
        (2, false) => ENEMY_PANEL_ICON_DISABLED_ALPHA - 0.04,
        (_, false) => ENEMY_PANEL_ICON_DISABLED_ALPHA,
    }
}

fn enemy_sprite_layer_color(
    profile: EnemyProfileId,
    tone: EnemySpriteLayerTone,
    is_alive: bool,
) -> &'static str {
    let palette = enemy_sprite_palette(profile);
    if !is_alive {
        return palette.dim;
    }

    match tone {
        EnemySpriteLayerTone::Base => palette.base,
        EnemySpriteLayerTone::DetailA => palette.detail_a,
        EnemySpriteLayerTone::DetailB => palette.detail_b,
        EnemySpriteLayerTone::DetailC => palette.detail_c,
        EnemySpriteLayerTone::DetailD => palette.detail_d,
        EnemySpriteLayerTone::DetailE => palette.detail_e,
    }
}

fn player_panel_metrics(
    app: &App,
    low_hand_layout: bool,
    tile_scale: f32,
    tile_insets: TileInsets,
) -> PlayerPanelMetrics {
    let scale = if low_hand_layout { 1.14 } else { 1.0 } * tile_scale;
    let label_size = 13.5 * scale;
    let stats_size = 20.0 * scale;
    let meta_size = 18.0 * scale;
    let status_size = 14.0 * scale;
    let meta_breathing_room = text_bottom_breathing_room(meta_size);
    let status_gap = 18.0 * scale;
    let content_pad_x = tile_insets.pad_x;
    let top_pad = tile_insets.top_pad;
    let line_gap = 10.0 * scale;
    let bottom_pad = tile_insets.bottom_pad;
    let displayed = app.displayed_actor_stats(Actor::Player);
    let stats_line_width = combat_actor_stats_line_width(
        displayed.hp,
        app.combat.player.fighter.max_hp,
        displayed.block,
        stats_size,
        combat_inline_group_gap(stats_size),
    );
    let meta_line_width = combat_meta_line_width(app, meta_size);
    let status_labels = status_labels(app.combat.player.fighter.statuses, 0, app.language);
    let status_row_height = status_row_height(&status_labels, status_size);
    let status_width = if status_labels.is_empty() {
        0.0
    } else {
        status_row_width(&status_labels, status_size, status_gap)
    };
    let width = stats_line_width
        .max(meta_line_width)
        .max(text_width(app.tr(PLAYER_NAME, "Jugador"), label_size))
        .max(status_width)
        + content_pad_x * 2.0;
    let height = top_pad
        + label_size
        + line_gap
        + stats_size
        + line_gap
        + status_row_height
        + if status_row_height > 0.0 {
            line_gap
        } else {
            0.0
        }
        + meta_size
        + meta_breathing_room
        + bottom_pad;

    PlayerPanelMetrics {
        label_size,
        stats_size,
        meta_size,
        status_size,
        status_row_height,
        status_gap,
        top_pad,
        line_gap,
        width,
        height,
    }
}

fn combat_inline_icon_height(font_size: f32) -> f32 {
    (font_size - text_bottom_breathing_room(font_size)) * COMBAT_INLINE_ICON_HEIGHT_RATIO
}

fn combat_inline_icon_width(font_size: f32) -> f32 {
    combat_inline_icon_height(font_size) * COMBAT_INLINE_ICON_ASPECT_RATIO
}

fn combat_inline_group_gap(font_size: f32) -> f32 {
    text_width(PANEL_TEXT_GAP, font_size)
}

fn combat_inline_icon_text_gap(font_size: f32) -> f32 {
    (font_size * COMBAT_INLINE_ICON_TEXT_GAP_RATIO).max(1.0)
}

fn combat_inline_icon_rect(x: f32, baseline_y: f32, font_size: f32) -> Rect {
    let height = combat_inline_icon_height(font_size);
    let width = combat_inline_icon_width(font_size);

    Rect {
        x,
        y: baseline_y - height,
        w: width,
        h: height,
    }
}

fn render_combat_inline_icon(
    scene: &mut SceneBuilder,
    cursor: &mut f32,
    baseline_y: f32,
    font_size: f32,
    src: &str,
    color: &str,
) {
    let rect = combat_inline_icon_rect(*cursor, baseline_y, font_size);
    scene.tinted_image(rect, src, color, 1.0);
    *cursor += rect.w + combat_inline_icon_text_gap(font_size);
}

fn combat_actor_stats_line_width(
    hp: i32,
    max_hp: i32,
    block: i32,
    font_size: f32,
    group_gap: f32,
) -> f32 {
    combat_inline_icon_width(font_size)
        + combat_inline_icon_text_gap(font_size)
        + text_width(&hp.to_string(), font_size)
        + text_width(&format!("/{max_hp}"), font_size)
        + group_gap
        + combat_inline_icon_width(font_size)
        + combat_inline_icon_text_gap(font_size)
        + text_width(&block.to_string(), font_size)
}

fn combat_meta_line_width(app: &App, font_size: f32) -> f32 {
    let displayed_meta = app.displayed_player_meta();
    let energy_text = format!("{}/{}", displayed_meta.energy, app.combat.player.max_energy);
    let draw_text = displayed_meta.draw_pile.to_string();
    let discard_text = displayed_meta.discard_pile.to_string();

    combat_inline_icon_width(font_size)
        + combat_inline_icon_text_gap(font_size)
        + text_width(&energy_text, font_size)
        + combat_inline_group_gap(font_size)
        + combat_inline_icon_width(font_size)
        + combat_inline_icon_text_gap(font_size)
        + text_width(&draw_text, font_size)
        + text_width("→", font_size)
        + text_width(&discard_text, font_size)
}

fn tile_insets_for_card_width(card_w: f32) -> TileInsets {
    let metrics = card_box_metrics(card_w);
    TileInsets {
        pad_x: metrics.pad_x,
        top_pad: metrics.top_pad,
        bottom_pad: metrics.bottom_pad,
    }
}

fn text_bottom_breathing_room(font_size: f32) -> f32 {
    font_size * 0.28
}

fn card_box_metrics(card_w: f32) -> CardBoxMetrics {
    let scale = (card_w / CARD_WIDTH).clamp(0.0, 2.6);
    let pad_x = (12.0 * scale).clamp(6.0, 21.0);
    let vertical_pad = (8.0 * scale).clamp(5.0, 16.0);
    let top_pad = vertical_pad;
    let bottom_pad = vertical_pad;
    let title_size = (15.0 * scale).clamp(10.0, 28.0);
    let cost_size = (26.0 * scale).clamp(16.0, 44.0);
    let body_size = (16.0 * scale).clamp(10.0, 28.0);
    let title_gap = (2.0 * scale).clamp(1.0, 4.0);
    let title_body_gap = (6.0 * scale).clamp(3.0, 10.0);
    let body_gap = (4.0 * scale).clamp(2.0, 8.0);
    let cost_lane = (cost_size * 1.15).clamp(22.0, 60.0);
    let title_chars = ((card_w - pad_x * 2.0 - cost_lane - title_gap) / (title_size * 0.56))
        .floor()
        .max(8.0) as usize;
    let body_wrap_reserve = (body_size * 0.9).clamp(6.0, 14.0);
    // Keep a small wrap margin so browser text rendering does not crowd the card edge.
    let body_max_width = (card_w - pad_x * 2.0 - body_wrap_reserve).max(1.0);
    let body_chars = ((card_w - pad_x * 2.0 - body_wrap_reserve) / (body_size * 0.56))
        .floor()
        .max(8.0) as usize;
    let min_height =
        (top_pad + title_size.max(cost_size) + title_body_gap + body_size + bottom_pad)
            .max((56.0 * scale).clamp(44.0, CARD_HEIGHT * 1.2));

    CardBoxMetrics {
        pad_x,
        top_pad,
        bottom_pad,
        title_size,
        cost_size,
        body_size,
        body_max_width,
        title_gap,
        title_body_gap,
        body_gap,
        title_chars,
        body_chars,
        min_height,
    }
}

fn card_content_height(def: CardDef, card_w: f32) -> f32 {
    card_content_height_with_description(def, def.description, card_w)
}

fn card_content_height_with_description(def: CardDef, description: &str, card_w: f32) -> f32 {
    let metrics = card_box_metrics(card_w);
    let title_lines = wrap_text(def.name, metrics.title_chars);
    let body_lines = wrap_text_by_width(description, metrics.body_size, metrics.body_max_width);
    let title_height = if title_lines.is_empty() {
        0.0
    } else {
        metrics.title_size * title_lines.len() as f32
            + metrics.title_gap * title_lines.len().saturating_sub(1) as f32
    };
    let body_height = if body_lines.is_empty() {
        0.0
    } else {
        metrics.body_size * body_lines.len() as f32
            + metrics.body_gap * body_lines.len().saturating_sub(1) as f32
    };
    let body_breathing_room = if body_lines.is_empty() {
        0.0
    } else {
        text_bottom_breathing_room(metrics.body_size) * 1.0
    };
    let header_height = title_height.max(metrics.cost_size);
    let content_height = metrics.top_pad
        + header_height
        + metrics.title_body_gap
        + body_height
        + body_breathing_room
        + metrics.bottom_pad;

    content_height.max(metrics.min_height)
}

fn module_box_metrics(card_w: f32) -> CardBoxMetrics {
    let mut metrics = card_box_metrics(card_w);
    let scale = (card_w / CARD_WIDTH).clamp(0.0, 2.6);
    metrics.pad_x = (metrics.pad_x * 0.85).max(6.0);
    metrics.top_pad = (metrics.top_pad * 0.82).max(4.0);
    metrics.bottom_pad = (metrics.bottom_pad * 1.08).max(7.0);
    metrics.title_size = (metrics.title_size * 0.86).max(10.0);
    metrics.body_size = (metrics.body_size * 0.84).max(10.0);
    metrics.title_gap = (metrics.title_gap * 0.8).max(1.0);
    metrics.title_body_gap = (metrics.title_body_gap * 0.75).max(2.0);
    metrics.body_gap = (metrics.body_gap * 0.8).max(2.0);
    metrics.cost_size = metrics.title_size;
    metrics.title_chars = ((card_w - metrics.pad_x * 2.0) / (metrics.title_size * 0.62))
        .floor()
        .max(8.0) as usize;
    let body_wrap_reserve = (metrics.body_size * 0.72).clamp(4.0, 10.0);
    metrics.body_max_width = (card_w - metrics.pad_x * 2.0 - body_wrap_reserve).max(1.0);
    metrics.body_chars = ((card_w - metrics.pad_x * 2.0 - body_wrap_reserve)
        / (metrics.body_size * 0.62))
        .floor()
        .max(10.0) as usize;
    metrics.min_height = (metrics.top_pad
        + metrics.title_size
        + metrics.title_body_gap
        + metrics.body_size
        + metrics.bottom_pad)
        .max((48.0 * scale).clamp(38.0, CARD_HEIGHT));
    metrics
}

fn module_content_height(def: crate::content::ModuleDef, card_w: f32) -> f32 {
    let metrics = module_box_metrics(card_w);
    let title_lines = wrap_text(def.name, metrics.title_chars);
    let body_lines = wrap_text(def.description, metrics.body_chars);
    let title_height = if title_lines.is_empty() {
        0.0
    } else {
        metrics.title_size * title_lines.len() as f32
            + metrics.title_gap * title_lines.len().saturating_sub(1) as f32
    };
    let body_height = if body_lines.is_empty() {
        0.0
    } else {
        metrics.body_size * body_lines.len() as f32
            + metrics.body_gap * body_lines.len().saturating_sub(1) as f32
    };
    let body_breathing_room = if body_lines.is_empty() {
        0.0
    } else {
        text_bottom_breathing_room(metrics.body_size)
    };
    let header_height = title_height;
    let content_height = metrics.top_pad
        + header_height
        + metrics.title_body_gap
        + body_height
        + body_breathing_room
        + metrics.bottom_pad;

    content_height.max(metrics.min_height)
}

fn event_box_metrics(card_w: f32) -> CardBoxMetrics {
    let mut metrics = module_box_metrics(card_w);
    let scale = (card_w / CARD_WIDTH).clamp(0.0, 2.6);
    metrics.pad_x = (metrics.pad_x * 0.96).max(6.0);
    metrics.top_pad = (metrics.top_pad * 1.02).max(4.0);
    metrics.bottom_pad = (metrics.bottom_pad * 1.04).max(7.0);
    metrics.title_size = (metrics.title_size * 0.76).max(9.0);
    metrics.body_size = (metrics.body_size * 0.74).max(9.0);
    metrics.title_gap = (metrics.title_gap * 0.88).max(1.0);
    metrics.title_body_gap = (metrics.title_body_gap * 0.86).max(2.0);
    metrics.body_gap = (metrics.body_gap * 0.88).max(2.0);
    metrics.cost_size = metrics.title_size;
    metrics.title_chars = ((card_w - metrics.pad_x * 2.0) / (metrics.title_size * 0.56))
        .floor()
        .max(10.0) as usize;
    let body_wrap_reserve = (metrics.body_size * 0.68).clamp(4.0, 10.0);
    metrics.body_max_width = (card_w - metrics.pad_x * 2.0 - body_wrap_reserve).max(1.0);
    metrics.body_chars = ((card_w - metrics.pad_x * 2.0 - body_wrap_reserve)
        / (metrics.body_size * 0.56))
        .floor()
        .max(12.0) as usize;
    metrics.min_height = (metrics.top_pad
        + metrics.title_size
        + metrics.title_body_gap
        + metrics.body_size
        + metrics.bottom_pad)
        .max((44.0 * scale).clamp(36.0, CARD_HEIGHT));
    metrics
}

fn event_choice_content_height(title: &str, body: &str, card_w: f32) -> f32 {
    let metrics = event_box_metrics(card_w);
    let title_lines = wrap_text(title, metrics.title_chars);
    let body_lines = wrap_text(body, metrics.body_chars);
    let title_height = if title_lines.is_empty() {
        0.0
    } else {
        metrics.title_size * title_lines.len() as f32
            + metrics.title_gap * title_lines.len().saturating_sub(1) as f32
    };
    let body_height = if body_lines.is_empty() {
        0.0
    } else {
        metrics.body_size * body_lines.len() as f32
            + metrics.body_gap * body_lines.len().saturating_sub(1) as f32
    };
    let body_breathing_room = if body_lines.is_empty() {
        0.0
    } else {
        text_bottom_breathing_room(metrics.body_size)
    };
    let content_height = metrics.top_pad
        + title_height
        + metrics.title_body_gap
        + body_height
        + body_breathing_room
        + metrics.bottom_pad;

    content_height.max(metrics.min_height)
}

fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    for raw_line in text.split('\n') {
        let mut current = String::new();

        for word in raw_line.split_whitespace() {
            let next_len = if current.is_empty() {
                word.len()
            } else {
                current.len() + 1 + word.len()
            };

            if next_len > max_chars && !current.is_empty() {
                lines.push(current);
                current = String::from(word);
            } else {
                if !current.is_empty() {
                    current.push(' ');
                }
                current.push_str(word);
            }
        }

        if !current.is_empty() {
            lines.push(current);
        } else if raw_line.is_empty() {
            lines.push(String::new());
        }
    }

    lines
}

fn wrap_word_indices_by_width(words: &[&str], size: f32, max_width: f32) -> Vec<Vec<usize>> {
    let mut lines = Vec::new();
    let max_width = max_width.max(0.0);
    let space_width = text_width(" ", size);
    let mut current = Vec::new();
    let mut current_width = 0.0;

    for (index, word) in words.iter().enumerate() {
        let word_width = text_width(word, size);
        let next_width = if current.is_empty() {
            word_width
        } else {
            current_width + space_width + word_width
        };

        if next_width > max_width && !current.is_empty() {
            lines.push(current);
            current = vec![index];
            current_width = word_width;
        } else {
            current.push(index);
            current_width = next_width;
        }
    }

    if !current.is_empty() {
        lines.push(current);
    }

    lines
}

fn wrap_text_by_width(text: &str, size: f32, max_width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    for raw_line in text.split('\n') {
        let words: Vec<&str> = raw_line.split_whitespace().collect();
        if words.is_empty() {
            if raw_line.is_empty() {
                lines.push(String::new());
            }
            continue;
        }

        for indices in wrap_word_indices_by_width(&words, size, max_width) {
            let mut line = String::new();
            for index in indices {
                if !line.is_empty() {
                    line.push(' ');
                }
                line.push_str(words[index]);
            }
            lines.push(line);
        }
    }

    lines
}

fn scaled_card_description(description: &str, statuses: StatusSet) -> String {
    if statuses.focus == 0 && statuses.rhythm == 0 && statuses.momentum == 0 {
        return description.to_string();
    }

    description
        .split('\n')
        .map(|line| scale_card_description_line(line, statuses))
        .collect::<Vec<_>>()
        .join("\n")
}

fn scale_card_description_line(line: &str, statuses: StatusSet) -> String {
    let words: Vec<&str> = line.split_whitespace().collect();
    if words.is_empty() {
        return String::new();
    }

    words
        .iter()
        .enumerate()
        .map(|(index, word)| scale_card_description_word(word, &words, index, statuses))
        .collect::<Vec<_>>()
        .join(" ")
}

fn scale_card_description_word(
    word: &str,
    words: &[&str],
    index: usize,
    statuses: StatusSet,
) -> String {
    let Some((amount, suffix)) = split_unsigned_numeric_token(word) else {
        return word.to_string();
    };

    let Some(kind) = card_description_effect_kind(words, index) else {
        return word.to_string();
    };

    let scaled = match kind {
        CardDescriptionEffectKind::Damage => preview_scaled_value(amount, statuses.focus),
        CardDescriptionEffectKind::Shield => preview_scaled_value(amount, statuses.rhythm),
        CardDescriptionEffectKind::Energy => preview_scaled_value(amount, statuses.momentum),
    };

    format!("{scaled}{suffix}")
}

fn split_unsigned_numeric_token(token: &str) -> Option<(i32, &str)> {
    let digit_len = token
        .chars()
        .take_while(|ch| ch.is_ascii_digit())
        .map(char::len_utf8)
        .sum::<usize>();
    if digit_len == 0 {
        return None;
    }

    let amount = token[..digit_len].parse().ok()?;
    Some((amount, &token[digit_len..]))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CardDescriptionEffectKind {
    Damage,
    Shield,
    Energy,
}

fn card_description_effect_kind(words: &[&str], index: usize) -> Option<CardDescriptionEffectKind> {
    let mut next_index = index + 1;
    let mut token = words
        .get(next_index)
        .map(|word| normalized_card_body_token(word))?;
    if token == "de" {
        next_index += 1;
        token = words
            .get(next_index)
            .map(|word| normalized_card_body_token(word))?;
    }

    if is_damage_term(&token) {
        Some(CardDescriptionEffectKind::Damage)
    } else if is_shield_term(&token) {
        Some(CardDescriptionEffectKind::Shield)
    } else if is_energy_term(&token) {
        Some(CardDescriptionEffectKind::Energy)
    } else {
        None
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CardBodyTint {
    Focus,
    Rhythm,
    Momentum,
}

#[allow(clippy::too_many_arguments)]
fn render_card_description(
    scene: &mut SceneBuilder,
    x: f32,
    start_y: f32,
    size: f32,
    gap: f32,
    text: &str,
    max_width: f32,
    default_color: &str,
) {
    let mut y = start_y;
    for raw_line in text.split('\n') {
        let words: Vec<&str> = raw_line.split_whitespace().collect();
        if words.is_empty() {
            y += size + gap;
            continue;
        }

        for line_indices in wrap_word_indices_by_width(&words, size, max_width) {
            render_card_description_line(scene, x, y, size, &words, &line_indices, default_color);
            y += size + gap;
        }
    }
}

fn render_card_description_line(
    scene: &mut SceneBuilder,
    x: f32,
    y: f32,
    size: f32,
    words: &[&str],
    indices: &[usize],
    default_color: &str,
) {
    let mut cursor_x = x;

    for (position, index) in indices.iter().enumerate() {
        let word = words[*index];
        if position > 0 {
            cursor_x += text_width(" ", size);
        }
        scene.text(
            cursor_x,
            y,
            size,
            "left",
            card_body_tint_color(card_body_token_tint(words, *index), default_color),
            "body",
            word,
        );
        cursor_x += text_width(word, size);
    }
}

fn card_body_tint_color(tint: Option<CardBodyTint>, default_color: &str) -> &str {
    match tint {
        Some(CardBodyTint::Focus) => TERM_GREEN,
        Some(CardBodyTint::Rhythm) => TERM_BLUE_SOFT,
        Some(CardBodyTint::Momentum) => TERM_CYAN,
        None => default_color,
    }
}

fn card_body_token_tint(words: &[&str], index: usize) -> Option<CardBodyTint> {
    let core = normalized_card_body_token(words[index]);
    if core.is_empty() {
        return None;
    }

    if is_focus_term(&core) || is_damage_term(&core) {
        return Some(CardBodyTint::Focus);
    }
    if is_rhythm_term(&core) || is_shield_term(&core) {
        return Some(CardBodyTint::Rhythm);
    }
    if is_momentum_term(&core) || is_energy_term(&core) {
        return Some(CardBodyTint::Momentum);
    }
    if !is_number_token(&core) {
        return None;
    }

    let start = index.saturating_sub(2);
    let end = (index + 2).min(words.len().saturating_sub(1));
    for (neighbor_index, neighbor) in words.iter().enumerate().take(end + 1).skip(start) {
        if neighbor_index == index {
            continue;
        }
        let neighbor = normalized_card_body_token(neighbor);
        if is_focus_term(&neighbor) || is_damage_term(&neighbor) {
            return Some(CardBodyTint::Focus);
        }
        if is_rhythm_term(&neighbor) || is_shield_term(&neighbor) {
            return Some(CardBodyTint::Rhythm);
        }
        if is_momentum_term(&neighbor) || is_energy_term(&neighbor) {
            return Some(CardBodyTint::Momentum);
        }
    }

    None
}

fn normalized_card_body_token(token: &str) -> String {
    token
        .trim_matches(|ch: char| {
            matches!(
                ch,
                '.' | ',' | ':' | ';' | '!' | '?' | '(' | ')' | '[' | ']' | '{' | '}'
            )
        })
        .to_lowercase()
}

fn is_number_token(token: &str) -> bool {
    let trimmed = token.trim_start_matches(['+', '-']);
    !trimmed.is_empty() && trimmed.chars().all(|ch| ch.is_ascii_digit())
}

fn is_focus_term(token: &str) -> bool {
    matches!(token, "focus" | "enfoque")
}

fn is_rhythm_term(token: &str) -> bool {
    matches!(token, "rhythm" | "ritmo" | "shield" | "escudo")
}

fn is_momentum_term(token: &str) -> bool {
    matches!(token, "momentum" | "impulso" | "energy" | "energía")
}

fn is_damage_term(token: &str) -> bool {
    matches!(token, "damage" | "daño")
}

fn is_shield_term(token: &str) -> bool {
    matches!(token, "shield" | "escudo")
}

fn is_energy_term(token: &str) -> bool {
    matches!(token, "energy" | "energía")
}

fn rgba((r, g, b): (u8, u8, u8), alpha: f32) -> String {
    format!("rgba({r}, {g}, {b}, {:.3})", alpha.clamp(0.0, 1.0))
}

fn ui_tile_fill_from_rgb(rgb: (u8, u8, u8), alpha_scale: f32) -> String {
    rgba(rgb, UI_TILE_FILL_ALPHA * alpha_scale.clamp(0.0, 1.0))
}

fn parse_rgba(color: &str) -> Option<((u8, u8, u8), f32)> {
    let inner = color.strip_prefix("rgba(")?.strip_suffix(')')?;
    let mut parts = inner.split(',').map(str::trim);
    let r = parts.next()?.parse().ok()?;
    let g = parts.next()?.parse().ok()?;
    let b = parts.next()?.parse().ok()?;
    let alpha = parts.next()?.parse().ok()?;
    Some(((r, g, b), alpha))
}

fn ui_emphasize_tile_stroke(stroke: &str) -> String {
    parse_rgba(stroke)
        .map(|(rgb, alpha)| {
            rgba(
                rgb,
                (alpha + UI_TILE_STROKE_ALPHA_BOOST).clamp(0.0, 1.0) * UI_TILE_STROKE_ALPHA_SCALE,
            )
        })
        .unwrap_or_else(|| stroke.to_string())
}

fn ui_tile_fill_from_stroke(stroke: &str, alpha_scale: f32) -> String {
    parse_rgba(stroke)
        .map(|(rgb, _)| ui_tile_fill_from_rgb(rgb, alpha_scale))
        .unwrap_or_else(|| COLOR_TILE_FILL.to_string())
}

fn ui_tile_style(stroke: &str, alpha_scale: f32) -> (String, String) {
    let stroke = ui_emphasize_tile_stroke(stroke);
    let fill = ui_tile_fill_from_stroke(&stroke, alpha_scale);
    (fill, stroke)
}

fn render_ui_tile(scene: &mut SceneBuilder, rect: Rect, radius: f32, stroke: &str) {
    render_ui_tile_scaled(scene, rect, radius, stroke, 1.0);
}

fn render_ui_tile_scaled(
    scene: &mut SceneBuilder,
    rect: Rect,
    radius: f32,
    stroke: &str,
    alpha_scale: f32,
) {
    let (fill, stroke) = ui_tile_style(stroke, alpha_scale);
    scene.rect(rect, radius, &fill, &stroke, UI_TILE_STROKE_WIDTH);
}

struct SceneBuilder {
    output: String,
}

impl SceneBuilder {
    fn new() -> Self {
        Self {
            output: String::with_capacity(16_384),
        }
    }

    fn finish(self) -> String {
        self.output
    }

    fn clear(&mut self, color: &str) {
        let _ = writeln!(self.output, "CLEAR|{}", sanitize(color));
    }

    fn push_layer(&mut self, alpha: f32, offset_x: f32, offset_y: f32, scale: f32) {
        let _ = writeln!(
            self.output,
            "PUSH|{alpha:.3}|{offset_x:.2}|{offset_y:.2}|{scale:.4}"
        );
    }

    fn pop_layer(&mut self) {
        let _ = writeln!(self.output, "POP");
    }

    fn line(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, color: &str, width: f32) {
        let _ = writeln!(
            self.output,
            "LINE|{x1:.2}|{y1:.2}|{x2:.2}|{y2:.2}|{}|{width:.2}",
            sanitize(color)
        );
    }

    fn blur_rect(&mut self, rect: Rect, radius: f32, amount: f32) {
        let _ = writeln!(
            self.output,
            "BLUR|{:.2}|{:.2}|{:.2}|{:.2}|{:.2}|{:.2}",
            rect.x, rect.y, rect.w, rect.h, radius, amount
        );
    }

    fn image(&mut self, rect: Rect, src: &str, alpha: f32) {
        let _ = writeln!(
            self.output,
            "IMAGE|{:.2}|{:.2}|{:.2}|{:.2}|{}|{alpha:.3}",
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            sanitize(src)
        );
    }

    fn tinted_image(&mut self, rect: Rect, src: &str, color: &str, alpha: f32) {
        let _ = writeln!(
            self.output,
            "TIMAGE|{:.2}|{:.2}|{:.2}|{:.2}|{}|{}|{alpha:.3}",
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            sanitize(src),
            sanitize(color)
        );
    }

    fn sprite(&mut self, rect: Rect, sprite_code: u8, color: &str, alpha: f32) {
        let _ = writeln!(
            self.output,
            "SPRITE|{:.2}|{:.2}|{:.2}|{:.2}|{sprite_code}|{}|{alpha:.3}",
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            sanitize(color)
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn regular_polygon(
        &mut self,
        center_x: f32,
        center_y: f32,
        radius: f32,
        sides: usize,
        rotation_deg: f32,
        fill: &str,
        stroke: &str,
        stroke_width: f32,
    ) {
        let _ = writeln!(
            self.output,
            "POLY|{center_x:.2}|{center_y:.2}|{radius:.2}|{sides}|{rotation_deg:.2}|{}|{}|{stroke_width:.2}",
            sanitize(fill),
            sanitize(stroke)
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn triangle(
        &mut self,
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
        fill: &str,
        stroke: &str,
        stroke_width: f32,
    ) {
        let _ = writeln!(
            self.output,
            "TRI|{x1:.2}|{y1:.2}|{x2:.2}|{y2:.2}|{x3:.2}|{y3:.2}|{}|{}|{stroke_width:.2}",
            sanitize(fill),
            sanitize(stroke)
        );
    }

    fn rect(&mut self, rect: Rect, radius: f32, fill: &str, stroke: &str, stroke_width: f32) {
        let _ = writeln!(
            self.output,
            "RECT|{:.2}|{:.2}|{:.2}|{:.2}|{:.2}|{}|{}|{:.2}",
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            radius,
            sanitize(fill),
            sanitize(stroke),
            stroke_width
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn text(
        &mut self,
        x: f32,
        y: f32,
        size: f32,
        align: &str,
        color: &str,
        font: &str,
        text: &str,
    ) {
        let _ = writeln!(
            self.output,
            "TEXT|{x:.2}|{y:.2}|{size:.2}|{}|{}|{}|{}",
            sanitize(align),
            sanitize(color),
            sanitize(font),
            sanitize(text)
        );
    }
}

fn sanitize(value: &str) -> String {
    value.replace('|', "/").replace('\n', " ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::{
        CardArchetype, CardDef, CardTarget, CardTraits, EnemyProfileId, ModuleDef, RewardTier,
        enemy_sprite_def, localized_enemy_name, module_def,
    };

    const TEST_RUN_SEED: u64 = 0x0BAD_5EED;
    const TEST_AUXILIARY_SEED: u64 = 0xDEAD_BEEF;
    const TEST_FALLBACK_SEED: u64 = 0xBAAD_F00D;
    const TEST_BUILD_TIMESTAMP: &str = "BUILD_TS";
    const TEST_BUILD_SHA: &str = "BUILD_SHA";
    type FrameTextEntry = (f32, f32, f32, String, String, String, String);

    fn primary_enemy(combat: &CombatState) -> &EnemyState {
        combat
            .enemy(0)
            .expect("test combat should have a primary enemy")
    }

    fn primary_enemy_mut(combat: &mut CombatState) -> &mut EnemyState {
        combat
            .enemy_mut(0)
            .expect("test combat should have a primary enemy")
    }

    fn displayed_primary_enemy(app: &App) -> ActorDisplayedStats {
        app.combat_feedback
            .displayed
            .enemies
            .first()
            .copied()
            .unwrap_or_default()
    }

    fn primary_enemy_rect(layout: &Layout) -> Rect {
        layout
            .enemy_rects
            .first()
            .copied()
            .expect("combat layout should have a primary enemy rect")
    }

    fn enemy_sprite_codes(profile: EnemyProfileId) -> Vec<u8> {
        enemy_sprite_def(profile)
            .layers
            .iter()
            .map(|layer| layer.code)
            .collect()
    }

    fn set_primary_enemy_intent(app: &mut App, profile: EnemyProfileId, intent_index: usize) {
        let enemy = primary_enemy_mut(&mut app.combat);
        enemy.profile = profile;
        enemy.intent_index = intent_index % 3;
    }

    fn advance_time(app: &mut App, total_ms: f32) {
        let mut remaining = total_ms;
        while remaining > 0.0 {
            let step = remaining.min(16.0);
            app.tick(step);
            remaining -= step;
        }
    }

    fn advance_until<F>(app: &mut App, mut predicate: F)
    where
        F: FnMut(&App) -> bool,
    {
        for _ in 0..400 {
            if predicate(app) {
                return;
            }
            advance_time(app, 16.0);
        }
        assert!(predicate(app), "condition was not reached in time");
    }

    fn restore_app_from_snapshot(snapshot: &str) -> App {
        let mut restored = App::new();
        restored
            .restore_from_save_raw(snapshot)
            .expect("save should restore");
        restored
    }

    fn skip_opening_intro(app: &mut App) {
        app.complete_opening_intro();
        app.continue_from_opening_intro();
        app.screen_transition = None;
        app.opening_intro = None;
    }

    fn start_run_to_map(app: &mut App) {
        app.start_run();
        skip_opening_intro(app);
        app.claim_module_select(0);
        app.screen_transition = None;
    }

    fn map_node_center_x(layout: &MapLayout, node_id: usize) -> f32 {
        layout
            .nodes
            .iter()
            .find(|node| node.id == node_id)
            .expect("map layout should include requested node")
            .center_x
    }

    fn active_module_select_fixture() -> App {
        let mut app = App::new();
        app.start_run();
        skip_opening_intro(&mut app);
        app
    }

    fn active_boss_module_select_fixture(boss_level: usize) -> App {
        let mut app = App::new();
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.current_level = boss_level + 1;
        app.dungeon = Some(dungeon);
        app.screen = AppScreen::Reward;
        app.reward = Some(RewardState {
            tier: RewardTier::Boss,
            options: vec![CardId::QuickStrike],
            followup: RewardFollowup {
                completed_run: false,
            },
            seed: TEST_RUN_SEED,
        });
        app.screen_transition = None;
        app
    }

    fn active_combat_fixture() -> App {
        let mut app = App::new();
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.nodes = vec![
            crate::dungeon::DungeonNode {
                id: 0,
                depth: 0,
                lane: 3,
                kind: RoomKind::Start,
                next: vec![1],
            },
            crate::dungeon::DungeonNode {
                id: 1,
                depth: 1,
                lane: 3,
                kind: RoomKind::Combat,
                next: vec![],
            },
        ];
        dungeon.current_node = Some(1);
        dungeon.available_nodes.clear();
        let setup = dungeon.current_encounter_setup().unwrap();
        app.dungeon = Some(dungeon);
        app.begin_encounter(setup);
        app.screen_transition = None;
        app
    }

    fn active_two_enemy_combat_fixture() -> App {
        let mut app = active_combat_fixture();
        let setup = crate::combat::EncounterSetup {
            player_hp: 32,
            player_max_hp: 32,
            player_max_energy: 3,
            enemies: vec![
                crate::combat::EncounterEnemySetup {
                    hp: 16,
                    max_hp: 16,
                    block: 0,
                    profile: EnemyProfileId::HeptarchCore,
                    intent_index: 1,
                    on_hit_bleed: 0,
                },
                crate::combat::EncounterEnemySetup {
                    hp: 16,
                    max_hp: 16,
                    block: 0,
                    profile: EnemyProfileId::HeptarchCore,
                    intent_index: 1,
                    on_hit_bleed: 0,
                },
            ],
        };
        app.begin_encounter(setup);
        app.screen_transition = None;
        app
    }

    fn combat_save_fixture() -> App {
        let mut app = active_combat_fixture();
        app.perform_action(CombatAction::EndTurn);
        app.combat.player.fighter.statuses.focus = 2;
        app.combat.player.fighter.statuses.rhythm = -1;
        app.combat.player.fighter.statuses.momentum = 3;
        primary_enemy_mut(&mut app.combat).fighter.statuses.focus = -1;
        primary_enemy_mut(&mut app.combat).fighter.statuses.rhythm = 2;
        primary_enemy_mut(&mut app.combat).fighter.statuses.momentum = -1;
        app
    }

    fn dense_rest_test_deck() -> Vec<CardId> {
        vec![
            CardId::FlareSlash,
            CardId::GuardStep,
            CardId::Slipstream,
            CardId::QuickStrike,
            CardId::PinpointJab,
            CardId::SignalTap,
            CardId::Reinforce,
            CardId::PressurePoint,
            CardId::BurstArray,
            CardId::CoverPulse,
        ]
    }

    fn active_rest_fixture(deck: Vec<CardId>, player_hp: i32) -> App {
        let mut app = App::new();
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.player_hp = player_hp;
        dungeon.deck = deck;
        dungeon.nodes = vec![
            crate::dungeon::DungeonNode {
                id: 0,
                depth: 0,
                lane: 3,
                kind: RoomKind::Start,
                next: vec![1],
            },
            crate::dungeon::DungeonNode {
                id: 1,
                depth: 1,
                lane: 3,
                kind: RoomKind::Rest,
                next: vec![],
            },
        ];
        dungeon.current_node = Some(1);
        dungeon.available_nodes.clear();
        app.dungeon = Some(dungeon);
        app.begin_rest();
        app.screen_transition = None;
        app
    }

    fn active_shop_fixture() -> App {
        let mut app = App::new();
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.credits = 32;
        dungeon.nodes = vec![
            crate::dungeon::DungeonNode {
                id: 0,
                depth: 0,
                lane: 3,
                kind: RoomKind::Start,
                next: vec![1],
            },
            crate::dungeon::DungeonNode {
                id: 1,
                depth: 1,
                lane: 3,
                kind: RoomKind::Shop,
                next: vec![],
            },
        ];
        dungeon.current_node = Some(1);
        dungeon.available_nodes.clear();
        app.dungeon = Some(dungeon);
        app.begin_shop();
        app.screen_transition = None;
        app
    }

    fn active_event_fixture(event: EventId) -> App {
        let mut app = App::new();
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.player_hp = 20;
        dungeon.nodes = vec![
            crate::dungeon::DungeonNode {
                id: 0,
                depth: 0,
                lane: 3,
                kind: RoomKind::Start,
                next: vec![1],
            },
            crate::dungeon::DungeonNode {
                id: 1,
                depth: 1,
                lane: 3,
                kind: RoomKind::Event,
                next: vec![2],
            },
            crate::dungeon::DungeonNode {
                id: 2,
                depth: 2,
                lane: 3,
                kind: RoomKind::Combat,
                next: vec![],
            },
        ];
        dungeon.current_node = Some(1);
        dungeon.available_nodes = vec![2];
        app.dungeon = Some(dungeon);
        app.screen = AppScreen::Event;
        app.event = Some(EventState { event });
        app
    }

    fn rect_contains_rect(outer: Rect, inner: Rect) -> bool {
        inner.x >= outer.x - 0.01
            && inner.y >= outer.y - 0.01
            && inner.x + inner.w <= outer.x + outer.w + 0.01
            && inner.y + inner.h <= outer.y + outer.h + 0.01
    }

    fn frame_sprite_entries(frame: &str) -> Vec<(u8, Rect)> {
        frame
            .lines()
            .filter_map(|line| {
                let mut parts = line.split('|');
                if parts.next()? != "SPRITE" {
                    return None;
                }
                let x = parts.next()?.parse().ok()?;
                let y = parts.next()?.parse().ok()?;
                let w = parts.next()?.parse().ok()?;
                let h = parts.next()?.parse().ok()?;
                let code = parts.next()?.parse().ok()?;
                Some((code, Rect { x, y, w, h }))
            })
            .collect()
    }

    #[derive(Debug, Clone, PartialEq)]
    struct FrameRectEntry {
        rect: Rect,
        radius: f32,
        fill: String,
        stroke: String,
        stroke_width: f32,
    }

    fn frame_rect_entries(frame: &str) -> Vec<FrameRectEntry> {
        frame
            .lines()
            .filter_map(|line| {
                let mut parts = line.split('|');
                if parts.next()? != "RECT" {
                    return None;
                }
                let x = parts.next()?.parse().ok()?;
                let y = parts.next()?.parse().ok()?;
                let w = parts.next()?.parse().ok()?;
                let h = parts.next()?.parse().ok()?;
                let radius = parts.next()?.parse().ok()?;
                let fill = String::from(parts.next()?);
                let stroke = String::from(parts.next()?);
                let stroke_width = parts.next()?.parse().ok()?;
                Some(FrameRectEntry {
                    rect: Rect { x, y, w, h },
                    radius,
                    fill,
                    stroke,
                    stroke_width,
                })
            })
            .collect()
    }

    fn find_frame_rect_entry(entries: &[FrameRectEntry], rect: Rect) -> &FrameRectEntry {
        entries
            .iter()
            .find(|entry| rects_match(entry.rect, rect))
            .unwrap_or_else(|| panic!("expected RECT entry for {:?}", rect))
    }

    fn assert_ui_tile_entry(entry: &FrameRectEntry, stroke: &str, alpha_scale: f32) {
        let expected_stroke = ui_emphasize_tile_stroke(stroke);
        assert_eq!(
            entry.fill,
            ui_tile_fill_from_stroke(&expected_stroke, alpha_scale)
        );
        assert_eq!(entry.stroke, expected_stroke);
        assert_eq!(entry.stroke_width, UI_TILE_STROKE_WIDTH);
    }

    #[derive(Debug, Clone, PartialEq)]
    struct FrameTintedImageEntry {
        rect: Rect,
        src: String,
        color: String,
    }

    fn frame_tinted_image_entries(frame: &str) -> Vec<FrameTintedImageEntry> {
        frame
            .lines()
            .filter_map(|line| {
                let mut parts = line.split('|');
                if parts.next()? != "TIMAGE" {
                    return None;
                }
                let x = parts.next()?.parse().ok()?;
                let y = parts.next()?.parse().ok()?;
                let w = parts.next()?.parse().ok()?;
                let h = parts.next()?.parse().ok()?;
                let src = String::from(parts.next()?);
                let color = String::from(parts.next()?);
                Some(FrameTintedImageEntry {
                    rect: Rect { x, y, w, h },
                    src,
                    color,
                })
            })
            .collect()
    }

    fn frame_text_entries(frame: &str) -> Vec<FrameTextEntry> {
        frame
            .lines()
            .filter_map(|line| {
                let mut parts = line.split('|');
                if parts.next()? != "TEXT" {
                    return None;
                }
                Some((
                    parts.next()?.parse().ok()?,
                    parts.next()?.parse().ok()?,
                    parts.next()?.parse().ok()?,
                    String::from(parts.next()?),
                    String::from(parts.next()?),
                    String::from(parts.next()?),
                    String::from(parts.next()?),
                ))
            })
            .collect()
    }

    fn panel_has_text_with_color(
        entries: &[FrameTextEntry],
        panel: Rect,
        text: &str,
        color: &str,
    ) -> bool {
        entries
            .iter()
            .any(|(x, y, size, _, entry_color, _, entry_text)| {
                entry_text == text
                    && entry_color == color
                    && *x >= panel.x - 0.01
                    && *x + text_width(entry_text, *size) <= panel.x + panel.w + 0.01
                    && *y >= panel.y - 0.01
                    && *y <= panel.y + panel.h + 0.01
            })
    }

    fn player_active_stats(app: &App) -> Vec<CombatStat> {
        app.combat_feedback
            .active_stats
            .iter()
            .filter(|active| active.actor == Actor::Player)
            .map(|active| active.stat)
            .collect()
    }

    fn button_label_fits(button: FittedPrimaryButton, label: &str) -> bool {
        text_width(label, button.font_size) <= button.rect.w + 0.01
            && button.font_size <= button.rect.h + 0.01
    }

    fn wrapped_lines_fit_width(lines: &[String], font_size: f32, max_width: f32) -> bool {
        lines
            .iter()
            .all(|line| text_width(line, font_size) <= max_width + 0.01)
    }

    #[test]
    fn enemy_sprite_palette_tracks_enemy_level() {
        assert_eq!(
            enemy_sprite_layer_color(EnemyProfileId::ScoutDrone, EnemySpriteLayerTone::Base, true),
            ENEMY_LEVEL_ONE_SPRITE_PALETTE.base
        );
        assert_eq!(
            enemy_sprite_layer_color(EnemyProfileId::VoltMantis, EnemySpriteLayerTone::Base, true),
            ENEMY_LEVEL_TWO_SPRITE_PALETTE.base
        );
        assert_eq!(
            enemy_sprite_layer_color(EnemyProfileId::NullRaider, EnemySpriteLayerTone::Base, true),
            ENEMY_LEVEL_THREE_SPRITE_PALETTE.base
        );
        assert_eq!(
            enemy_sprite_layer_color(
                EnemyProfileId::HeptarchCore,
                EnemySpriteLayerTone::DetailC,
                true,
            ),
            ENEMY_LEVEL_THREE_SPRITE_PALETTE.detail_c
        );
    }

    #[test]
    fn enemy_level_three_icons_render_more_present_than_level_one() {
        assert!(
            enemy_panel_icon_alpha(EnemyProfileId::NullRaider, true)
                > enemy_panel_icon_alpha(EnemyProfileId::ScoutDrone, true)
        );
        assert_eq!(
            enemy_sprite_layer_color(
                EnemyProfileId::NullRaider,
                EnemySpriteLayerTone::DetailC,
                true
            ),
            ENEMY_LEVEL_THREE_SPRITE_PALETTE.detail_c
        );
    }

    #[test]
    fn ui_tile_fill_from_stroke_reuses_border_rgb_and_scales_alpha() {
        assert_eq!(
            ui_tile_fill_from_stroke(COLOR_GREEN_STROKE_IDLE, 1.0),
            rgba((51, 255, 102), UI_TILE_FILL_ALPHA)
        );
        assert_eq!(
            ui_tile_fill_from_stroke(COLOR_LIME_STROKE_TARGET, 0.5),
            rgba((216, 255, 61), UI_TILE_FILL_ALPHA * 0.5)
        );
    }

    #[test]
    fn ui_emphasize_tile_stroke_adds_the_standard_alpha_boost() {
        assert_eq!(
            ui_emphasize_tile_stroke(COLOR_GREEN_STROKE_IDLE),
            rgba(
                (51, 255, 102),
                (0.55 + UI_TILE_STROKE_ALPHA_BOOST).clamp(0.0, 1.0) * UI_TILE_STROKE_ALPHA_SCALE
            )
        );
        assert_eq!(
            ui_emphasize_tile_stroke(COLOR_GREEN_STROKE_STRONG),
            rgba(
                (51, 255, 102),
                (0.92 + UI_TILE_STROKE_ALPHA_BOOST).clamp(0.0, 1.0) * UI_TILE_STROKE_ALPHA_SCALE
            )
        );
    }

    fn module_tile_copy_fits_rect(def: ModuleDef, rect: Rect) -> bool {
        let metrics = module_box_metrics(rect.w);
        let inner_width = rect.w - metrics.pad_x * 2.0;
        let title_lines = wrap_text(def.name, metrics.title_chars);
        let body_lines = wrap_text(def.description, metrics.body_chars);
        let title_height = if title_lines.is_empty() {
            0.0
        } else {
            metrics.title_size * title_lines.len() as f32
                + metrics.title_gap * title_lines.len().saturating_sub(1) as f32
        };
        let body_height = if body_lines.is_empty() {
            0.0
        } else {
            metrics.body_size * body_lines.len() as f32
                + metrics.body_gap * body_lines.len().saturating_sub(1) as f32
        };
        let body_breathing_room = if body_lines.is_empty() {
            0.0
        } else {
            text_bottom_breathing_room(metrics.body_size)
        };
        let content_bottom = rect.y
            + metrics.top_pad
            + title_height
            + metrics.title_body_gap
            + body_height
            + body_breathing_room
            + metrics.bottom_pad;

        wrapped_lines_fit_width(&title_lines, metrics.title_size, inner_width)
            && wrapped_lines_fit_width(&body_lines, metrics.body_size, inner_width)
            && content_bottom <= rect.y + rect.h + 0.01
    }

    fn body_text_entries_in_rect(frame: &str, rect: Rect) -> Vec<FrameTextEntry> {
        frame_text_entries(frame)
            .into_iter()
            .filter(|(x, y, _, _, _, font, _)| {
                font == "body"
                    && *x >= rect.x - 0.01
                    && *x <= rect.x + rect.w + 0.01
                    && *y >= rect.y - 0.01
                    && *y <= rect.y + rect.h + 0.01
            })
            .collect()
    }

    #[test]
    fn selection_card_colors_damage_shield_momentum_and_cost_in_english() {
        let app = App::new();
        let mut scene = SceneBuilder::new();
        app.render_selection_card(
            &mut scene,
            Rect {
                x: 0.0,
                y: 0.0,
                w: CARD_WIDTH,
                h: CARD_HEIGHT,
            },
            CardId::VectorLock,
            COLOR_GREEN_STROKE_CARD,
        );

        let frame = scene.finish();
        let entries = frame_text_entries(&frame);

        assert!(entries.iter().any(|(_, _, _, _, color, font, text)| {
            font == "body" && text == "6" && color == TERM_GREEN
        }));
        assert!(entries.iter().any(|(_, _, _, _, color, font, text)| {
            font == "body" && text == "Momentum" && color == TERM_CYAN
        }));
        assert!(entries.iter().any(|(_, _, _, _, color, font, text)| {
            font == "body" && text.starts_with("-2") && color == TERM_CYAN
        }));
        assert!(entries.iter().any(|(_, _, _, _, color, font, text)| {
            font == "body" && text == "5" && color == TERM_BLUE_SOFT
        }));
        assert!(entries.iter().any(|(_, _, _, _, color, font, text)| {
            font == "display" && text == "1" && color == TERM_CYAN
        }));
    }

    #[test]
    fn selection_card_colors_shield_and_momentum_segments_in_spanish() {
        let mut app = App::new();
        app.language = Language::Spanish;
        let mut scene = SceneBuilder::new();
        app.render_selection_card(
            &mut scene,
            Rect {
                x: 0.0,
                y: 0.0,
                w: CARD_WIDTH,
                h: CARD_HEIGHT,
            },
            CardId::CapacitiveShell,
            COLOR_GREEN_STROKE_CARD,
        );

        let frame = scene.finish();
        let entries = frame_text_entries(&frame);

        assert!(entries.iter().any(|(_, _, _, _, color, font, text)| {
            font == "body" && text == "Escudo." && color == TERM_BLUE_SOFT
        }));
        assert!(entries.iter().any(|(_, _, _, _, color, font, text)| {
            font == "body" && text == "Impulso" && color == TERM_CYAN
        }));
        assert!(entries.iter().any(|(_, _, _, _, color, font, text)| {
            font == "body" && text == "+2." && color == TERM_CYAN
        }));
        assert!(entries.iter().any(|(_, _, _, _, color, font, text)| {
            font == "display" && text == "1" && color == TERM_CYAN
        }));
    }

    #[test]
    fn scaled_card_description_updates_scaled_effect_numbers_in_english() {
        let description = "Deal 5 damage. Gain 5 Shield. Gain 2 Energy. Apply Focus -1.";
        let statuses = StatusSet {
            focus: 1,
            rhythm: 2,
            momentum: 2,
            ..StatusSet::default()
        };

        assert_eq!(
            scaled_card_description(description, statuses),
            "Deal 6 damage. Gain 6 Shield. Gain 2 Energy. Apply Focus -1."
        );
    }

    #[test]
    fn scaled_card_description_updates_scaled_effect_numbers_in_spanish() {
        let description =
            "Inflige 5 de daño. Gana 5 de Escudo. Gana 2 de Energía. Aplica Enfoque -1.";
        let statuses = StatusSet {
            focus: 1,
            rhythm: 2,
            momentum: 2,
            ..StatusSet::default()
        };

        assert_eq!(
            scaled_card_description(description, statuses),
            "Inflige 6 de daño. Gana 6 de Escudo. Gana 2 de Energía. Aplica Enfoque -1."
        );
    }

    #[test]
    fn combat_card_description_reflects_current_rhythm_scaling() {
        let mut app = active_combat_fixture();
        app.combat.player.fighter.statuses.rhythm = 3;

        assert_eq!(
            app.combat_card_description(CardId::GuardStep),
            "Gain 7 Shield."
        );
    }

    #[test]
    fn combat_layout_prefers_single_row_hand_in_landscape_with_four_cards() {
        let mut app = active_combat_fixture();
        app.combat.deck.hand = vec![
            CardId::QuickStrike,
            CardId::GuardStep,
            CardId::FlareSlash,
            CardId::PinpointJab,
        ];
        app.combat.deck.draw_pile.clear();
        app.combat.deck.discard_pile.clear();
        app.resize(1280.0, 720.0);

        let layout = app.layout();

        assert_eq!(layout.hand_arrangement.row_counts, vec![4]);
    }

    #[test]
    fn combat_layout_prefers_two_by_two_hand_in_portrait_with_four_cards() {
        let mut app = active_combat_fixture();
        app.combat.deck.hand = vec![
            CardId::QuickStrike,
            CardId::GuardStep,
            CardId::FlareSlash,
            CardId::PinpointJab,
        ];
        app.combat.deck.draw_pile.clear();
        app.combat.deck.discard_pile.clear();
        app.resize(320.0, 568.0);

        let layout = app.layout();

        assert_eq!(layout.hand_arrangement.row_counts, vec![2, 2]);
    }

    #[test]
    fn combat_layout_prefers_three_by_three_hand_in_portrait_with_nine_cards() {
        let mut app = active_combat_fixture();
        app.combat.deck.hand = vec![
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
        app.combat.deck.draw_pile.clear();
        app.combat.deck.discard_pile.clear();
        app.resize(320.0, 568.0);

        let layout = app.layout();
        let viewport = Rect {
            x: 0.0,
            y: 0.0,
            w: 320.0,
            h: 568.0,
        };

        assert_eq!(layout.hand_arrangement.row_counts, vec![3, 3, 3]);
        assert!(rect_contains_rect(viewport, combat_layout_bounds(&layout)));
    }

    #[test]
    fn combat_layout_handles_tiny_viewports_without_panicking() {
        let mut app = active_combat_fixture();
        app.resize(40.0, 40.0);

        let layout = app.layout();

        assert!(layout.player_rect.w.is_finite());
        assert!(layout.player_rect.h.is_finite());
    }

    #[test]
    fn combat_layout_prefers_single_enemy_row_in_landscape() {
        let mut app = active_two_enemy_combat_fixture();
        app.resize(1280.0, 720.0);

        let layout = app.layout();

        assert_eq!(layout.enemy_arrangement.row_counts, vec![2]);
    }

    #[test]
    fn combat_layout_keeps_enemy_row_in_portrait_when_status_wrap_reduces_panel_width() {
        let mut app = active_two_enemy_combat_fixture();
        for enemy in &mut app.combat.enemies {
            enemy.fighter.statuses = StatusSet {
                bleed: 1,
                focus: 1,
                rhythm: -1,
                momentum: 1,
            };
        }
        app.resize(320.0, 568.0);

        let layout = app.layout();

        assert_eq!(layout.enemy_arrangement.row_counts, vec![2]);
    }

    #[test]
    fn enemy_turn_playback_can_recompute_enemy_arrangement_after_hiding_the_hand() {
        let mut app = active_two_enemy_combat_fixture();
        app.combat.deck.hand = vec![
            CardId::QuickStrike,
            CardId::GuardStep,
            CardId::FlareSlash,
            CardId::PinpointJab,
        ];
        app.combat.deck.draw_pile.clear();
        app.combat.deck.discard_pile.clear();
        app.resize(430.0, 640.0);
        let baseline_layout = app.layout();

        app.perform_action(CombatAction::EndTurn);
        advance_until(&mut app, |app| {
            app.combat_feedback.playback_kind == Some(CombatPlaybackKind::EnemyTurn)
                && app.layout().hand_rects.is_empty()
                && app.layout_transition.is_none()
        });

        let collapsed_layout = app.layout();

        assert!(
            baseline_layout.enemy_arrangement.row_counts
                != collapsed_layout.enemy_arrangement.row_counts
                || !rect_vecs_match(&baseline_layout.enemy_rects, &collapsed_layout.enemy_rects)
        );
        assert!(collapsed_layout.hand_arrangement.row_counts.is_empty());
    }

    #[test]
    fn combat_hint_bar_smoothly_interpolates_when_message_width_changes() {
        let hand_sets = [
            vec![CardId::QuickStrike],
            vec![
                CardId::QuickStrike,
                CardId::GuardStep,
                CardId::FlareSlash,
                CardId::PinpointJab,
            ],
            vec![
                CardId::QuickStrike,
                CardId::GuardStep,
                CardId::FlareSlash,
                CardId::PinpointJab,
                CardId::BurstArray,
                CardId::CoverPulse,
                CardId::BarrierField,
                CardId::TacticalBurst,
                CardId::RazorNet,
            ],
        ];
        let mut scenario = None;

        'search: for two_enemies in [false, true] {
            for language in [Language::English, Language::Spanish] {
                for (width, height) in [(320.0, 568.0), (360.0, 640.0), (430.0, 640.0)] {
                    for hand in &hand_sets {
                        let mut app = if two_enemies {
                            active_two_enemy_combat_fixture()
                        } else {
                            active_combat_fixture()
                        };
                        app.set_language(language);
                        app.combat.deck.hand = hand.clone();
                        app.combat.deck.draw_pile.clear();
                        app.combat.deck.discard_pile.clear();
                        app.resize(width, height);

                        let before = app.layout();
                        app.select_or_play_card(0);
                        let target = app.layout_target();
                        let other_tiles_changed =
                            !rects_match(before.player_rect, target.player_rect)
                                || !rect_vecs_match(&before.hand_rects, &target.hand_rects)
                                || !rect_vecs_match(&before.enemy_rects, &target.enemy_rects);

                        if other_tiles_changed {
                            scenario = Some((app, before, target));
                            break 'search;
                        }
                    }
                }
            }
        }

        let Some((mut app, before, target)) = scenario else {
            panic!("expected at least one combat layout where the hint changes other tiles");
        };
        let at_start = app.layout();

        assert!(app.layout_transition.is_some());
        assert!(!combat_layouts_match(&before, &target));
        assert!(rects_match(before.player_rect, at_start.player_rect));
        assert!(optional_rects_match(before.hint_rect, at_start.hint_rect));

        advance_time(&mut app, LAYOUT_TRANSITION_MS * 0.5);
        let mid = app.layout();
        let mid_hint = mid.hint_rect.unwrap();
        let before_hint = before.hint_rect.unwrap();
        let target_hint = target.hint_rect.unwrap();

        assert!(mid_hint.w > before_hint.w.min(target_hint.w));
        assert!(mid_hint.w < before_hint.w.max(target_hint.w));
        assert!(mid.player_rect.y > before.player_rect.y.min(target.player_rect.y));
        assert!(mid.player_rect.y < before.player_rect.y.max(target.player_rect.y));

        advance_time(&mut app, LAYOUT_TRANSITION_MS);
        let final_layout = app.layout();

        assert!(combat_layouts_match(&final_layout, &target));
    }

    #[test]
    fn combat_layout_smoothly_interpolates_when_enemy_signal_wrap_changes() {
        let mut app = active_combat_fixture();
        app.set_language(Language::Spanish);
        app.resize(320.0, 568.0);

        set_primary_enemy_intent(&mut app, EnemyProfileId::HeptarchCore, 2);
        app.sync_combat_feedback_to_combat();
        let before = app.layout();

        app.snapshot_combat_layout_target();
        set_primary_enemy_intent(&mut app, EnemyProfileId::HeptarchCore, 1);
        app.sync_combat_feedback_to_combat();
        app.refresh_combat_layout_transition();

        let target = app.layout_target();
        let at_start = app.layout();

        assert!(app.layout_transition.is_some());
        assert!(!combat_layouts_match(&before, &target));
        assert!(rect_vecs_match(&before.enemy_rects, &at_start.enemy_rects));
        assert!(rects_match(before.player_rect, at_start.player_rect));

        advance_time(&mut app, LAYOUT_TRANSITION_MS * 0.5);
        let mid = app.layout();

        assert!(
            mid.player_rect.y > before.player_rect.y.min(target.player_rect.y)
                && mid.player_rect.y < before.player_rect.y.max(target.player_rect.y)
        );

        advance_time(&mut app, LAYOUT_TRANSITION_MS);
        let final_layout = app.layout();

        assert!(combat_layouts_match(&final_layout, &target));
    }

    #[test]
    fn combat_layout_retarget_preserves_the_in_flight_interpolated_layout() {
        let mut app = active_combat_fixture();
        app.set_language(Language::Spanish);
        app.resize(320.0, 568.0);

        set_primary_enemy_intent(&mut app, EnemyProfileId::HeptarchCore, 2);
        app.sync_combat_feedback_to_combat();
        let initial_layout = app.layout();

        app.snapshot_combat_layout_target();
        set_primary_enemy_intent(&mut app, EnemyProfileId::HeptarchCore, 1);
        app.sync_combat_feedback_to_combat();
        app.refresh_combat_layout_transition();

        let first_target = app.layout_target();
        assert!(app.layout_transition.is_some());
        assert!(!combat_layouts_match(&initial_layout, &first_target));

        advance_time(&mut app, LAYOUT_TRANSITION_MS * 0.5);
        let mid = app.layout();

        set_primary_enemy_intent(&mut app, EnemyProfileId::HeptarchCore, 2);
        app.sync_combat_feedback_to_combat();
        app.refresh_combat_layout_transition();

        let second_target = app.layout_target();
        let transition = app.layout_transition.as_ref().unwrap();

        assert!(!combat_layouts_match(&first_target, &second_target));
        assert!(combat_layouts_match(&transition.from_layout, &mid));
        assert!(combat_layouts_match(&transition.to_layout, &second_target));
        assert!(combat_layouts_match(&app.layout(), &mid));

        advance_time(&mut app, LAYOUT_TRANSITION_MS);
        let final_layout = app.layout();

        assert!(combat_layouts_match(&final_layout, &second_target));
    }

    #[test]
    fn enemy_panel_wraps_long_next_signal_without_growing_wider() {
        let mut app = active_combat_fixture();
        let tile_insets = tile_insets_for_card_width(CARD_WIDTH);

        set_primary_enemy_intent(&mut app, EnemyProfileId::HeptarchCore, 2);
        app.sync_combat_feedback_to_combat();
        let short_metrics = enemy_panel_metrics(&app, 0, false, 1.0, tile_insets);

        set_primary_enemy_intent(&mut app, EnemyProfileId::HeptarchCore, 1);
        app.sync_combat_feedback_to_combat();
        let long_metrics = enemy_panel_metrics(&app, 0, false, 1.0, tile_insets);
        let long_lines = enemy_intent_lines(
            app.tr(NEXT_SIGNAL_LABEL, "Siguiente"),
            app.enemy_signal_summary(0),
            long_metrics.info_body_chars,
        );

        assert!(
            (short_metrics.width - long_metrics.width).abs() < 0.1,
            "long next-signal text should not widen the enemy panel"
        );
        assert!(
            long_metrics.height > short_metrics.height,
            "long next-signal text should increase panel height instead"
        );
        assert!(
            long_lines.line_count() > 1,
            "long next-signal text should wrap to multiple lines"
        );
    }

    #[test]
    fn enemy_intent_lines_allow_label_only_first_line_when_summary_cannot_share_it() {
        let lines = enemy_intent_lines("Siguiente", "Escudo", "Siguiente".len());

        assert_eq!(lines.first_line_label, "Siguiente");
        assert!(lines.first_line_summary.is_empty());
        assert_eq!(lines.continuation_lines, vec![String::from("Escudo")]);
    }

    #[test]
    fn enemy_panel_renders_next_label_inline_with_signal_summary_in_both_languages() {
        let mut app = active_combat_fixture();

        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        let entries = frame_text_entries(&frame);
        let next_label = entries
            .iter()
            .find(|(_, _, _, _, _, font, text)| font == "label" && text == "Next ")
            .expect("enemy panel should render Next label");
        let next_summary = entries
            .iter()
            .find(|(_, y, _, _, color, font, text)| {
                (*y - next_label.1).abs() < 0.01
                    && font == "body"
                    && color == TERM_CYAN_SOFT
                    && !text.is_empty()
            })
            .expect("enemy panel should render summary inline with Next");
        assert!(next_summary.0 > next_label.0);
        assert!(!next_summary.6.starts_with("Next"));

        app.language = Language::Spanish;
        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        let entries = frame_text_entries(&frame);
        let next_label = entries
            .iter()
            .find(|(_, _, _, _, _, font, text)| font == "label" && text == "Siguiente ")
            .expect("enemy panel should render Siguiente label");
        let next_summary = entries
            .iter()
            .find(|(_, y, _, _, color, font, text)| {
                (*y - next_label.1).abs() < 0.01
                    && font == "body"
                    && color == TERM_CYAN_SOFT
                    && !text.is_empty()
            })
            .expect("enemy panel should render summary inline with Siguiente");
        assert!(next_summary.0 > next_label.0);
        assert!(!next_summary.6.starts_with("Siguiente"));
    }

    #[test]
    fn set_language_relocalizes_cached_enemy_intents_in_combat() {
        let mut app = active_combat_fixture();
        set_primary_enemy_intent(&mut app, EnemyProfileId::ScoutDrone, 0);
        app.sync_combat_feedback_to_combat();

        assert_eq!(app.enemy_signal_summary(0), "Deal 5 damage.");

        app.set_language(Language::Spanish);

        assert_eq!(app.enemy_signal_summary(0), "Inflige 5 de daño.");
        assert_eq!(
            app.combat_feedback.displayed_intents[0].name,
            "Aguja de Choque"
        );
    }

    #[test]
    fn intent_advance_keeps_enemy_signal_summary_in_spanish() {
        let mut app = active_combat_fixture();
        app.set_language(Language::Spanish);
        set_primary_enemy_intent(&mut app, EnemyProfileId::ScoutDrone, 0);
        app.sync_combat_feedback_to_combat();

        app.perform_action(CombatAction::EndTurn);

        advance_until(&mut app, |app| {
            app.enemy_signal_summary(0) == "Inflige 3 de daño dos veces."
        });

        assert_eq!(app.enemy_signal_summary(0), "Inflige 3 de daño dos veces.");
        assert!(!app.enemy_signal_summary(0).contains("Deal"));
        assert!(
            app.log
                .iter()
                .any(|line| line.ends_with("siguiente accion: Fuego Cruzado."))
        );
    }

    #[test]
    fn enemy_panel_keeps_wrapped_spanish_intent_text_inside_the_panel() {
        let mut app = active_two_enemy_combat_fixture();
        app.language = Language::Spanish;
        app.resize(320.0, 568.0);
        set_primary_enemy_intent(&mut app, EnemyProfileId::HeptarchCore, 1);
        app.sync_combat_feedback_to_combat();
        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        let panel = primary_enemy_rect(&app.layout());
        let summary_entries = frame_text_entries(&frame)
            .into_iter()
            .filter(|(x, y, size, _, color, font, text)| {
                *x >= panel.x - 0.01
                    && *x <= panel.x + panel.w + 0.01
                    && *y >= panel.y - 0.01
                    && *y <= panel.y + panel.h + 0.01
                    && font == "body"
                    && color == TERM_CYAN_SOFT
                    && !text.is_empty()
                    && *x + text_width(text, *size) <= panel.x + panel.w + 0.01
            })
            .collect::<Vec<_>>();

        assert!(
            summary_entries.len() > 1,
            "wrapped spanish intent text should span multiple body lines in a narrow panel"
        );
        assert!(
            summary_entries
                .iter()
                .all(|(_, _, _, _, _, _, text)| !text.starts_with("Siguiente")),
            "summary text should never repeat the green Siguiente label in blue"
        );
    }

    #[test]
    fn enemy_panel_emits_a_sprite_command() {
        let mut app = active_combat_fixture();
        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        let sprites = frame_sprite_entries(&frame);
        let expected_codes = enemy_sprite_codes(primary_enemy(&app.combat).profile);

        assert_eq!(sprites.len(), expected_codes.len());
        assert_eq!(
            sprites.iter().map(|(code, _)| *code).collect::<Vec<_>>(),
            expected_codes
        );
    }

    #[test]
    fn enemy_sprite_stays_inside_enemy_panel_top_row() {
        let mut app = active_combat_fixture();
        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        let sprites = frame_sprite_entries(&frame);
        let panel = primary_enemy_rect(&app.layout());
        let insets = app.layout().tile_insets;
        let expected_count = enemy_sprite_codes(primary_enemy(&app.combat).profile).len();

        assert_eq!(sprites.len(), expected_count);
        assert!(
            sprites
                .iter()
                .all(|(_, sprite_rect)| rect_contains_rect(panel, *sprite_rect))
        );
        assert!(
            sprites.iter().all(|(_, sprite_rect)| {
                (sprite_rect.x - (panel.x + insets.pad_x)).abs() < 0.02
            })
        );
        assert!(
            sprites
                .iter()
                .all(|(_, sprite_rect)| sprite_rect.y >= panel.y + insets.top_pad - 0.02)
        );
    }

    #[test]
    fn enemy_panel_does_not_render_enemy_name_text() {
        let mut app = active_combat_fixture();
        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        let panel = primary_enemy_rect(&app.layout());
        let enemy_name =
            localized_enemy_name(primary_enemy(&app.combat).profile, Language::English);
        let panel_texts = frame_text_entries(&frame);

        assert!(panel_texts.iter().all(|(x, y, _, _, _, _, text)| {
            !(*x >= panel.x - 0.01
                && *x <= panel.x + panel.w + 0.01
                && *y >= panel.y - 0.01
                && *y <= panel.y + panel.h + 0.01
                && text == enemy_name)
        }));
    }

    #[test]
    fn enemy_panel_top_row_uses_consistent_group_gap() {
        let mut app = active_combat_fixture();
        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        let panel = primary_enemy_rect(&app.layout());
        let sprite_rect = frame_sprite_entries(&frame)
            .into_iter()
            .map(|(_, rect)| rect)
            .find(|rect| rect_contains_rect(panel, *rect))
            .expect("enemy panel should render a sprite");
        let icons = frame_tinted_image_entries(&frame);
        let heart_icon = icons
            .iter()
            .find(|entry| {
                entry.src == COMBAT_HEART_ICON_ASSET_PATH && rect_contains_rect(panel, entry.rect)
            })
            .expect("enemy panel should render a heart icon");
        let shield_icon = icons
            .iter()
            .find(|entry| {
                entry.src == COMBAT_SHIELD_ICON_ASSET_PATH && rect_contains_rect(panel, entry.rect)
            })
            .expect("enemy panel should render a shield icon");
        let max_hp_text = format!("/{}", primary_enemy(&app.combat).fighter.max_hp);
        let max_hp_entry = frame_text_entries(&frame)
            .into_iter()
            .find(|(x, y, _, _, _, _, text)| {
                *x >= panel.x - 0.01
                    && *x <= panel.x + panel.w + 0.01
                    && *y >= panel.y - 0.01
                    && *y <= panel.y + panel.h + 0.01
                    && text == &max_hp_text
            })
            .expect("enemy panel should render max hp text");
        let sprite_gap = heart_icon.rect.x - (sprite_rect.x + sprite_rect.w);
        let hp_group_gap =
            shield_icon.rect.x - (max_hp_entry.0 + text_width(&max_hp_text, max_hp_entry.2));

        assert!((sprite_gap - hp_group_gap).abs() < 0.05);
    }

    #[test]
    fn overlay_button_row_stacks_only_when_horizontal_fit_is_impossible() {
        let row_metrics = fit_overlay_button_metrics(&["Cancelar", "Reiniciar"], 260.0);
        let stacked_metrics = fit_overlay_button_metrics(&["Cancelar", "Reiniciar"], 200.0);

        assert_eq!(row_metrics.flow, OverlayButtonFlow::Row);
        assert_eq!(stacked_metrics.flow, OverlayButtonFlow::Stack);
        assert!(row_metrics.block_w <= 260.0 + 0.01);
        assert!(stacked_metrics.block_w <= 200.0 + 0.01);
    }

    #[test]
    fn restart_confirm_buttons_fit_inside_modal_in_english() {
        let mut app = App::new();
        app.set_saved_run_available(true);
        app.set_language(Language::English);

        for (width, height) in [(320.0, 568.0), (360.0, 640.0)] {
            app.resize(width, height);
            let layout = app.restart_confirm_layout().unwrap();
            let viewport = Rect {
                x: 0.0,
                y: 0.0,
                w: width,
                h: height,
            };

            assert!(rect_contains_rect(viewport, layout.modal_rect));
            assert!(rect_contains_rect(
                layout.modal_rect,
                layout.cancel_button.rect
            ));
            assert!(rect_contains_rect(
                layout.modal_rect,
                layout.restart_button.rect
            ));
            assert!(button_label_fits(
                layout.cancel_button,
                app.tr(BOOT_RESTART_CONFIRM_CANCEL_LABEL, "Cancelar")
            ));
            assert!(button_label_fits(
                layout.restart_button,
                app.tr(BOOT_RESTART_LABEL, "Reiniciar")
            ));
        }
    }

    #[test]
    fn restart_confirm_buttons_fit_inside_modal_in_spanish() {
        let mut app = App::new();
        app.set_saved_run_available(true);
        app.set_language(Language::Spanish);

        for (width, height) in [(320.0, 568.0), (360.0, 640.0)] {
            app.resize(width, height);
            let layout = app.restart_confirm_layout().unwrap();
            let viewport = Rect {
                x: 0.0,
                y: 0.0,
                w: width,
                h: height,
            };

            assert!(rect_contains_rect(viewport, layout.modal_rect));
            assert!(rect_contains_rect(
                layout.modal_rect,
                layout.cancel_button.rect
            ));
            assert!(rect_contains_rect(
                layout.modal_rect,
                layout.restart_button.rect
            ));
            assert!(button_label_fits(
                layout.cancel_button,
                app.tr(BOOT_RESTART_CONFIRM_CANCEL_LABEL, "Cancelar")
            ));
            assert!(button_label_fits(
                layout.restart_button,
                app.tr(BOOT_RESTART_LABEL, "Reiniciar")
            ));
        }
    }

    #[test]
    fn settings_buttons_fit_inside_modal_in_both_languages() {
        let mut app = App::new();

        for language in [Language::English, Language::Spanish] {
            app.set_language(language);
            app.resize(320.0, 568.0);
            let layout = app.settings_layout();
            let viewport = Rect {
                x: 0.0,
                y: 0.0,
                w: 320.0,
                h: 568.0,
            };

            assert!(rect_contains_rect(viewport, layout.modal_rect));
            assert!(rect_contains_rect(
                layout.modal_rect,
                layout.english_button.rect
            ));
            assert!(rect_contains_rect(
                layout.modal_rect,
                layout.spanish_button.rect
            ));
            assert!(button_label_fits(layout.english_button, "English"));
            assert!(button_label_fits(layout.spanish_button, "Español"));
        }
    }

    #[test]
    fn install_help_close_button_fits_inside_modal_in_both_languages() {
        let mut app = App::new();

        for language in [Language::English, Language::Spanish] {
            app.set_language(language);
            app.resize(320.0, 568.0);
            let layout = app.install_help_layout();
            let viewport = Rect {
                x: 0.0,
                y: 0.0,
                w: 320.0,
                h: 568.0,
            };

            assert!(rect_contains_rect(viewport, layout.modal_rect));
            assert!(rect_contains_rect(
                layout.modal_rect,
                layout.close_button.rect
            ));
            assert!(button_label_fits(
                layout.close_button,
                app.tr("Close", "Cerrar")
            ));
        }
    }

    #[test]
    fn card_body_width_wrapping_keeps_each_line_inside_available_space() {
        let metrics = card_box_metrics(96.0);
        let lines = wrap_text_by_width(
            "Gana Ritmo +1. Roba 1. Gana 2 de Escudo.",
            metrics.body_size,
            metrics.body_max_width,
        );

        assert!(lines.len() > 1);
        assert!(wrapped_lines_fit_width(
            &lines,
            metrics.body_size,
            metrics.body_max_width
        ));
    }

    #[test]
    fn selection_card_keeps_wrapped_spanish_body_inside_card_bounds() {
        let mut app = App::new();
        app.set_language(Language::Spanish);
        let def = app.localized_card_def(CardId::Slipstream);
        let card_w = 96.0;
        let rect = Rect {
            x: 0.0,
            y: 0.0,
            w: card_w,
            h: card_content_height(def, card_w),
        };
        let mut scene = SceneBuilder::new();
        app.render_selection_card(
            &mut scene,
            rect,
            CardId::Slipstream,
            COLOR_GREEN_STROKE_CARD,
        );

        let frame = scene.finish();
        let body_entries = body_text_entries_in_rect(&frame, rect);

        assert!(
            body_entries
                .iter()
                .any(|(_, _, _, _, _, _, text)| text == "de")
        );
        assert!(body_entries.iter().all(|(x, _, size, _, _, _, text)| {
            *x + text_width(text, *size) <= rect.x + rect.w + 0.01
        }));
    }

    #[test]
    fn combat_hand_card_keeps_wrapped_spanish_body_inside_card_bounds() {
        let mut app = active_combat_fixture();
        app.set_language(Language::Spanish);
        app.combat.deck.hand = vec![
            CardId::Slipstream,
            CardId::QuickStrike,
            CardId::GuardStep,
            CardId::FlareSlash,
        ];
        app.combat.deck.draw_pile.clear();
        app.combat.deck.discard_pile.clear();
        app.resize(320.0, 568.0);
        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        let card_rect = app.layout().hand_rects[0];
        let body_entries = body_text_entries_in_rect(&frame, card_rect);

        assert!(
            body_entries
                .iter()
                .any(|(_, _, _, _, _, _, text)| text == "Ritmo")
        );
        assert!(
            body_entries
                .iter()
                .any(|(_, _, _, _, _, _, text)| text == "de")
        );
        assert!(
            body_entries
                .iter()
                .any(|(_, _, _, _, _, _, text)| text == "Escudo.")
        );
        assert!(body_entries.iter().all(|(x, _, size, _, _, _, text)| {
            *x + text_width(text, *size) <= card_rect.x + card_rect.w + 0.01
        }));
    }

    #[test]
    fn card_tiles_grow_taller_for_wrapped_copy_without_needing_more_width() {
        let card_w = 158.0;
        let short = CardDef {
            id: CardId::QuickStrike,
            name: "Pulse",
            cost: 1,
            target: CardTarget::Enemy,
            description: "Deal 6 damage.",
            archetype: CardArchetype::Pressure,
            reward_tier: Some(RewardTier::Combat),
            traits: CardTraits::default(),
        };
        let long = CardDef {
            id: CardId::QuickStrike,
            name: "Pulse Synchronization Cascade",
            cost: 1,
            target: CardTarget::Enemy,
            description: "Deal 6 damage. Gain 4 Shield. Draw 1 card. Apply Momentum -1 if the target has Bleed.",
            archetype: CardArchetype::Pressure,
            reward_tier: Some(RewardTier::Combat),
            traits: CardTraits::default(),
        };
        let metrics = card_box_metrics(card_w);
        let long_body_lines =
            wrap_text_by_width(long.description, metrics.body_size, metrics.body_max_width);

        assert!(wrap_text(long.name, metrics.title_chars).len() > 1 || long_body_lines.len() > 1);
        assert!(card_content_height(long, card_w) > card_content_height(short, card_w));
    }

    #[test]
    fn module_tiles_grow_taller_for_wrapped_copy_without_needing_more_width() {
        let card_w = 166.0;
        let short = ModuleDef {
            id: ModuleId::AegisDrive,
            name: "Pulse Drive",
            description: "Start each combat with 5 Shield.",
        };
        let long = ModuleDef {
            id: ModuleId::AegisDrive,
            name: "Reactive Synchronization Lattice",
            description: "Start each combat with 5 Shield. After each victory, recover 2 HP and gain 4 additional Credits.",
        };
        let metrics = module_box_metrics(card_w);

        assert!(
            wrap_text(long.name, metrics.title_chars).len() > 1
                || wrap_text(long.description, metrics.body_chars).len() > 1
        );
        assert!(module_content_height(long, card_w) > module_content_height(short, card_w));
    }

    #[test]
    fn module_select_cards_keep_copy_inside_card_bounds_on_mobile() {
        for language in [Language::English, Language::Spanish] {
            let mut app = active_module_select_fixture();
            app.set_language(language);
            app.resize(320.0, 568.0);

            let layout = app.module_select_layout().unwrap();
            let modules = app.module_select.as_ref().unwrap().options.clone();

            for (index, module) in modules.iter().copied().enumerate() {
                let def = app.localized_module_def(module);
                assert!(
                    module_tile_copy_fits_rect(def, layout.card_rects[index]),
                    "module copy should fit inside the card for {} in {:?}",
                    def.name,
                    language
                );
            }
        }
    }

    #[test]
    fn event_choice_tiles_grow_taller_for_wrapped_copy_without_needing_more_width() {
        let card_w = 170.0;
        let short_title = "Take the cache";
        let short_body = "Gain 16 Credits.";
        let long_title = "Route power through the unstable relay assembly";
        let long_body = "Lose 5 HP. Gain 30 Credits. Add a reinforced shell to your deck after the charge cycle completes.";
        let metrics = event_box_metrics(card_w);

        assert!(
            wrap_text(long_title, metrics.title_chars).len() > 1
                || wrap_text(long_body, metrics.body_chars).len() > 1
        );
        assert!(
            event_choice_content_height(long_title, long_body, card_w)
                > event_choice_content_height(short_title, short_body, card_w)
        );
    }

    #[test]
    fn map_save_round_trip_restores_the_run_exactly() {
        let mut app = App::new();
        start_run_to_map(&mut app);
        let snapshot = app.serialize_current_run().unwrap();

        let restored = restore_app_from_snapshot(&snapshot);

        assert!(matches!(restored.screen, AppScreen::Map));
        assert_eq!(restored.dungeon, app.dungeon);
        assert_eq!(restored.log, app.log);
    }

    #[test]
    fn map_save_round_trip_preserves_new_cards_in_the_deck() {
        let mut app = App::new();
        start_run_to_map(&mut app);
        let dungeon = app.dungeon.as_mut().unwrap();
        dungeon.deck.push(CardId::PinpointJab);
        dungeon.deck.push(CardId::FracturePulse);
        dungeon.deck.push(CardId::OverwatchGrid);

        let snapshot = app.serialize_current_run().unwrap();
        let restored = restore_app_from_snapshot(&snapshot);

        assert_eq!(
            restored.dungeon.as_ref().unwrap().deck,
            app.dungeon.as_ref().unwrap().deck
        );
    }

    #[test]
    fn map_save_round_trip_preserves_credits() {
        let mut app = App::new();
        start_run_to_map(&mut app);
        app.dungeon.as_mut().unwrap().credits = 27;

        let snapshot = app.serialize_current_run().unwrap();
        let restored = restore_app_from_snapshot(&snapshot);

        assert_eq!(restored.dungeon.as_ref().unwrap().credits, 27);
    }

    #[test]
    fn restore_old_map_save_without_credits_defaults_to_zero() {
        let mut app = App::new();
        start_run_to_map(&mut app);
        app.dungeon.as_mut().unwrap().credits = 27;
        let snapshot = app.serialize_current_run().unwrap();
        let mut value: serde_json::Value = serde_json::from_str(&snapshot).unwrap();
        value["active_state"]["dungeon"]
            .as_object_mut()
            .unwrap()
            .remove("credits");
        value["fallback_checkpoint"]["dungeon"]
            .as_object_mut()
            .unwrap()
            .remove("credits");

        let restored = restore_app_from_snapshot(&serde_json::to_string(&value).unwrap());

        assert!(matches!(restored.screen, AppScreen::Map));
        assert_eq!(restored.dungeon.as_ref().unwrap().credits, 0);
    }

    #[test]
    fn start_run_opens_opening_intro_before_module_select() {
        let mut app = App::new();

        app.start_run();

        assert!(matches!(app.screen, AppScreen::OpeningIntro));
        assert!(app.opening_intro.is_some());
        assert!(app.module_select.is_some());
        assert!(matches!(
            app.module_select.as_ref().unwrap().context,
            ModuleSelectContext::Starter
        ));
        assert!(app.dungeon.as_ref().unwrap().modules.is_empty());
        assert!(app.run_save_snapshot.is_none());
        assert!(!app.has_saved_run);
    }

    #[test]
    fn opening_intro_tick_reveals_text_then_completes() {
        let mut app = App::new();
        app.start_run();
        app.screen_transition = None;

        advance_time(&mut app, OPENING_INTRO_LINE_FADE_MS * 0.5);
        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        assert!(frame.contains("|body|You walk down a narrow hallway toward a door."));
        assert!(!frame.contains("|body|You enter through the door."));
        assert!(!app.opening_intro_complete());

        advance_until(&mut app, |app| app.opening_intro_complete());

        assert!(app.opening_intro_complete());
        assert!(app.opening_intro_button_transition_progress() > 0.0);
    }

    #[test]
    fn opening_intro_skip_reveals_all_lines_and_continue_enters_module_select() {
        let mut app = App::new();
        app.start_run();
        app.screen_transition = None;

        app.handle_opening_intro_action();

        assert!(matches!(app.screen, AppScreen::ModuleSelect));
        assert!(app.run_save_snapshot.is_some());
        assert!(app.has_saved_run);
    }

    #[test]
    fn opening_intro_action_button_animates_to_continue_label() {
        let mut app = App::new();
        app.start_run();
        app.screen_transition = None;

        let skip_button = app.opening_intro_action_button();
        app.complete_opening_intro();
        let transition_start_button = app.opening_intro_action_button();
        advance_time(&mut app, OPENING_INTRO_BUTTON_TRANSITION_MS * 0.5);
        let transition_mid_button = app.opening_intro_action_button();
        advance_time(&mut app, OPENING_INTRO_BUTTON_TRANSITION_MS * 0.5);
        let continue_button = app.opening_intro_action_button();

        assert_eq!(transition_start_button.rect.w, skip_button.rect.w);
        assert!(transition_mid_button.rect.w < skip_button.rect.w);
        assert!(transition_mid_button.rect.w > continue_button.rect.w);
        assert!(continue_button.rect.w < skip_button.rect.w);
        assert_eq!(
            continue_button.rect.x + continue_button.rect.w * 0.5,
            skip_button.rect.x + skip_button.rect.w * 0.5
        );
    }

    #[test]
    fn opening_intro_renders_localized_copy_with_regular_first_line() {
        let mut app = App::new();
        app.set_language(Language::Spanish);
        app.start_run();
        app.screen_transition = None;
        app.complete_opening_intro();
        advance_time(&mut app, OPENING_INTRO_BUTTON_TRANSITION_MS);
        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        assert!(frame.contains("|body|Avanzas por un pasillo estrecho hacia una puerta."));
        assert!(frame.contains("|body|Cruzas la puerta."));
        assert!(frame.contains("|body|Hay tres puertas delante."));
        assert!(frame.contains("|label|Continuar"));
    }

    #[test]
    fn opening_intro_does_not_serialize_until_module_select() {
        let mut app = App::new();
        app.start_run();

        assert!(app.serialize_current_run().is_none());
        assert!(app.run_save_snapshot.is_none());

        skip_opening_intro(&mut app);

        assert!(matches!(app.screen, AppScreen::ModuleSelect));
        assert!(app.serialize_current_run().is_some());
        assert!(app.run_save_snapshot.is_some());
    }

    #[test]
    fn module_select_claim_adds_the_module_and_enters_the_map() {
        let mut app = active_module_select_fixture();
        let expected = app.module_select.as_ref().unwrap().options[1];

        app.claim_module_select(1);

        assert!(matches!(app.screen, AppScreen::Map));
        assert_eq!(app.dungeon.as_ref().unwrap().modules, vec![expected]);
        let expected_log = format!("Selected {}.", module_def(expected).name);
        assert_eq!(
            app.log.front().map(String::as_str),
            Some(expected_log.as_str())
        );
    }

    #[test]
    fn module_select_save_round_trip_keeps_options_and_seed() {
        let app = active_module_select_fixture();
        let snapshot = app.serialize_current_run().unwrap();

        let restored = restore_app_from_snapshot(&snapshot);
        let module_select = restored.module_select.as_ref().unwrap();

        assert!(matches!(restored.screen, AppScreen::ModuleSelect));
        assert_eq!(
            module_select.options,
            app.module_select.as_ref().unwrap().options
        );
        assert_eq!(module_select.seed, app.module_select.as_ref().unwrap().seed);
        assert_eq!(module_select.context, ModuleSelectContext::Starter);
    }

    #[test]
    fn module_select_fallback_recomputes_options() {
        let app = active_module_select_fixture();
        let mut envelope = parse_run_save(&app.serialize_current_run().unwrap()).unwrap();
        if let SavedRunState::ModuleSelect { module_select, .. } = &mut envelope.active_state {
            module_select.options[0] = "removed_module".to_string();
        } else {
            panic!("expected module select save");
        }

        let restored = restore_app_from_snapshot(&serialize_envelope(&envelope).unwrap());

        assert!(matches!(restored.screen, AppScreen::ModuleSelect));
        assert_eq!(
            restored.module_select.as_ref().unwrap().options,
            starter_module_choices()
        );
    }

    #[test]
    fn old_module_select_saves_default_to_starter_context() {
        let app = active_module_select_fixture();
        let mut value: serde_json::Value =
            serde_json::from_str(&app.serialize_current_run().unwrap()).unwrap();
        let module_select = value
            .get_mut("active_state")
            .and_then(|state| state.get_mut("module_select"))
            .and_then(serde_json::Value::as_object_mut)
            .unwrap();
        module_select.remove("kind");
        module_select.remove("boss_level");

        let restored = restore_app_from_snapshot(&serde_json::to_string(&value).unwrap());

        assert!(matches!(restored.screen, AppScreen::ModuleSelect));
        assert_eq!(
            restored.module_select.as_ref().unwrap().context,
            ModuleSelectContext::Starter
        );
    }

    #[test]
    fn restore_old_map_save_without_modules_defaults_to_aegis_drive() {
        let mut app = App::new();
        start_run_to_map(&mut app);
        let snapshot = app.serialize_current_run().unwrap();
        let mut value: serde_json::Value = serde_json::from_str(&snapshot).unwrap();
        value["active_state"]["dungeon"]
            .as_object_mut()
            .unwrap()
            .remove("modules");
        value["fallback_checkpoint"]["dungeon"]
            .as_object_mut()
            .unwrap()
            .remove("modules");

        let restored = restore_app_from_snapshot(&serde_json::to_string(&value).unwrap());

        assert!(matches!(restored.screen, AppScreen::Map));
        assert_eq!(
            restored.dungeon.as_ref().unwrap().modules,
            vec![ModuleId::AegisDrive]
        );
    }

    #[test]
    fn shop_save_round_trip_keeps_shop_offers_and_seed() {
        let mut app = App::new();
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.credits = 27;
        dungeon.current_node = Some(1);
        app.dungeon = Some(dungeon);
        app.screen = AppScreen::Shop;
        app.shop = Some(ShopState {
            offers: vec![
                ShopOffer {
                    card: CardId::QuickStrike,
                    price: 16,
                },
                ShopOffer {
                    card: CardId::BarrierField,
                    price: 24,
                },
            ],
            seed: TEST_AUXILIARY_SEED,
        });
        app.push_log("Shopping.");
        let snapshot = app.serialize_current_run().unwrap();

        let restored = restore_app_from_snapshot(&snapshot);
        let shop = restored.shop.as_ref().unwrap();

        assert!(matches!(restored.screen, AppScreen::Shop));
        assert_eq!(shop.offers[0].card, CardId::QuickStrike);
        assert_eq!(shop.offers[0].price, 16);
        assert_eq!(shop.offers[1].card, CardId::BarrierField);
        assert_eq!(shop.offers[1].price, 24);
        assert_eq!(shop.seed, TEST_AUXILIARY_SEED);
        assert_eq!(restored.log, app.log);
    }

    #[test]
    fn shop_save_round_trip_preserves_expanded_card_ids_across_tiers() {
        let mut app = App::new();
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.current_level = 3;
        dungeon.credits = 55;
        dungeon.current_node = Some(1);
        dungeon.deck = vec![
            CardId::RiftDartPlus,
            CardId::SeverArcPlus,
            CardId::TerminalLoopPlus,
        ];
        app.dungeon = Some(dungeon);
        app.screen = AppScreen::Shop;
        app.shop = Some(ShopState {
            offers: vec![
                ShopOffer {
                    card: CardId::RiftDartPlus,
                    price: 16,
                },
                ShopOffer {
                    card: CardId::SeverArcPlus,
                    price: 24,
                },
                ShopOffer {
                    card: CardId::TerminalLoopPlus,
                    price: 40,
                },
            ],
            seed: TEST_AUXILIARY_SEED,
        });

        let snapshot = app.serialize_current_run().unwrap();
        let restored = restore_app_from_snapshot(&snapshot);

        assert!(matches!(restored.screen, AppScreen::Shop));
        assert_eq!(
            restored.dungeon.as_ref().unwrap().deck,
            vec![
                CardId::RiftDartPlus,
                CardId::SeverArcPlus,
                CardId::TerminalLoopPlus,
            ]
        );
        assert_eq!(
            restored.shop.as_ref().unwrap().offers,
            vec![
                ShopOffer {
                    card: CardId::RiftDartPlus,
                    price: 16,
                },
                ShopOffer {
                    card: CardId::SeverArcPlus,
                    price: 24,
                },
                ShopOffer {
                    card: CardId::TerminalLoopPlus,
                    price: 40,
                },
            ]
        );
    }

    #[test]
    fn shop_fallback_recomputes_offers_for_the_saved_level() {
        let mut app = App::new();
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.current_level = 2;
        dungeon.credits = 27;
        dungeon.current_node = Some(1);
        app.dungeon = Some(dungeon);
        app.screen = AppScreen::Shop;
        app.shop = Some(ShopState {
            offers: vec![
                ShopOffer {
                    card: CardId::BarrierField,
                    price: 24,
                },
                ShopOffer {
                    card: CardId::ExecutionBeam,
                    price: 40,
                },
            ],
            seed: TEST_FALLBACK_SEED,
        });
        let mut envelope = parse_run_save(&app.serialize_current_run().unwrap()).unwrap();
        if let SavedRunState::Shop { shop, .. } = &mut envelope.active_state {
            shop.offers[0].card = "removed_card".to_string();
        } else {
            panic!("expected shop save");
        }

        let restored = restore_app_from_snapshot(&serialize_envelope(&envelope).unwrap());
        let shop = restored.shop.as_ref().unwrap();

        assert!(matches!(restored.screen, AppScreen::Shop));
        assert_eq!(shop.offers, shop_offers(TEST_FALLBACK_SEED, 2));
    }

    #[test]
    fn reward_save_round_trip_keeps_reward_options_and_seed() {
        let mut app = App::new();
        let dungeon = DungeonRun::new(TEST_RUN_SEED);
        app.dungeon = Some(dungeon);
        app.screen = AppScreen::Reward;
        app.reward = Some(RewardState {
            tier: RewardTier::Elite,
            options: vec![CardId::QuickStrike, CardId::BarrierField],
            followup: RewardFollowup {
                completed_run: false,
            },
            seed: TEST_AUXILIARY_SEED,
        });
        app.push_log("Reward pending.");
        let snapshot = app.serialize_current_run().unwrap();

        let restored = restore_app_from_snapshot(&snapshot);
        let reward = restored.reward.as_ref().unwrap();

        assert!(matches!(restored.screen, AppScreen::Reward));
        assert_eq!(reward.tier, RewardTier::Elite);
        assert_eq!(
            reward.options,
            vec![CardId::QuickStrike, CardId::BarrierField]
        );
        assert_eq!(reward.seed, TEST_AUXILIARY_SEED);
        assert_eq!(restored.log, app.log);
    }

    #[test]
    fn reward_fallback_recomputes_elite_options_for_the_saved_level() {
        let mut app = App::new();
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.current_level = 2;
        app.dungeon = Some(dungeon);
        app.screen = AppScreen::Reward;
        app.reward = Some(RewardState {
            tier: RewardTier::Elite,
            options: vec![CardId::BarrierField, CardId::BreachSignal],
            followup: RewardFollowup {
                completed_run: false,
            },
            seed: TEST_FALLBACK_SEED,
        });
        let mut envelope = parse_run_save(&app.serialize_current_run().unwrap()).unwrap();
        if let SavedRunState::Reward { reward, .. } = &mut envelope.active_state {
            reward.options[0] = "removed_card".to_string();
        } else {
            panic!("expected reward save");
        }

        let restored = restore_app_from_snapshot(&serialize_envelope(&envelope).unwrap());
        let reward = restored.reward.as_ref().unwrap();

        assert!(matches!(restored.screen, AppScreen::Reward));
        assert_eq!(
            reward.options,
            reward_choices(TEST_FALLBACK_SEED, RewardTier::Elite, 2)
        );
    }

    #[test]
    fn shop_screen_renders_leave_button_and_credit_labels() {
        let mut app = active_shop_fixture();

        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        assert!(frame.contains("|label|Leave"));
        assert!(frame.contains("|label|You have 32 Credits"));
        assert!(frame.contains("|label|16 Credits"));
    }

    #[test]
    fn non_combat_primary_and_flow_buttons_use_tinted_tiles() {
        let mut boot = App::new();
        boot.rebuild_frame();
        let boot_layout = boot.boot_buttons_layout(false);
        let boot_frame = String::from_utf8(boot.frame.clone()).unwrap();
        let boot_rects = frame_rect_entries(&boot_frame);
        assert_ui_tile_entry(
            find_frame_rect_entry(&boot_rects, boot_layout.start_button),
            COLOR_GREEN_STROKE_START,
            1.0,
        );

        let mut rest = active_rest_fixture(dense_rest_test_deck(), 24);
        rest.rebuild_frame();
        let rest_layout = rest.rest_layout().unwrap();
        let rest_frame = String::from_utf8(rest.frame.clone()).unwrap();
        let rest_rects = frame_rect_entries(&rest_frame);
        assert_ui_tile_entry(
            find_frame_rect_entry(&rest_rects, rest_layout.heal_rect),
            COLOR_CYAN_STROKE_IDLE,
            1.0,
        );

        let mut shop = active_shop_fixture();
        shop.rebuild_frame();
        let shop_layout = shop.shop_layout().unwrap();
        let shop_frame = String::from_utf8(shop.frame.clone()).unwrap();
        let shop_rects = frame_rect_entries(&shop_frame);
        assert_ui_tile_entry(
            find_frame_rect_entry(&shop_rects, shop_layout.leave_button),
            COLOR_GREEN_STROKE_START,
            1.0,
        );

        let mut reward = App::new();
        reward.dungeon = Some(DungeonRun::new(TEST_RUN_SEED));
        reward.screen = AppScreen::Reward;
        reward.reward = Some(RewardState {
            tier: RewardTier::Combat,
            options: vec![CardId::QuickStrike],
            followup: RewardFollowup {
                completed_run: false,
            },
            seed: TEST_RUN_SEED,
        });
        reward.rebuild_frame();
        let reward_layout = reward.reward_layout().unwrap();
        let reward_frame = String::from_utf8(reward.frame.clone()).unwrap();
        let reward_rects = frame_rect_entries(&reward_frame);
        assert_ui_tile_entry(
            find_frame_rect_entry(&reward_rects, reward_layout.skip_button),
            COLOR_GREEN_STROKE_START,
            1.0,
        );
    }

    #[test]
    fn non_combat_selection_tiles_use_tinted_fill() {
        let mut module_select = active_module_select_fixture();
        module_select.rebuild_frame();
        let module_layout = module_select.module_select_layout().unwrap();
        let module = module_select.module_select.as_ref().unwrap().options[0];
        let module_frame = String::from_utf8(module_select.frame.clone()).unwrap();
        let module_rects = frame_rect_entries(&module_frame);
        assert_ui_tile_entry(
            find_frame_rect_entry(&module_rects, module_layout.card_rects[0]),
            &module_stroke(module),
            1.0,
        );

        let mut event = active_event_fixture(EventId::SalvageCache);
        event.rebuild_frame();
        let event_layout = event.event_layout().unwrap();
        let event_frame = String::from_utf8(event.frame.clone()).unwrap();
        let event_rects = frame_rect_entries(&event_frame);
        assert_ui_tile_entry(
            find_frame_rect_entry(&event_rects, event_layout.choice_rects[0]),
            COLOR_BLUE_STROKE_IDLE,
            1.0,
        );

        let mut shop = active_shop_fixture();
        shop.shop.as_mut().unwrap().offers = vec![ShopOffer {
            card: CardId::QuickStrike,
            price: 16,
        }];
        shop.rebuild_frame();
        let shop_layout = shop.shop_layout().unwrap();
        let shop_frame = String::from_utf8(shop.frame.clone()).unwrap();
        let shop_rects = frame_rect_entries(&shop_frame);
        assert_ui_tile_entry(
            find_frame_rect_entry(&shop_rects, shop_layout.card_rects[0]),
            COLOR_CYAN_STROKE_IDLE,
            1.0,
        );

        let mut reward = App::new();
        reward.dungeon = Some(DungeonRun::new(TEST_RUN_SEED));
        reward.screen = AppScreen::Reward;
        reward.reward = Some(RewardState {
            tier: RewardTier::Elite,
            options: vec![CardId::QuickStrike],
            followup: RewardFollowup {
                completed_run: false,
            },
            seed: TEST_RUN_SEED,
        });
        reward.rebuild_frame();
        let reward_layout = reward.reward_layout().unwrap();
        let reward_frame = String::from_utf8(reward.frame.clone()).unwrap();
        let reward_rects = frame_rect_entries(&reward_frame);
        assert_ui_tile_entry(
            find_frame_rect_entry(&reward_rects, reward_layout.card_rects[0]),
            reward_tier_stroke(RewardTier::Elite),
            1.0,
        );
    }

    #[test]
    fn map_buttons_and_run_info_panel_use_tinted_tiles() {
        let mut app = App::new();
        start_run_to_map(&mut app);
        app.open_run_info();
        app.ui.run_info_progress = 1.0;

        app.rebuild_frame();

        let map_layout = app.map_layout().unwrap();
        let run_info_layout = app.run_info_layout().unwrap();
        let frame = String::from_utf8(app.frame.clone()).unwrap();
        let rects = frame_rect_entries(&frame);
        assert_ui_tile_entry(
            find_frame_rect_entry(&rects, map_layout.menu_button),
            COLOR_GREEN_STROKE_IDLE,
            1.0,
        );
        assert_ui_tile_entry(
            find_frame_rect_entry(&rects, map_layout.info_button),
            COLOR_GREEN_STROKE_STRONG,
            1.0,
        );
        assert_ui_tile_entry(
            find_frame_rect_entry(&rects, run_info_layout.modal_rect),
            COLOR_GREEN_STROKE_PANEL,
            1.0,
        );
    }

    #[test]
    fn shop_layout_keeps_prices_close_to_cards_and_credits_farther_below() {
        let app = active_shop_fixture();
        let layout = app.shop_layout().unwrap();
        let price_size =
            fit_text_size("40 Credits", 18.0, (app.logical_width() - 48.0).max(120.0)).max(12.0);
        let credits_size = fit_text_size(
            "You have 99 Credits",
            18.0,
            (app.logical_width() - 48.0).max(120.0),
        )
        .max(12.0);
        let cards_bottom = layout
            .card_rects
            .iter()
            .map(|rect| rect.y + rect.h)
            .fold(0.0, f32::max);
        let first_price_top_gap =
            layout.price_ys[0] - price_size - (layout.card_rects[0].y + layout.card_rects[0].h);
        let credits_top_gap =
            layout.credits_y - credits_size - layout.price_ys.iter().copied().fold(0.0, f32::max);

        assert!(first_price_top_gap <= 8.0);
        assert!(credits_top_gap >= 10.0);
        assert!(layout.credits_y > cards_bottom);
    }

    #[test]
    fn shop_layout_on_mobile_keeps_cards_and_labels_above_leave_button() {
        let mut app = active_shop_fixture();
        app.resize(320.0, 568.0);

        let layout = app.shop_layout().unwrap();
        let cards_bottom = layout
            .card_rects
            .iter()
            .map(|rect| rect.y + rect.h)
            .fold(0.0, f32::max);
        let prices_bottom = layout.price_ys.iter().copied().fold(0.0, f32::max);

        assert!(cards_bottom < layout.leave_button.y);
        assert!(prices_bottom < layout.leave_button.y);
        assert!(layout.credits_y < layout.leave_button.y);
    }

    #[test]
    fn shop_leave_button_hit_test_wins_at_its_center() {
        let app = active_shop_fixture();
        let layout = app.shop_layout().unwrap();

        assert_eq!(
            app.hit_test(
                layout.leave_button.x + layout.leave_button.w * 0.5,
                layout.leave_button.y + layout.leave_button.h * 0.5,
            ),
            Some(HitTarget::ShopLeave)
        );
    }

    #[test]
    fn rest_layout_paginates_on_mobile_without_overlapping_controls() {
        let mut app = active_rest_fixture(dense_rest_test_deck(), 24);
        app.resize(320.0, 568.0);

        let layout = app.rest_layout().unwrap();
        let pagination_top = layout
            .prev_button
            .unwrap()
            .rect
            .y
            .min(layout.next_button.unwrap().rect.y);

        assert_eq!(layout.page_count, 3);
        assert_eq!(layout.current_page, 0);
        assert_eq!(layout.card_rects.len(), 4);
        assert_eq!(layout.visible_upgrade_indices, vec![0, 1, 2, 3]);
        assert_eq!(layout.page_status_label.as_deref(), Some("1/3"));
        for rect in &layout.card_rects {
            assert!(rect.y + rect.h <= pagination_top + 0.01);
            assert!(rect.y + rect.h <= layout.confirm_rect.y - 0.01);
        }
    }

    #[test]
    fn rest_layout_limits_columns_when_mid_width_would_overflow() {
        let mut app = active_rest_fixture(dense_rest_test_deck(), 24);
        app.resize(600.0, 720.0);

        let layout = app.rest_layout().unwrap();

        assert_eq!(layout.page_count, 2);
        assert_eq!(layout.card_rects.len(), 9);
        for rect in &layout.card_rects {
            assert!(rect.x >= 0.0);
            assert!(rect.x + rect.w <= app.logical_width() + 0.01);
        }
    }

    #[test]
    fn rest_pagination_renders_as_compact_inline_group() {
        let mut app = active_rest_fixture(dense_rest_test_deck(), 24);
        app.resize(320.0, 568.0);

        app.rebuild_frame();
        let frame = String::from_utf8(app.frame.clone()).unwrap();
        let layout = app.rest_layout().unwrap();
        let prev_button = layout.prev_button.unwrap();
        let next_button = layout.next_button.unwrap();
        let page_status_x = layout.page_status_x.unwrap();
        let page_status_size = layout.page_status_size.unwrap();
        let page_status_w = text_width(
            layout.page_status_label.as_deref().unwrap(),
            page_status_size,
        );
        let status_left = page_status_x - page_status_w * 0.5;
        let status_right = page_status_x + page_status_w * 0.5;

        assert!(frame.contains("|label|<"));
        assert!(frame.contains("|body|1/3"));
        assert!(frame.contains("|label|>"));
        assert!(!frame.contains("|label|Previous"));
        assert!(!frame.contains("|label|Next"));
        assert!(!frame.contains("|body|Page 1/3"));
        assert!((status_left - (prev_button.rect.x + prev_button.rect.w)) >= 10.0);
        assert!((next_button.rect.x - status_right) >= 10.0);
        assert!(next_button.rect.x - prev_button.rect.x < app.logical_width() * 0.35);
    }

    #[test]
    fn rest_confirm_hit_test_wins_over_cards_in_dense_layout() {
        let mut app = active_rest_fixture(dense_rest_test_deck(), 24);
        app.resize(320.0, 568.0);

        let layout = app.rest_layout().unwrap();
        app.select_rest_option(RestSelection::Upgrade(layout.visible_upgrade_indices[0]));
        let layout = app.rest_layout().unwrap();

        assert_eq!(
            app.hit_test(
                layout.confirm_rect.x + layout.confirm_rect.w * 0.5,
                layout.confirm_rect.y + layout.confirm_rect.h * 0.5,
            ),
            Some(HitTarget::RestConfirm)
        );
    }

    #[test]
    fn rest_pagination_keyboard_navigation_updates_visible_selection() {
        let mut app = active_rest_fixture(dense_rest_test_deck(), 32);
        app.resize(320.0, 568.0);

        app.key_down(39);
        assert_eq!(app.ui.rest_page, 1);

        let layout = app.rest_layout().unwrap();
        assert_eq!(layout.visible_upgrade_indices, vec![4, 5, 6, 7]);
        let expected_selection = layout.visible_upgrade_indices[0];

        app.key_down(49);
        assert_eq!(
            app.ui.rest_selection,
            Some(RestSelection::Upgrade(expected_selection))
        );

        app.key_down(37);
        assert_eq!(app.ui.rest_page, 0);
        assert_eq!(app.ui.rest_selection, None);
    }

    #[test]
    fn visited_nodes_keep_room_type_accents_on_map() {
        let mut app = App::new();
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.nodes = vec![
            crate::dungeon::DungeonNode {
                id: 0,
                depth: 0,
                lane: 3,
                kind: RoomKind::Start,
                next: vec![1, 2, 3],
            },
            crate::dungeon::DungeonNode {
                id: 1,
                depth: 1,
                lane: 2,
                kind: RoomKind::Elite,
                next: vec![],
            },
            crate::dungeon::DungeonNode {
                id: 2,
                depth: 1,
                lane: 3,
                kind: RoomKind::Rest,
                next: vec![],
            },
            crate::dungeon::DungeonNode {
                id: 3,
                depth: 1,
                lane: 4,
                kind: RoomKind::Shop,
                next: vec![],
            },
        ];
        dungeon.available_nodes.clear();
        dungeon.visited_nodes = vec![0, 1, 2, 3];
        app.dungeon = Some(dungeon);
        app.screen = AppScreen::Map;

        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        assert!(frame.contains(&room_visited_stroke(RoomKind::Elite)));
        assert!(frame.contains(&room_visited_text_color(RoomKind::Elite)));
        assert!(frame.contains(&room_visited_stroke(RoomKind::Rest)));
        assert!(frame.contains(&room_visited_text_color(RoomKind::Rest)));
        assert!(frame.contains(&room_visited_stroke(RoomKind::Shop)));
        assert!(frame.contains(&room_visited_text_color(RoomKind::Shop)));
    }

    #[test]
    fn shop_selection_from_map_opens_the_shop_screen() {
        let mut app = App::new();
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.credits = 24;
        dungeon.nodes = vec![
            crate::dungeon::DungeonNode {
                id: 0,
                depth: 0,
                lane: 3,
                kind: RoomKind::Start,
                next: vec![1],
            },
            crate::dungeon::DungeonNode {
                id: 1,
                depth: 1,
                lane: 3,
                kind: RoomKind::Shop,
                next: vec![],
            },
        ];
        dungeon.available_nodes = vec![1];
        app.dungeon = Some(dungeon);
        app.screen = AppScreen::Map;

        app.select_map_node(1);

        assert!(matches!(app.screen, AppScreen::Shop));
        assert!(app.shop.is_some());
    }

    #[test]
    fn selecting_an_event_node_from_the_map_opens_the_event_screen() {
        let mut app = App::new();
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.nodes = vec![
            crate::dungeon::DungeonNode {
                id: 0,
                depth: 0,
                lane: 3,
                kind: RoomKind::Start,
                next: vec![1],
            },
            crate::dungeon::DungeonNode {
                id: 1,
                depth: 1,
                lane: 3,
                kind: RoomKind::Event,
                next: vec![2],
            },
            crate::dungeon::DungeonNode {
                id: 2,
                depth: 2,
                lane: 3,
                kind: RoomKind::Combat,
                next: vec![],
            },
        ];
        dungeon.current_node = Some(0);
        dungeon.available_nodes = vec![1];
        dungeon.visited_nodes = vec![0];
        app.dungeon = Some(dungeon);
        app.screen = AppScreen::Map;

        app.select_map_node(1);

        assert!(matches!(app.screen, AppScreen::Event));
        assert!(app.event.is_some());
    }

    #[test]
    fn event_choice_hotkey_applies_effect_and_returns_to_map() {
        let mut app = active_event_fixture(EventId::SalvageCache);
        let initial_credits = app.dungeon.as_ref().unwrap().credits;

        app.key_down(49);

        assert!(matches!(app.screen, AppScreen::Map));
        assert_eq!(app.dungeon.as_ref().unwrap().credits, initial_credits + 16);
        assert_eq!(app.dungeon.as_ref().unwrap().player_hp, 20);
        assert!(app.event.is_none());
        assert_eq!(
            app.log.back().map(String::as_str),
            Some("Recovered 16 Credits from Salvage Cache.")
        );
    }

    #[test]
    fn event_save_round_trip_keeps_event_state() {
        let mut app = active_event_fixture(EventId::ClinicPod);
        app.push_log("Event pending.");
        let snapshot = app.serialize_current_run().unwrap();

        let restored = restore_app_from_snapshot(&snapshot);

        assert!(matches!(restored.screen, AppScreen::Event));
        assert_eq!(
            restored.event,
            Some(EventState {
                event: EventId::ClinicPod
            })
        );
        assert_eq!(restored.log, app.log);
    }

    #[test]
    fn shop_purchase_spends_credits_adds_card_and_returns_to_map() {
        let mut app = active_shop_fixture();
        app.dungeon.as_mut().unwrap().available_nodes = vec![2];
        app.dungeon.as_mut().unwrap().nodes[1].next = vec![2];
        app.dungeon
            .as_mut()
            .unwrap()
            .nodes
            .push(crate::dungeon::DungeonNode {
                id: 2,
                depth: 2,
                lane: 3,
                kind: RoomKind::Combat,
                next: vec![],
            });
        app.shop.as_mut().unwrap().offers = vec![
            ShopOffer {
                card: CardId::QuickStrike,
                price: 16,
            },
            ShopOffer {
                card: CardId::BarrierField,
                price: 24,
            },
            ShopOffer {
                card: CardId::ExecutionBeam,
                price: 40,
            },
        ];
        let initial_deck_len = app.dungeon.as_ref().unwrap().deck.len();

        app.claim_shop_offer(0);

        assert!(matches!(app.screen, AppScreen::Map));
        assert!(app.shop.is_none());
        assert_eq!(app.dungeon.as_ref().unwrap().credits, 16);
        assert_eq!(
            app.dungeon.as_ref().unwrap().deck.len(),
            initial_deck_len + 1
        );
        assert_eq!(
            app.dungeon.as_ref().unwrap().deck.last(),
            Some(&CardId::QuickStrike)
        );
        assert_eq!(
            app.log.back().map(String::as_str),
            Some("Bought Quick Strike for 16 Credits.")
        );
    }

    #[test]
    fn shop_does_not_buy_unaffordable_cards() {
        let mut app = active_shop_fixture();
        app.dungeon.as_mut().unwrap().credits = 8;
        app.shop.as_mut().unwrap().offers = vec![ShopOffer {
            card: CardId::BarrierField,
            price: 24,
        }];
        let initial_deck = app.dungeon.as_ref().unwrap().deck.clone();

        app.claim_shop_offer(0);

        assert!(matches!(app.screen, AppScreen::Shop));
        assert_eq!(app.dungeon.as_ref().unwrap().credits, 8);
        assert_eq!(app.dungeon.as_ref().unwrap().deck, initial_deck);
    }

    #[test]
    fn shop_leave_hotkeys_exit_without_spending_credits() {
        let mut app = active_shop_fixture();
        app.dungeon.as_mut().unwrap().available_nodes = vec![2];
        app.dungeon.as_mut().unwrap().nodes[1].next = vec![2];
        app.dungeon
            .as_mut()
            .unwrap()
            .nodes
            .push(crate::dungeon::DungeonNode {
                id: 2,
                depth: 2,
                lane: 3,
                kind: RoomKind::Combat,
                next: vec![],
            });
        let initial_deck = app.dungeon.as_ref().unwrap().deck.clone();

        app.key_down(48);

        assert!(matches!(app.screen, AppScreen::Map));
        assert_eq!(app.dungeon.as_ref().unwrap().credits, 32);
        assert_eq!(app.dungeon.as_ref().unwrap().deck, initial_deck);
        assert_eq!(app.log.back().map(String::as_str), Some("Left shop."));
    }

    #[test]
    fn shop_hotkeys_buy_affordable_slots_and_escape_leaves() {
        let mut app = active_shop_fixture();
        app.dungeon.as_mut().unwrap().available_nodes = vec![2];
        app.dungeon.as_mut().unwrap().nodes[1].next = vec![2];
        app.dungeon
            .as_mut()
            .unwrap()
            .nodes
            .push(crate::dungeon::DungeonNode {
                id: 2,
                depth: 2,
                lane: 3,
                kind: RoomKind::Combat,
                next: vec![],
            });
        app.shop.as_mut().unwrap().offers = vec![
            ShopOffer {
                card: CardId::BarrierField,
                price: 24,
            },
            ShopOffer {
                card: CardId::ExecutionBeam,
                price: 40,
            },
        ];

        app.key_down(49);

        assert!(matches!(app.screen, AppScreen::Map));
        assert_eq!(app.dungeon.as_ref().unwrap().credits, 8);

        let mut app = active_shop_fixture();
        app.dungeon.as_mut().unwrap().available_nodes = vec![2];
        app.dungeon.as_mut().unwrap().nodes[1].next = vec![2];
        app.dungeon
            .as_mut()
            .unwrap()
            .nodes
            .push(crate::dungeon::DungeonNode {
                id: 2,
                depth: 2,
                lane: 3,
                kind: RoomKind::Combat,
                next: vec![],
            });
        app.key_down(27);
        assert!(matches!(app.screen, AppScreen::Map));
        assert_eq!(app.log.back().map(String::as_str), Some("Left shop."));
    }

    #[test]
    fn reward_screen_renders_skip_button() {
        let mut app = App::new();
        app.dungeon = Some(DungeonRun::new(TEST_RUN_SEED));
        app.screen = AppScreen::Reward;
        app.reward = Some(RewardState {
            tier: RewardTier::Combat,
            options: vec![CardId::QuickStrike],
            followup: RewardFollowup {
                completed_run: false,
            },
            seed: TEST_RUN_SEED,
        });

        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        assert!(frame.contains("|label|Skip"));
        assert!(frame.contains("|label|+6 Credits"));
    }

    #[test]
    fn map_screen_does_not_render_credits_badge() {
        let mut app = App::new();
        start_run_to_map(&mut app);
        app.dungeon.as_mut().unwrap().credits = 27;
        app.screen_transition = None;

        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        assert!(!frame.contains("|label|27 Credits"));
    }

    #[test]
    fn map_edges_use_white_for_ready_paths_and_gray_for_locked_paths() {
        let mut app = App::new();
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.nodes = vec![
            crate::dungeon::DungeonNode {
                id: 0,
                depth: 0,
                lane: 3,
                kind: RoomKind::Start,
                next: vec![1, 2],
            },
            crate::dungeon::DungeonNode {
                id: 1,
                depth: 1,
                lane: 2,
                kind: RoomKind::Combat,
                next: vec![],
            },
            crate::dungeon::DungeonNode {
                id: 2,
                depth: 1,
                lane: 4,
                kind: RoomKind::Elite,
                next: vec![],
            },
        ];
        dungeon.available_nodes = vec![1];
        dungeon.visited_nodes = vec![0];
        app.dungeon = Some(dungeon);
        app.screen = AppScreen::Map;

        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        assert!(frame.contains(COLOR_WHITE_STROKE_PATH));
        assert!(frame.contains(COLOR_GRAY_STROKE_DISABLED));
    }

    #[test]
    fn map_layout_caps_adjacent_lane_spacing_on_wide_viewports() {
        let mut app = App::new();
        app.resize(2200.0, 720.0);
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.nodes = vec![
            crate::dungeon::DungeonNode {
                id: 0,
                depth: 0,
                lane: 3,
                kind: RoomKind::Start,
                next: vec![1, 2],
            },
            crate::dungeon::DungeonNode {
                id: 1,
                depth: 1,
                lane: 2,
                kind: RoomKind::Combat,
                next: vec![],
            },
            crate::dungeon::DungeonNode {
                id: 2,
                depth: 1,
                lane: 3,
                kind: RoomKind::Elite,
                next: vec![],
            },
        ];
        app.dungeon = Some(dungeon);
        app.screen = AppScreen::Map;

        let layout = app.map_layout().unwrap();
        let center_gap = (map_node_center_x(&layout, 2) - map_node_center_x(&layout, 1)).abs();
        let edge_gap = center_gap - MAP_NODE_DIAMETER;

        assert!(center_gap <= MAP_MAX_ADJACENT_LANE_CENTER_SPACING + 0.01);
        assert!(edge_gap <= MAP_NODE_DIAMETER + 0.01);
    }

    #[test]
    fn map_layout_preserves_empty_lanes_while_capping_wide_spacing() {
        let mut app = App::new();
        app.resize(2200.0, 720.0);
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.nodes = vec![
            crate::dungeon::DungeonNode {
                id: 0,
                depth: 0,
                lane: 3,
                kind: RoomKind::Start,
                next: vec![1, 2],
            },
            crate::dungeon::DungeonNode {
                id: 1,
                depth: 1,
                lane: 2,
                kind: RoomKind::Combat,
                next: vec![],
            },
            crate::dungeon::DungeonNode {
                id: 2,
                depth: 1,
                lane: 4,
                kind: RoomKind::Elite,
                next: vec![],
            },
        ];
        app.dungeon = Some(dungeon);
        app.screen = AppScreen::Map;

        let layout = app.map_layout().unwrap();
        let center_gap = (map_node_center_x(&layout, 2) - map_node_center_x(&layout, 1)).abs();

        assert!(
            (center_gap - MAP_MAX_ADJACENT_LANE_CENTER_SPACING * 2.0).abs() <= 0.01,
            "expected one preserved empty lane between visible branches"
        );
    }

    #[test]
    fn map_layout_keeps_narrow_viewport_lane_spacing_below_the_cap() {
        let mut app = App::new();
        app.resize(320.0, 568.0);
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.nodes = vec![
            crate::dungeon::DungeonNode {
                id: 0,
                depth: 0,
                lane: 3,
                kind: RoomKind::Start,
                next: vec![1, 2, 3],
            },
            crate::dungeon::DungeonNode {
                id: 1,
                depth: 1,
                lane: 2,
                kind: RoomKind::Combat,
                next: vec![],
            },
            crate::dungeon::DungeonNode {
                id: 2,
                depth: 1,
                lane: 3,
                kind: RoomKind::Rest,
                next: vec![],
            },
            crate::dungeon::DungeonNode {
                id: 3,
                depth: 1,
                lane: 6,
                kind: RoomKind::Shop,
                next: vec![],
            },
        ];
        app.dungeon = Some(dungeon);
        app.screen = AppScreen::Map;

        let layout = app.map_layout().unwrap();
        let center_gap = (map_node_center_x(&layout, 2) - map_node_center_x(&layout, 1)).abs();
        let side_pad = (app.logical_width() * 0.12).clamp(54.0, 132.0);
        let expected_spacing = (app.logical_width() - side_pad * 2.0).max(0.0) / 6.0;

        assert!(center_gap < MAP_MAX_ADJACENT_LANE_CENTER_SPACING);
        assert!((center_gap - expected_spacing).abs() <= 0.01);
    }

    #[test]
    fn map_legend_includes_the_shop_entry() {
        let mut app = App::new();
        start_run_to_map(&mut app);
        app.screen_transition = None;
        app.ui.legend_open = true;
        app.ui.legend_progress = 1.0;

        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        assert!(frame.contains("|body|Shop"));
    }

    #[test]
    fn map_legend_includes_the_event_entry() {
        let mut app = App::new();
        start_run_to_map(&mut app);
        app.screen_transition = None;
        app.ui.legend_open = true;
        app.ui.legend_progress = 1.0;

        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        assert!(frame.contains("|body|Event"));
    }

    #[test]
    fn shop_map_symbol_renders_as_house_shape() {
        let app = App::new();
        let mut scene = SceneBuilder::new();

        app.render_map_node_symbol(
            &mut scene,
            RoomKind::Shop,
            5,
            MapNodeSymbolLayout {
                center_x: 100.0,
                center_y: 120.0,
                radius: 10.0,
            },
            "#ffaa00",
        );

        let frame = scene.finish();
        assert!(frame.contains("RECT|92.93|120.00|14.14|7.07|0.00|#ffaa00|transparent|0.00"));
        assert!(frame.lines().any(|line| {
            line == "TRI|100.00|112.93|107.07|120.00|92.93|120.00|#ffaa00|transparent|0.00"
        }));
    }

    #[test]
    fn event_map_symbol_renders_as_question_mark() {
        let app = App::new();
        let mut scene = SceneBuilder::new();

        app.render_map_node_symbol(
            &mut scene,
            RoomKind::Event,
            5,
            MapNodeSymbolLayout {
                center_x: 100.0,
                center_y: 120.0,
                radius: 10.0,
            },
            "#88aaff",
        );

        let frame = scene.finish();
        assert!(frame.contains("TEXT|100.00|128.40|21.00|center|#88aaff|display|?"));
    }

    #[test]
    fn map_info_panel_renders_modules_and_credits() {
        let mut app = App::new();
        start_run_to_map(&mut app);
        app.dungeon.as_mut().unwrap().modules = vec![ModuleId::TargetingRelay];
        app.dungeon.as_mut().unwrap().credits = 27;
        app.open_run_info();
        app.ui.run_info_progress = 1.0;

        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        assert!(frame.contains("|label|Run Info"));
        assert!(frame.contains(&format!("|center|{}|body|Level 1", TERM_GREEN_TEXT)));
        assert!(frame.contains(&format!("|center|{}|body|HP 32/32", TERM_GREEN_TEXT)));
        assert!(frame.contains(&format!("|center|{}|body|27 Credits", TERM_GREEN_TEXT)));
        assert!(frame.contains(&format!("|center|{}|body|12 Card Deck", TERM_GREEN_TEXT)));
        assert!(!frame.contains("|label|Modules"));
        assert!(frame.contains("|label|Targeting Relay"));
    }

    #[test]
    fn module_select_screen_uses_centered_title_and_single_column_cards() {
        let mut app = active_module_select_fixture();
        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        assert!(frame.contains("|display|Choose your module"));
        assert!(
            !frame.lines().any(|line| {
                line.ends_with("|1") || line.ends_with("|2") || line.ends_with("|3")
            })
        );

        let layout = app.module_select_layout().unwrap();
        assert_eq!(layout.card_rects.len(), 3);
        assert_eq!(
            app.module_select.as_ref().unwrap().options,
            vec![
                ModuleId::Nanoforge,
                ModuleId::AegisDrive,
                ModuleId::TargetingRelay,
            ]
        );
        for pair in layout.card_rects.windows(2) {
            let upper = pair[0];
            let lower = pair[1];
            assert!((upper.x - lower.x).abs() < 0.01);
            assert!(lower.y >= upper.y + upper.h);
        }
        let title_size = fit_text_size(
            "Choose your module",
            40.0,
            (app.logical_width() - 48.0).max(120.0),
        )
        .max(24.0);
        let top_margin = layout.title_y - title_size;
        let bottom_margin = app.logical_height()
            - layout.card_rects.last().unwrap().y
            - layout.card_rects.last().unwrap().h;
        assert!((top_margin - bottom_margin).abs() < 0.01);
    }

    #[test]
    fn run_info_modules_render_in_fixed_order() {
        let mut app = App::new();
        start_run_to_map(&mut app);
        app.dungeon.as_mut().unwrap().modules = vec![
            ModuleId::TargetingRelay,
            ModuleId::AegisDrive,
            ModuleId::Nanoforge,
        ];
        app.open_run_info();
        app.ui.run_info_progress = 1.0;

        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        let nanoforge = frame.find("|label|Nanoforge").unwrap();
        let aegis = frame.find("|label|Aegis Drive").unwrap();
        let targeting = frame.find("|label|Targeting Relay").unwrap();
        assert!(nanoforge < aegis);
        assert!(aegis < targeting);
    }

    #[test]
    fn run_info_keeps_bottom_padding_with_multiple_modules() {
        let mut app = App::new();
        start_run_to_map(&mut app);
        app.dungeon.as_mut().unwrap().modules = vec![
            ModuleId::PrismScope,
            ModuleId::SalvageLedger,
            ModuleId::RecoveryMatrix,
        ];
        app.open_run_info();
        app.ui.run_info_progress = 1.0;

        let layout = app.run_info_layout().unwrap();
        let rect = layout.modal_rect;
        let title_size = 24.0;
        let row_size = 18.0_f32;
        let module_name_size = 18.0;
        let module_body_size = 16.0;
        let module_title_top_gap = 10.0;
        let line_gap = 8.0_f32;
        let module_gap = 10.0;
        let title_gap = 34.0_f32;
        let modules_gap = (title_gap - (row_size + line_gap)).max(0.0_f32);
        let (_, pad_y) = standard_overlay_padding();
        let inner_w = (rect.w - 14.0 * 2.0).max(136.0);
        let modules = app.dungeon.as_ref().unwrap().modules.clone();
        let content_bottom = rect.y
            + pad_y
            + title_size
            + title_gap
            + 4.0 * row_size
            + 3.0 * line_gap
            + modules_gap
            + app.run_info_modules_block_height(
                &modules,
                inner_w,
                module_name_size,
                module_body_size,
                module_title_top_gap,
                module_gap,
            );

        assert!(rect.y + rect.h - content_bottom >= pad_y - 0.01);
    }

    #[test]
    fn map_info_button_closes_the_legend() {
        let mut app = App::new();
        start_run_to_map(&mut app);
        app.ui.legend_open = true;
        app.ui.legend_progress = 1.0;
        let info_button = app.map_layout().unwrap().info_button;

        app.handle_map_pointer(
            info_button.x + info_button.w * 0.5,
            info_button.y + info_button.h * 0.5,
        );

        assert!(app.ui.run_info_open);
        assert!(!app.ui.legend_open);
    }

    #[test]
    fn player_panel_toggles_run_info_when_no_card_is_selected() {
        let mut app = active_combat_fixture();
        let player_rect = app.layout().player_rect;

        app.handle_combat_pointer(
            player_rect.x + player_rect.w * 0.5,
            player_rect.y + player_rect.h * 0.5,
        );
        assert!(app.ui.run_info_open);

        app.ui.run_info_progress = 1.0;
        app.handle_combat_pointer(
            player_rect.x + player_rect.w * 0.5,
            player_rect.y + player_rect.h * 0.5,
        );
        assert!(!app.ui.run_info_open);
    }

    #[test]
    fn player_panel_keeps_self_target_cards_working_instead_of_opening_info() {
        let mut app = active_combat_fixture();
        app.combat.player.energy = 3;
        app.combat.deck.hand = vec![CardId::GuardStep];
        app.combat.deck.draw_pile.clear();
        app.combat.deck.discard_pile.clear();
        app.sync_combat_feedback_to_combat();
        app.select_or_play_card(0);
        let player_rect = app.layout().player_rect;

        app.handle_combat_pointer(
            player_rect.x + player_rect.w * 0.5,
            player_rect.y + player_rect.h * 0.5,
        );

        assert!(!app.ui.run_info_open);
        assert_eq!(
            app.combat_feedback.playback_kind,
            Some(CombatPlaybackKind::PlayerAction)
        );
    }

    #[test]
    fn enemy_panel_toggles_enemy_inspect_when_no_card_is_selected() {
        let mut app = active_combat_fixture();
        let enemy_rect = primary_enemy_rect(&app.layout());

        app.handle_combat_pointer(
            enemy_rect.x + enemy_rect.w * 0.5,
            enemy_rect.y + enemy_rect.h * 0.5,
        );
        assert!(app.ui.enemy_inspect_open);
        assert_eq!(app.ui.enemy_inspect_enemy, Some(0));

        app.ui.enemy_inspect_progress = 1.0;
        app.handle_combat_pointer(
            enemy_rect.x + enemy_rect.w * 0.5,
            enemy_rect.y + enemy_rect.h * 0.5,
        );
        assert!(!app.ui.enemy_inspect_open);
    }

    #[test]
    fn enemy_inspect_switches_between_panels() {
        let mut app = active_combat_fixture();
        let mut setup = crate::combat::EncounterSetup {
            player_hp: 32,
            player_max_hp: 32,
            player_max_energy: 3,
            enemies: Vec::new(),
        };
        setup.enemies.push(crate::combat::EncounterEnemySetup {
            hp: 6,
            max_hp: 6,
            block: 0,
            profile: EnemyProfileId::ScoutDrone,
            intent_index: 0,
            on_hit_bleed: 0,
        });
        setup.enemies.push(crate::combat::EncounterEnemySetup {
            hp: 14,
            max_hp: 14,
            block: 0,
            profile: EnemyProfileId::NeedlerDrone,
            intent_index: 1,
            on_hit_bleed: 0,
        });
        app.begin_encounter(setup);
        app.screen_transition = None;

        let first_enemy_rect = app.layout().enemy_rect(0).unwrap();
        let second_enemy_rect = app.layout().enemy_rect(1).unwrap();

        app.handle_combat_pointer(
            first_enemy_rect.x + first_enemy_rect.w * 0.5,
            first_enemy_rect.y + first_enemy_rect.h * 0.5,
        );
        assert_eq!(app.ui.enemy_inspect_enemy, Some(0));

        app.ui.enemy_inspect_progress = 1.0;
        app.handle_combat_pointer(
            second_enemy_rect.x + second_enemy_rect.w * 0.5,
            second_enemy_rect.y + second_enemy_rect.h * 0.5,
        );
        assert!(app.ui.enemy_inspect_open);
        assert_eq!(app.ui.enemy_inspect_enemy, Some(1));
    }

    #[test]
    fn combat_panels_switch_between_run_info_and_enemy_inspect() {
        let mut app = active_combat_fixture();
        let player_rect = app.layout().player_rect;
        let enemy_rect = primary_enemy_rect(&app.layout());

        app.handle_combat_pointer(
            player_rect.x + player_rect.w * 0.5,
            player_rect.y + player_rect.h * 0.5,
        );
        assert!(app.ui.run_info_open);
        assert!(!app.ui.enemy_inspect_open);

        app.ui.run_info_progress = 1.0;
        app.handle_combat_pointer(
            enemy_rect.x + enemy_rect.w * 0.5,
            enemy_rect.y + enemy_rect.h * 0.5,
        );
        assert!(!app.ui.run_info_open);
        assert!(app.ui.enemy_inspect_open);
        assert_eq!(app.ui.enemy_inspect_enemy, Some(0));

        app.ui.enemy_inspect_progress = 1.0;
        app.handle_combat_pointer(
            player_rect.x + player_rect.w * 0.5,
            player_rect.y + player_rect.h * 0.5,
        );
        assert!(app.ui.run_info_open);
        assert!(!app.ui.enemy_inspect_open);
    }

    #[test]
    fn enemy_panel_keeps_enemy_target_cards_working_instead_of_opening_inspect() {
        let mut app = active_combat_fixture();
        app.combat.player.energy = 3;
        app.combat.deck.hand = vec![CardId::QuickStrike];
        app.combat.deck.draw_pile.clear();
        app.combat.deck.discard_pile.clear();
        app.sync_combat_feedback_to_combat();
        app.select_or_play_card(0);
        let enemy_rect = primary_enemy_rect(&app.layout());

        app.handle_combat_pointer(
            enemy_rect.x + enemy_rect.w * 0.5,
            enemy_rect.y + enemy_rect.h * 0.5,
        );

        assert!(!app.ui.enemy_inspect_open);
        assert_eq!(
            app.combat_feedback.playback_kind,
            Some(CombatPlaybackKind::PlayerAction)
        );
    }

    #[test]
    fn enemy_inspect_modal_closes_on_outside_tap_and_escape() {
        let mut app = active_combat_fixture();
        let enemy_rect = primary_enemy_rect(&app.layout());
        let close_point = app.layout().menu_button;

        app.handle_combat_pointer(
            enemy_rect.x + enemy_rect.w * 0.5,
            enemy_rect.y + enemy_rect.h * 0.5,
        );
        app.ui.enemy_inspect_progress = 1.0;

        app.handle_combat_pointer(
            close_point.x + close_point.w * 0.5,
            close_point.y + close_point.h * 0.5,
        );
        assert!(!app.ui.enemy_inspect_open);

        app.handle_combat_pointer(
            enemy_rect.x + enemy_rect.w * 0.5,
            enemy_rect.y + enemy_rect.h * 0.5,
        );
        assert!(app.ui.enemy_inspect_open);

        app.key_down(27);
        assert!(!app.ui.enemy_inspect_open);
    }

    #[test]
    fn enemy_inspect_modal_renders_centered_name_and_sprite() {
        let mut app = active_combat_fixture();
        let enemy_name =
            localized_enemy_name(primary_enemy(&app.combat).profile, Language::English);
        let expected_codes = enemy_sprite_codes(primary_enemy(&app.combat).profile);
        app.open_enemy_inspect(0);
        app.ui.enemy_inspect_progress = 1.0;

        app.rebuild_frame();

        let layout = app.enemy_inspect_layout().unwrap();
        let frame = String::from_utf8(app.frame.clone()).unwrap();
        let title_entry = frame_text_entries(&frame)
            .into_iter()
            .find(|(x, y, _, align, _, font, text)| {
                *x >= layout.modal_rect.x - 0.01
                    && *x <= layout.modal_rect.x + layout.modal_rect.w + 0.01
                    && *y >= layout.modal_rect.y - 0.01
                    && *y <= layout.modal_rect.y + layout.modal_rect.h + 0.01
                    && align == "center"
                    && font == "label"
                    && text == enemy_name
            })
            .expect("enemy inspect modal should render a centered name");
        assert!((title_entry.0 - (layout.modal_rect.x + layout.modal_rect.w * 0.5)).abs() < 0.05);

        let modal_sprites = frame_sprite_entries(&frame)
            .into_iter()
            .filter(|(_, rect)| rect_contains_rect(layout.modal_rect, *rect))
            .collect::<Vec<_>>();
        assert_eq!(modal_sprites.len(), expected_codes.len());
        assert_eq!(
            modal_sprites
                .iter()
                .map(|(code, _)| *code)
                .collect::<Vec<_>>(),
            expected_codes
        );
        assert!(
            modal_sprites
                .iter()
                .all(|(_, rect)| rect_contains_rect(layout.modal_rect, *rect)
                    && (rect.x - layout.sprite_rect.x).abs() < 0.02
                    && (rect.y - layout.sprite_rect.y).abs() < 0.02
                    && (rect.w - layout.sprite_rect.w).abs() < 0.02
                    && (rect.h - layout.sprite_rect.h).abs() < 0.02)
        );
    }

    #[test]
    fn reward_skip_button_anchors_to_bottom() {
        let mut app = App::new();
        app.dungeon = Some(DungeonRun::new(TEST_RUN_SEED));
        app.screen = AppScreen::Reward;
        app.reward = Some(RewardState {
            tier: RewardTier::Combat,
            options: vec![CardId::QuickStrike, CardId::BarrierField, CardId::SignalTap],
            followup: RewardFollowup {
                completed_run: false,
            },
            seed: TEST_RUN_SEED,
        });

        let layout = app.reward_layout().unwrap();
        let (_, pad_y) = boot_button_tile_padding();

        assert_eq!(
            layout.skip_button.x + layout.skip_button.w * 0.5,
            LOGICAL_WIDTH * 0.5
        );
        assert!(
            (LOGICAL_HEIGHT - (layout.skip_button.y + layout.skip_button.h) - pad_y).abs() < 0.1
        );
    }

    #[test]
    fn reward_credits_label_sits_below_cards() {
        let mut app = App::new();
        app.dungeon = Some(DungeonRun::new(TEST_RUN_SEED));
        app.screen = AppScreen::Reward;
        app.reward = Some(RewardState {
            tier: RewardTier::Combat,
            options: vec![CardId::QuickStrike, CardId::BarrierField, CardId::SignalTap],
            followup: RewardFollowup {
                completed_run: false,
            },
            seed: TEST_RUN_SEED,
        });

        let layout = app.reward_layout().unwrap();
        let cards_bottom = layout
            .card_rects
            .iter()
            .map(|rect| rect.y + rect.h)
            .fold(0.0, f32::max);

        assert!(layout.credits_y > cards_bottom);
        assert!(layout.credits_y < layout.skip_button.y);
    }

    #[test]
    fn reward_layout_on_mobile_keeps_cards_and_credits_above_skip_button() {
        let mut app = App::new();
        app.dungeon = Some(DungeonRun::new(TEST_RUN_SEED));
        app.screen = AppScreen::Reward;
        app.reward = Some(RewardState {
            tier: RewardTier::Combat,
            options: vec![CardId::QuickStrike, CardId::BarrierField, CardId::SignalTap],
            followup: RewardFollowup {
                completed_run: false,
            },
            seed: TEST_RUN_SEED,
        });
        app.resize(320.0, 568.0);

        let layout = app.reward_layout().unwrap();
        let cards_bottom = layout
            .card_rects
            .iter()
            .map(|rect| rect.y + rect.h)
            .fold(0.0, f32::max);

        assert!(cards_bottom < layout.skip_button.y);
        assert!(layout.credits_y < layout.skip_button.y);
    }

    #[test]
    fn reward_skip_button_hit_test_wins_at_its_center() {
        let mut app = App::new();
        app.dungeon = Some(DungeonRun::new(TEST_RUN_SEED));
        app.screen = AppScreen::Reward;
        app.reward = Some(RewardState {
            tier: RewardTier::Combat,
            options: vec![CardId::QuickStrike, CardId::BarrierField, CardId::SignalTap],
            followup: RewardFollowup {
                completed_run: false,
            },
            seed: TEST_RUN_SEED,
        });
        let layout = app.reward_layout().unwrap();

        assert_eq!(
            app.hit_test(
                layout.skip_button.x + layout.skip_button.w * 0.5,
                layout.skip_button.y + layout.skip_button.h * 0.5,
            ),
            Some(HitTarget::RewardSkip)
        );
    }

    #[test]
    fn reward_skip_button_returns_to_map_without_adding_a_card() {
        let mut app = App::new();
        let dungeon = DungeonRun::new(TEST_RUN_SEED);
        let initial_deck_len = dungeon.deck.len();
        app.dungeon = Some(dungeon);
        app.screen = AppScreen::Reward;
        app.reward = Some(RewardState {
            tier: RewardTier::Combat,
            options: vec![CardId::QuickStrike, CardId::BarrierField],
            followup: RewardFollowup {
                completed_run: false,
            },
            seed: TEST_RUN_SEED,
        });

        let layout = app.reward_layout().unwrap();
        let x = layout.skip_button.x + layout.skip_button.w * 0.5;
        let y = layout.skip_button.y + layout.skip_button.h * 0.5;

        assert_eq!(app.hit_test(x, y), Some(HitTarget::RewardSkip));

        app.handle_reward_pointer(x, y);

        assert!(matches!(app.screen, AppScreen::Map));
        assert!(app.reward.is_none());
        assert_eq!(app.dungeon.as_ref().unwrap().deck.len(), initial_deck_len);
        assert_eq!(
            app.log.back().map(String::as_str),
            Some("Skipped card reward.")
        );
    }

    #[test]
    fn reward_skip_hotkey_uses_zero() {
        let mut app = App::new();
        let dungeon = DungeonRun::new(TEST_RUN_SEED);
        let initial_deck_len = dungeon.deck.len();
        app.dungeon = Some(dungeon);
        app.screen = AppScreen::Reward;
        app.reward = Some(RewardState {
            tier: RewardTier::Elite,
            options: vec![CardId::BarrierField, CardId::BreachSignal],
            followup: RewardFollowup {
                completed_run: false,
            },
            seed: TEST_AUXILIARY_SEED,
        });

        app.key_down(48);

        assert!(matches!(app.screen, AppScreen::Map));
        assert_eq!(app.dungeon.as_ref().unwrap().deck.len(), initial_deck_len);
        assert!(app.reward.is_none());
    }

    #[test]
    fn combat_save_round_trip_restores_exact_combat_state() {
        let app = combat_save_fixture();
        let snapshot = app.serialize_current_run().unwrap();

        let restored = restore_app_from_snapshot(&snapshot);

        assert!(matches!(restored.screen, AppScreen::Combat));
        assert_eq!(restored.dungeon, app.dungeon);
        assert_eq!(restored.combat, app.combat);
        assert_eq!(restored.log, app.log);
    }

    #[test]
    fn restore_from_buffer_clears_snapshot_for_unsupported_save_format() {
        let mut app = combat_save_fixture();
        let snapshot = app.serialize_current_run().unwrap();
        let mut value: serde_json::Value = serde_json::from_str(&snapshot).unwrap();
        value["save_format_version"] =
            serde_json::Value::from(crate::save::SAVE_FORMAT_VERSION + 1);
        let invalid_snapshot = serde_json::to_string(&value).unwrap();
        app.set_run_save_snapshot(Some(invalid_snapshot.clone()));
        app.resume_request_pending = true;
        app.restore_buffer = invalid_snapshot.into_bytes();

        let restored = app.restore_from_buffer(app.restore_buffer.len());

        assert!(!restored);
        assert!(!app.resume_request_pending);
        assert!(app.run_save_snapshot.is_none());
        assert!(!app.has_saved_run);
    }

    #[test]
    fn restore_from_save_clears_stale_combat_layout_target() {
        let mut app = combat_save_fixture();
        let snapshot = app.serialize_current_run().unwrap();
        let mut stale_target = app.layout_target();
        stale_target.player_rect.x += 100.0;
        app.combat_layout_target = Some(stale_target);

        app.restore_from_save_raw(&snapshot).unwrap();

        assert!(matches!(app.screen, AppScreen::Combat));
        assert!(app.layout_transition.is_none());
        assert!(combat_layouts_match(
            app.combat_layout_target.as_ref().unwrap(),
            &app.layout_target()
        ));
    }

    fn incompatible_combat_save_falls_back_to_encounter_checkpoint() {
        let app = combat_save_fixture();
        let original_turn = app.combat.turn;
        let mut envelope = parse_run_save(&app.serialize_current_run().unwrap()).unwrap();
        if let SavedRunState::Combat { combat, .. } = &mut envelope.active_state {
            combat.deck.hand[0] = "removed_card".to_string();
        } else {
            panic!("expected combat save");
        }
        let broken_snapshot = serialize_envelope(&envelope).unwrap();

        let restored = restore_app_from_snapshot(&broken_snapshot);

        assert!(matches!(restored.screen, AppScreen::Combat));
        assert!(restored.combat.turn < original_turn);
        assert_eq!(restored.log.len(), 1);
        assert_eq!(restored.log.back().unwrap(), "Run resumed from checkpoint.");
    }

    #[test]
    fn aegis_drive_grants_shield_at_the_start_of_combat() {
        let mut app = App::new();
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.modules = vec![ModuleId::AegisDrive];
        dungeon.nodes = vec![
            crate::dungeon::DungeonNode {
                id: 0,
                depth: 0,
                lane: 3,
                kind: RoomKind::Start,
                next: vec![1],
            },
            crate::dungeon::DungeonNode {
                id: 1,
                depth: 1,
                lane: 3,
                kind: RoomKind::Combat,
                next: vec![],
            },
        ];
        dungeon.current_node = Some(1);
        dungeon.available_nodes.clear();
        let setup = dungeon.current_encounter_setup().unwrap();
        app.dungeon = Some(dungeon);

        app.begin_encounter(setup);

        assert_eq!(app.combat.player.fighter.block, 5);
        assert!(
            app.log
                .iter()
                .any(|line| line == "Aegis Drive grants 5 Shield.")
        );
    }

    #[test]
    fn targeting_relay_grants_focus_at_the_start_of_combat() {
        let mut app = App::new();
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.modules = vec![ModuleId::TargetingRelay];
        dungeon.nodes = vec![
            crate::dungeon::DungeonNode {
                id: 0,
                depth: 0,
                lane: 3,
                kind: RoomKind::Start,
                next: vec![1],
            },
            crate::dungeon::DungeonNode {
                id: 1,
                depth: 1,
                lane: 3,
                kind: RoomKind::Combat,
                next: vec![],
            },
        ];
        dungeon.current_node = Some(1);
        dungeon.available_nodes.clear();
        let setup = dungeon.current_encounter_setup().unwrap();
        app.dungeon = Some(dungeon);

        app.begin_encounter(setup);

        assert_eq!(app.combat.player.fighter.statuses.focus, 1);
        assert!(
            app.log
                .iter()
                .any(|line| line == "Targeting Relay grants Focus +1.")
        );
    }

    #[test]
    fn capacitor_bank_grants_momentum_at_the_start_of_combat() {
        let mut app = App::new();
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.modules = vec![ModuleId::CapacitorBank];
        app.dungeon = Some(dungeon);

        app.begin_encounter(crate::combat::EncounterSetup::default());

        assert_eq!(app.combat.player.fighter.statuses.momentum, 1);
        assert!(
            app.log
                .iter()
                .any(|line| line == "Capacitor Bank grants Momentum +1.")
        );
    }

    #[test]
    fn prism_scope_applies_rhythm_to_all_enemies_at_the_start_of_combat() {
        let mut app = App::new();
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.modules = vec![ModuleId::PrismScope];
        app.dungeon = Some(dungeon);
        let mut setup = crate::combat::EncounterSetup::default();
        setup.enemies.push(crate::combat::EncounterEnemySetup {
            hp: 28,
            max_hp: 28,
            block: 0,
            profile: EnemyProfileId::NeedlerDrone,
            intent_index: 1,
            on_hit_bleed: 0,
        });

        app.begin_encounter(setup);

        assert_eq!(app.combat.enemies.len(), 2);
        assert!(
            app.combat
                .enemies
                .iter()
                .all(|enemy| enemy.fighter.statuses.rhythm == -1)
        );
        assert!(
            app.log
                .iter()
                .any(|line| line == "Prism Scope applies Rhythm -1 to all enemies.")
        );
    }

    #[test]
    fn overclock_core_grants_extra_energy_at_the_start_of_combat() {
        let mut app = App::new();
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.modules = vec![ModuleId::OverclockCore];
        app.dungeon = Some(dungeon);

        app.begin_encounter(crate::combat::EncounterSetup::default());

        assert_eq!(app.combat.player.max_energy, 4);
        assert_eq!(app.combat.player.energy, 4);
        assert!(
            app.log
                .iter()
                .any(|line| line == "Overclock Core grants 1 extra Energy.")
        );
    }

    #[test]
    fn suppression_field_applies_focus_to_all_enemies_at_the_start_of_combat() {
        let mut app = App::new();
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.modules = vec![ModuleId::SuppressionField];
        app.dungeon = Some(dungeon);
        let mut setup = crate::combat::EncounterSetup::default();
        setup.enemies.push(crate::combat::EncounterEnemySetup {
            hp: 28,
            max_hp: 28,
            block: 0,
            profile: EnemyProfileId::NeedlerDrone,
            intent_index: 1,
            on_hit_bleed: 0,
        });

        app.begin_encounter(setup);

        assert!(
            app.combat
                .enemies
                .iter()
                .all(|enemy| enemy.fighter.statuses.focus == -1)
        );
        assert!(
            app.log
                .iter()
                .any(|line| line == "Suppression Field applies Focus -1 to all enemies.")
        );
    }

    #[test]
    fn combat_panels_render_focus_rhythm_and_momentum_labels() {
        let mut app = active_combat_fixture();
        app.combat.player.fighter.statuses.focus = 2;
        app.combat.player.fighter.statuses.rhythm = -1;
        primary_enemy_mut(&mut app.combat).fighter.statuses.rhythm = -1;
        primary_enemy_mut(&mut app.combat).fighter.statuses.momentum = 1;

        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        assert!(frame.contains("|body|Focus+2"));
        assert!(frame.contains("|body|Rhythm-1"));
        assert!(frame.contains("|body|Momentum+1"));
    }

    #[test]
    fn nanoforge_restores_hp_after_victory() {
        let mut app = App::new();
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.modules = vec![ModuleId::Nanoforge];
        dungeon.player_hp = 20;
        dungeon.nodes = vec![
            crate::dungeon::DungeonNode {
                id: 0,
                depth: 0,
                lane: 3,
                kind: RoomKind::Start,
                next: vec![1],
            },
            crate::dungeon::DungeonNode {
                id: 1,
                depth: 1,
                lane: 3,
                kind: RoomKind::Combat,
                next: vec![],
            },
        ];
        dungeon.current_node = Some(1);
        dungeon.available_nodes.clear();
        app.dungeon = Some(dungeon);
        app.screen = AppScreen::Combat;
        app.combat.player.fighter.hp = 20;
        app.debug_mode = true;

        app.debug_end_battle();

        assert_eq!(app.dungeon.as_ref().unwrap().player_hp, 22);
        assert!(
            app.log
                .iter()
                .any(|line| line == "Nanoforge restores 2 HP.")
        );
    }

    #[test]
    fn salvage_ledger_grants_credits_after_victory() {
        let mut app = App::new();
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.modules = vec![ModuleId::SalvageLedger];
        dungeon.nodes = vec![
            crate::dungeon::DungeonNode {
                id: 0,
                depth: 0,
                lane: 3,
                kind: RoomKind::Start,
                next: vec![1],
            },
            crate::dungeon::DungeonNode {
                id: 1,
                depth: 1,
                lane: 3,
                kind: RoomKind::Combat,
                next: vec![],
            },
        ];
        dungeon.current_node = Some(1);
        dungeon.available_nodes.clear();
        app.dungeon = Some(dungeon);
        app.screen = AppScreen::Combat;
        app.debug_mode = true;

        app.debug_end_battle();

        assert_eq!(app.dungeon.as_ref().unwrap().credits, 10);
        assert!(
            app.log
                .iter()
                .any(|line| line == "Salvage Ledger grants 4 additional Credits.")
        );
    }

    #[test]
    fn recovery_matrix_restores_hp_after_victory() {
        let mut app = App::new();
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.modules = vec![ModuleId::RecoveryMatrix];
        dungeon.player_hp = 20;
        dungeon.nodes = vec![
            crate::dungeon::DungeonNode {
                id: 0,
                depth: 0,
                lane: 3,
                kind: RoomKind::Start,
                next: vec![1],
            },
            crate::dungeon::DungeonNode {
                id: 1,
                depth: 1,
                lane: 3,
                kind: RoomKind::Combat,
                next: vec![],
            },
        ];
        dungeon.current_node = Some(1);
        dungeon.available_nodes.clear();
        app.dungeon = Some(dungeon);
        app.screen = AppScreen::Combat;
        app.combat.player.fighter.hp = 20;
        app.debug_mode = true;

        app.debug_end_battle();

        assert_eq!(app.dungeon.as_ref().unwrap().player_hp, 25);
        assert!(
            app.log
                .iter()
                .any(|line| line == "Recovery Matrix restores 5 HP.")
        );
    }

    #[test]
    fn end_turn_playback_locks_combat_input_and_shows_enemy_banner() {
        let mut app = active_combat_fixture();
        let end_turn_button = app.layout().end_turn_button;

        assert_eq!(
            app.hit_test(
                end_turn_button.x + end_turn_button.w * 0.5,
                end_turn_button.y + end_turn_button.h * 0.5
            ),
            Some(HitTarget::EndTurn)
        );

        app.perform_action(CombatAction::EndTurn);

        assert!(app.combat_input_locked());
        assert_eq!(
            app.hit_test(
                end_turn_button.x + end_turn_button.w * 0.5,
                end_turn_button.y + end_turn_button.h * 0.5
            ),
            None
        );

        advance_until(&mut app, |app| {
            app.combat_feedback
                .turn_banner
                .as_ref()
                .is_some_and(|banner| banner.text == "Enemy Turn")
        });
    }

    #[test]
    fn end_turn_playback_counts_down_block_before_hp() {
        let mut app = active_combat_fixture();
        app.combat.player.fighter.hp = 32;
        app.combat.player.fighter.block = 5;
        set_primary_enemy_intent(&mut app, EnemyProfileId::RampartDrone, 0);
        app.sync_combat_feedback_to_combat();

        app.perform_action(CombatAction::EndTurn);

        advance_until(&mut app, |app| {
            app.combat_feedback.displayed.player.block < 5
        });
        assert_eq!(app.combat_feedback.displayed.player.hp, 32);

        advance_until(&mut app, |app| {
            app.combat_feedback.displayed.player.block == 0
        });
        assert_eq!(app.combat_feedback.displayed.player.hp, 32);

        advance_until(&mut app, |app| app.combat_feedback.displayed.player.hp < 32);
        assert_eq!(app.combat_feedback.displayed.player.block, 0);
    }

    #[test]
    fn end_turn_playback_animates_player_meta_from_real_deck_events() {
        let mut app = active_combat_fixture();
        app.combat.player.energy = 0;
        app.combat.player.max_energy = 3;
        app.combat.deck.hand = vec![CardId::QuickStrike, CardId::GuardStep];
        app.combat.deck.draw_pile =
            vec![CardId::FlareSlash, CardId::PinpointJab, CardId::Slipstream];
        app.combat.deck.discard_pile = vec![
            CardId::SignalTap,
            CardId::BurstArray,
            CardId::CoverPulse,
            CardId::BarrierField,
            CardId::ZeroPoint,
            CardId::PressurePoint,
            CardId::GuardStepPlus,
            CardId::QuickStrikePlus,
        ];
        app.sync_combat_feedback_to_combat();

        let initial_meta = app.displayed_player_meta();
        assert_eq!(
            initial_meta,
            PlayerDisplayedMeta {
                energy: 0,
                draw_pile: 3,
                discard_pile: 8,
            }
        );

        app.perform_action(CombatAction::EndTurn);
        assert_eq!(app.displayed_player_meta(), initial_meta);

        advance_until(&mut app, |app| {
            app.displayed_player_meta().discard_pile == 9
        });
        assert_eq!(app.displayed_player_meta().draw_pile, 3);
        advance_until(&mut app, |app| {
            app.displayed_player_meta().discard_pile == 10
        });
        assert_eq!(app.displayed_player_meta().draw_pile, 3);

        advance_until(&mut app, |app| {
            player_active_stats(app).contains(&CombatStat::Energy)
        });
        advance_until(&mut app, |app| {
            app.displayed_player_meta().energy > initial_meta.energy
        });
        let animated_energy = app.displayed_player_meta().energy;
        assert!(animated_energy < i32::from(app.combat.player.max_energy));
        app.rebuild_frame();
        let player_panel = app.layout().player_rect;
        let entries = frame_text_entries(&String::from_utf8(app.frame.clone()).unwrap());
        assert!(panel_has_text_with_color(
            &entries,
            player_panel,
            &format!("{animated_energy}/{}", app.combat.player.max_energy),
            TERM_CYAN_SOFT,
        ));

        advance_until(&mut app, |app| app.displayed_player_meta().draw_pile == 2);
        advance_until(&mut app, |app| app.displayed_player_meta().draw_pile == 1);
        advance_until(&mut app, |app| app.displayed_player_meta().draw_pile == 0);
        assert_eq!(app.displayed_player_meta().discard_pile, 10);

        advance_until(&mut app, |app| {
            let active_stats = player_active_stats(app);
            active_stats.contains(&CombatStat::DrawPile)
                && active_stats.contains(&CombatStat::DiscardPile)
        });
        assert_eq!(app.displayed_player_meta().draw_pile, 0);
        assert_eq!(app.displayed_player_meta().discard_pile, 10);
        app.rebuild_frame();
        let entries = frame_text_entries(&String::from_utf8(app.frame.clone()).unwrap());
        assert!(panel_has_text_with_color(
            &entries,
            player_panel,
            "0",
            TERM_CYAN_SOFT,
        ));
        assert!(panel_has_text_with_color(
            &entries,
            player_panel,
            "10",
            TERM_CYAN_SOFT,
        ));

        advance_until(&mut app, |app| {
            app.displayed_player_meta().draw_pile == 10
                && app.displayed_player_meta().discard_pile == 0
        });
        advance_until(&mut app, |app| app.displayed_player_meta().draw_pile == 9);

        advance_until(&mut app, |app| app.combat_feedback.playback_kind.is_none());

        let final_meta = app.displayed_player_meta();
        assert_eq!(
            final_meta,
            PlayerDisplayedMeta {
                energy: app.combat.player.energy as i32,
                draw_pile: app.combat.deck.draw_pile.len() as i32,
                discard_pile: app.combat.deck.discard_pile.len() as i32,
            }
        );

        app.rebuild_frame();
        let entries = frame_text_entries(&String::from_utf8(app.frame.clone()).unwrap());
        assert!(panel_has_text_with_color(
            &entries,
            player_panel,
            &format!("{}/{}", final_meta.energy, app.combat.player.max_energy),
            TERM_CYAN,
        ));
        assert!(panel_has_text_with_color(
            &entries,
            player_panel,
            &final_meta.draw_pile.to_string(),
            TERM_CYAN,
        ));
        assert!(panel_has_text_with_color(
            &entries,
            player_panel,
            &final_meta.discard_pile.to_string(),
            TERM_CYAN,
        ));
    }

    #[test]
    fn player_attack_playback_counts_enemy_block_before_hp() {
        let mut app = active_combat_fixture();
        app.combat.player.energy = 3;
        app.combat.deck.hand = vec![CardId::FlareSlash];
        app.combat.deck.draw_pile.clear();
        app.combat.deck.discard_pile.clear();
        primary_enemy_mut(&mut app.combat).fighter.hp = 10;
        primary_enemy_mut(&mut app.combat).fighter.block = 4;
        app.sync_combat_feedback_to_combat();

        app.perform_action(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(Actor::Enemy(0)),
        });

        assert!(app.combat_input_locked());
        assert_eq!(
            app.combat_feedback.playback_kind,
            Some(CombatPlaybackKind::PlayerAction)
        );

        advance_until(&mut app, |app| displayed_primary_enemy(app).block < 4);
        assert_eq!(displayed_primary_enemy(&app).hp, 10);

        advance_until(&mut app, |app| displayed_primary_enemy(app).block == 0);
        assert_eq!(displayed_primary_enemy(&app).hp, 10);

        advance_until(&mut app, |app| displayed_primary_enemy(app).hp < 10);
        assert_eq!(displayed_primary_enemy(&app).block, 0);

        advance_until(&mut app, |app| !app.combat_input_locked());
    }

    #[test]
    fn player_action_playback_syncs_player_meta_without_meta_countdowns() {
        let mut app = active_combat_fixture();
        app.combat.player.energy = 3;
        app.combat.deck.hand = vec![CardId::FlareSlash];
        app.combat.deck.draw_pile = vec![CardId::GuardStep, CardId::ZeroPoint];
        app.combat.deck.discard_pile.clear();
        app.sync_combat_feedback_to_combat();

        let expected_meta = PlayerDisplayedMeta {
            energy: 2,
            draw_pile: 2,
            discard_pile: 1,
        };

        app.perform_action(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(Actor::Enemy(0)),
        });

        assert_eq!(
            app.combat_feedback.playback_kind,
            Some(CombatPlaybackKind::PlayerAction)
        );
        assert_eq!(
            app.displayed_player_meta(),
            PlayerDisplayedMeta {
                energy: 3,
                draw_pile: 2,
                discard_pile: 0,
            }
        );

        let mut saw_expected_meta = false;
        let mut saw_meta_countdown = false;
        for _ in 0..400 {
            if app.combat_feedback.active_stats.iter().any(|active| {
                active.actor == Actor::Player
                    && matches!(
                        active.stat,
                        CombatStat::Energy | CombatStat::DrawPile | CombatStat::DiscardPile
                    )
            }) {
                saw_meta_countdown = true;
            }
            if app.displayed_player_meta() == expected_meta {
                saw_expected_meta = true;
            }
            if app.combat_feedback.playback_kind.is_none() {
                break;
            }
            advance_time(&mut app, 16.0);
        }

        assert!(saw_expected_meta);
        assert!(!saw_meta_countdown);
        assert_eq!(app.displayed_player_meta(), expected_meta);
    }

    #[test]
    fn player_block_card_playback_counts_shield_up_in_green() {
        let mut app = active_combat_fixture();
        app.combat.player.energy = 3;
        app.combat.deck.hand = vec![CardId::GuardStep];
        app.combat.deck.draw_pile.clear();
        app.combat.deck.discard_pile.clear();
        app.combat.player.fighter.block = 0;
        app.sync_combat_feedback_to_combat();

        app.perform_action(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(Actor::Player),
        });

        assert!(app.combat_input_locked());
        assert_eq!(
            app.combat_feedback.playback_kind,
            Some(CombatPlaybackKind::PlayerAction)
        );

        advance_until(&mut app, |app| {
            app.combat_feedback.displayed.player.block > 0
        });
        assert!(app.combat_feedback.displayed.player.block < 5);
        assert_eq!(
            animated_stat_color(&app, Actor::Player, CombatStat::Block),
            TERM_GREEN_SOFT
        );

        advance_until(&mut app, |app| {
            app.combat_feedback.displayed.player.block == 5
        });
        advance_until(&mut app, |app| !app.combat_input_locked());
    }

    #[test]
    fn enemy_turn_banner_and_damage_render_into_frame() {
        let mut app = active_combat_fixture();
        app.combat.player.fighter.hp = 32;
        app.combat.player.fighter.block = 0;
        set_primary_enemy_intent(&mut app, EnemyProfileId::ScoutDrone, 0);
        app.sync_combat_feedback_to_combat();

        app.perform_action(CombatAction::EndTurn);
        advance_until(&mut app, |app| {
            app.combat_feedback
                .turn_banner
                .as_ref()
                .is_some_and(|banner| banner.text == "Enemy Turn")
        });

        let layout = app.layout();
        let banner_rect = app.turn_banner_rect(&layout, "Enemy Turn");
        let font_size = combat_top_button_font_size(layout.low_hand_layout, layout.tile_scale);
        let banner = app
            .combat_feedback
            .turn_banner
            .as_ref()
            .expect("enemy turn banner should be active");
        let alpha = (banner.ttl_ms / banner.total_ms).clamp(0.0, 1.0);
        let expected_stroke = ui_emphasize_tile_stroke(&rgba(banner.color, 0.78 * alpha));
        let expected_fill = rgba(banner.color, UI_TILE_FILL_ALPHA * alpha);
        let banner_frame = String::from_utf8(app.frame.clone()).unwrap();
        let banner_rects = frame_rect_entries(&banner_frame);
        let banner_entry = find_frame_rect_entry(&banner_rects, banner_rect);
        assert_eq!(banner_entry.radius, BUTTON_RADIUS);
        assert_eq!(banner_entry.stroke_width, UI_TILE_STROKE_WIDTH);
        assert_eq!(banner_entry.fill, expected_fill);
        assert_eq!(banner_entry.stroke, expected_stroke);
        assert!(banner_frame.contains(&format!(
            "TEXT|{:.2}|{:.2}|{:.2}|center|",
            banner_rect.x + banner_rect.w * 0.5,
            button_text_baseline(banner_rect, font_size),
            font_size
        )));
        assert!(banner_frame.contains("|label|Enemy Turn"));

        advance_until(&mut app, |app| {
            app.combat_feedback.displayed.player.hp == 31
        });
        let hp_frame = String::from_utf8(app.frame.clone()).unwrap();
        assert!(hp_frame.contains(&format!("|left|{}|body|31", TERM_PINK_SOFT)));
    }

    #[test]
    fn combat_action_buttons_use_scaled_font_and_padding() {
        let mut app = active_combat_fixture();
        app.debug_mode = true;

        let layout = app.layout();
        let font_size = combat_action_button_font_size(layout.low_hand_layout, layout.tile_scale);
        let (pad_x, pad_y) = combat_action_button_padding(layout.tile_insets);
        let menu_size = button_size(app.tr("Menu", "Menú"), font_size, pad_x, pad_y);
        let end_turn_size =
            button_size(app.tr("End Turn", "Fin del turno"), font_size, pad_x, pad_y);
        let end_battle_size = button_size(
            app.tr("End Battle", "Fin de batalla"),
            font_size,
            pad_x,
            pad_y,
        );

        assert!((layout.menu_button.w - menu_size.0).abs() < 0.01);
        assert!((layout.menu_button.h - menu_size.1).abs() < 0.01);
        assert!((layout.end_turn_button.w - end_turn_size.0).abs() < 0.01);
        assert!((layout.end_turn_button.h - end_turn_size.1).abs() < 0.01);

        let end_battle_button = layout
            .end_battle_button
            .expect("debug mode should show End Battle");
        assert!((end_battle_button.w - end_battle_size.0).abs() < 0.01);
        assert!((end_battle_button.h - end_battle_size.1).abs() < 0.01);

        let banner_rect = app.turn_banner_rect(&layout, "Enemy Turn");
        assert!(layout.menu_button.h < banner_rect.h);
        assert!(layout.end_turn_button.h < banner_rect.h);
        assert!(end_battle_button.h < banner_rect.h);
    }

    #[test]
    fn combat_hint_tile_uses_scaled_metrics_and_stays_centered() {
        let app = active_combat_fixture();
        let layout = app.layout();
        let hint_rect = layout
            .hint_rect
            .expect("combat layout should include a hint tile");
        let (hint_font_size, pad_x, pad_y) = hand_hint_metrics(layout.tile_scale);
        let (message, _, _) = combat_hint_tile(&app, app.combat.hand_len());

        assert!((hint_rect.w - (text_width(&message, hint_font_size) + pad_x * 2.0)).abs() < 0.01);
        assert!((hint_rect.h - (hint_font_size + pad_y * 2.0)).abs() < 0.01);
        assert!((hint_rect.x + hint_rect.w * 0.5 - app.logical_center_x()).abs() < 0.05);
    }

    #[test]
    fn combat_action_tiles_match_hand_card_stroke_width() {
        let mut app = active_combat_fixture();
        app.rebuild_frame();

        let layout = app.layout();
        let frame = String::from_utf8(app.frame.clone()).unwrap();
        let rect_entries = frame_rect_entries(&frame);
        let hint_rect = layout
            .hint_rect
            .expect("combat layout should include a hint tile");
        let menu = find_frame_rect_entry(&rect_entries, layout.menu_button);
        let end_turn = find_frame_rect_entry(&rect_entries, layout.end_turn_button);
        let hint = find_frame_rect_entry(&rect_entries, hint_rect);
        let first_card = find_frame_rect_entry(&rect_entries, layout.hand_rects[0]);

        assert_eq!(menu.stroke_width, UI_TILE_STROKE_WIDTH);
        assert_eq!(end_turn.stroke_width, UI_TILE_STROKE_WIDTH);
        assert_eq!(hint.stroke_width, UI_TILE_STROKE_WIDTH);
        assert_eq!(first_card.stroke_width, UI_TILE_STROKE_WIDTH);
        assert_eq!(menu.stroke_width, first_card.stroke_width);
        assert_eq!(end_turn.stroke_width, first_card.stroke_width);
        assert_eq!(hint.stroke_width, first_card.stroke_width);
    }

    #[test]
    fn combat_tiles_use_tinted_fill_in_idle_state() {
        let mut app = active_combat_fixture();
        app.debug_mode = true;
        app.rebuild_frame();

        let layout = app.layout();
        let frame = String::from_utf8(app.frame.clone()).unwrap();
        let rect_entries = frame_rect_entries(&frame);
        let hint_rect = layout
            .hint_rect
            .expect("combat layout should include a hint tile");

        let menu = find_frame_rect_entry(&rect_entries, layout.menu_button);
        assert_eq!(menu.fill, rgba((51, 255, 102), UI_TILE_FILL_ALPHA));
        assert_eq!(menu.stroke_width, UI_TILE_STROKE_WIDTH);
        assert_eq!(
            menu.stroke,
            ui_emphasize_tile_stroke(COLOR_GREEN_STROKE_IDLE)
        );

        let end_turn = find_frame_rect_entry(&rect_entries, layout.end_turn_button);
        assert_eq!(end_turn.fill, rgba((61, 245, 255), UI_TILE_FILL_ALPHA));
        assert_eq!(end_turn.stroke_width, UI_TILE_STROKE_WIDTH);
        assert_eq!(
            end_turn.stroke,
            ui_emphasize_tile_stroke(COLOR_CYAN_STROKE_IDLE)
        );

        let end_battle = find_frame_rect_entry(
            &rect_entries,
            layout
                .end_battle_button
                .expect("debug mode should show End Battle"),
        );
        assert_eq!(end_battle.fill, rgba((51, 255, 102), UI_TILE_FILL_ALPHA));
        assert_eq!(end_battle.stroke_width, UI_TILE_STROKE_WIDTH);
        assert_eq!(
            end_battle.stroke,
            ui_emphasize_tile_stroke(COLOR_GREEN_STROKE_IDLE)
        );

        let hint = find_frame_rect_entry(&rect_entries, hint_rect);
        assert_eq!(hint.fill, rgba((51, 255, 102), UI_TILE_FILL_ALPHA));
        assert_eq!(hint.stroke_width, UI_TILE_STROKE_WIDTH);
        assert_eq!(
            hint.stroke,
            ui_emphasize_tile_stroke(COLOR_GREEN_STROKE_CARD)
        );

        let enemy = find_frame_rect_entry(&rect_entries, primary_enemy_rect(&layout));
        assert_eq!(enemy.fill, rgba((51, 255, 102), UI_TILE_FILL_ALPHA));
        assert_eq!(enemy.stroke_width, UI_TILE_STROKE_WIDTH);
        assert_eq!(
            enemy.stroke,
            ui_emphasize_tile_stroke(COLOR_GREEN_STROKE_PANEL)
        );

        let player = find_frame_rect_entry(&rect_entries, layout.player_rect);
        assert_eq!(player.fill, rgba((51, 255, 102), UI_TILE_FILL_ALPHA));
        assert_eq!(player.stroke_width, UI_TILE_STROKE_WIDTH);
        assert_eq!(
            player.stroke,
            ui_emphasize_tile_stroke(COLOR_GREEN_STROKE_CARD)
        );

        let first_card = find_frame_rect_entry(&rect_entries, layout.hand_rects[0]);
        assert_eq!(first_card.fill, rgba((51, 255, 102), UI_TILE_FILL_ALPHA));
        assert_eq!(first_card.stroke_width, UI_TILE_STROKE_WIDTH);
        assert_eq!(
            first_card.stroke,
            ui_emphasize_tile_stroke(COLOR_GREEN_STROKE_CARD)
        );
    }

    #[test]
    fn combat_tiles_tint_fill_tracks_selected_target_state() {
        let mut app = active_combat_fixture();
        app.combat.deck.hand = vec![CardId::FlareSlash, CardId::GuardStep];
        assert!(app.combat.card_requires_enemy(0));
        assert!(!app.combat.card_requires_enemy(1));

        app.ui.selected_card = Some(0);
        app.rebuild_frame();
        let enemy_frame = String::from_utf8(app.frame.clone()).unwrap();
        let enemy_rects = frame_rect_entries(&enemy_frame);
        let enemy_layout = app.layout();
        let targeted_enemy = find_frame_rect_entry(&enemy_rects, primary_enemy_rect(&enemy_layout));
        assert_eq!(
            targeted_enemy.fill,
            rgba((216, 255, 61), UI_TILE_FILL_ALPHA)
        );
        assert_eq!(targeted_enemy.stroke_width, UI_TILE_STROKE_WIDTH);
        assert_eq!(
            targeted_enemy.stroke,
            ui_emphasize_tile_stroke(COLOR_LIME_STROKE_TARGET)
        );
        let selected_enemy_card = find_frame_rect_entry(&enemy_rects, enemy_layout.hand_rects[0]);
        assert_eq!(
            selected_enemy_card.fill,
            rgba((216, 255, 61), UI_TILE_FILL_ALPHA)
        );
        assert_eq!(selected_enemy_card.stroke_width, UI_TILE_STROKE_WIDTH);
        assert_eq!(
            selected_enemy_card.stroke,
            ui_emphasize_tile_stroke(COLOR_LIME_STROKE_TARGET)
        );

        app.ui.selected_card = Some(1);
        app.rebuild_frame();
        let player_frame = String::from_utf8(app.frame.clone()).unwrap();
        let player_rects = frame_rect_entries(&player_frame);
        let player_layout = app.layout();
        let targeted_player = find_frame_rect_entry(&player_rects, player_layout.player_rect);
        assert_eq!(
            targeted_player.fill,
            rgba((61, 245, 255), UI_TILE_FILL_ALPHA)
        );
        assert_eq!(targeted_player.stroke_width, UI_TILE_STROKE_WIDTH);
        assert_eq!(
            targeted_player.stroke,
            ui_emphasize_tile_stroke(COLOR_CYAN_STROKE_TARGET)
        );
        let selected_player_card =
            find_frame_rect_entry(&player_rects, player_layout.hand_rects[1]);
        assert_eq!(
            selected_player_card.fill,
            rgba((61, 245, 255), UI_TILE_FILL_ALPHA)
        );
        assert_eq!(selected_player_card.stroke_width, UI_TILE_STROKE_WIDTH);
        assert_eq!(
            selected_player_card.stroke,
            ui_emphasize_tile_stroke(COLOR_CYAN_STROKE_TARGET)
        );
    }

    #[test]
    fn combat_panels_use_tinted_svg_icons_in_both_languages() {
        let mut app = active_combat_fixture();
        app.combat.player.energy = 2;
        app.combat.deck.draw_pile = vec![CardId::FlareSlash, CardId::GuardStep];
        app.combat.deck.discard_pile = vec![CardId::QuickStrike];
        app.sync_combat_feedback_to_combat();
        app.dirty = true;
        app.tick(0.0);

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        let icons = frame_tinted_image_entries(&frame);
        let player_panel = app.layout().player_rect;
        let enemy_panel = primary_enemy_rect(&app.layout());

        assert_eq!(
            icons
                .iter()
                .filter(|entry| entry.src == COMBAT_HEART_ICON_ASSET_PATH)
                .count(),
            2
        );
        assert_eq!(
            icons
                .iter()
                .filter(|entry| entry.src == COMBAT_SHIELD_ICON_ASSET_PATH)
                .count(),
            2
        );
        assert_eq!(
            icons
                .iter()
                .filter(|entry| entry.src == COMBAT_ENERGY_ICON_ASSET_PATH)
                .count(),
            1
        );
        assert_eq!(
            icons
                .iter()
                .filter(|entry| entry.src == COMBAT_DECK_ICON_ASSET_PATH)
                .count(),
            1
        );
        assert_eq!(
            icons
                .iter()
                .filter(|entry| entry.color == TERM_GREEN_TEXT)
                .count(),
            4
        );
        assert_eq!(
            icons
                .iter()
                .filter(|entry| entry.color == TERM_CYAN)
                .count(),
            2
        );
        assert_eq!(
            icons
                .iter()
                .filter(|entry| {
                    entry.src == COMBAT_HEART_ICON_ASSET_PATH
                        && rect_contains_rect(player_panel, entry.rect)
                })
                .count(),
            1
        );
        assert_eq!(
            icons
                .iter()
                .filter(|entry| {
                    entry.src == COMBAT_HEART_ICON_ASSET_PATH
                        && rect_contains_rect(enemy_panel, entry.rect)
                })
                .count(),
            1
        );
        assert!(icons.iter().all(|entry| {
            rect_contains_rect(player_panel, entry.rect)
                || rect_contains_rect(enemy_panel, entry.rect)
        }));

        app.language = Language::Spanish;
        app.sync_combat_feedback_to_combat();
        app.dirty = true;
        app.tick(0.0);

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        let icons = frame_tinted_image_entries(&frame);
        assert_eq!(icons.len(), 6);
        assert_eq!(
            icons
                .iter()
                .filter(|entry| entry.color == TERM_GREEN_TEXT)
                .count(),
            4
        );
        assert_eq!(
            icons
                .iter()
                .filter(|entry| entry.color == TERM_CYAN)
                .count(),
            2
        );
    }

    #[test]
    fn player_panel_label_is_centered_and_seventy_five_percent_size_in_both_languages() {
        let mut app = active_combat_fixture();
        app.rebuild_frame();

        let layout = app.layout();
        let player_panel = layout.player_rect;
        let expected_label_size =
            13.5 * if layout.low_hand_layout { 1.14 } else { 1.0 } * layout.tile_scale;
        let player_metrics = player_panel_metrics(
            &app,
            layout.low_hand_layout,
            layout.tile_scale,
            layout.tile_insets,
        );
        assert!((player_metrics.label_size - expected_label_size).abs() < 0.01);

        let player_center_x = player_panel.x + player_panel.w * 0.5;
        let frame = String::from_utf8(app.frame.clone()).unwrap();
        let entries = frame_text_entries(&frame);
        let player_label = entries
            .iter()
            .find(|(_, _, _, _, _, font, text)| font == "label" && text == "Player")
            .expect("player panel should render the Player label");
        assert_eq!(player_label.3, "center");
        assert!((player_label.0 - player_center_x).abs() < 0.05);

        app.language = Language::Spanish;
        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        let entries = frame_text_entries(&frame);
        let player_label = entries
            .iter()
            .find(|(_, _, _, _, _, font, text)| font == "label" && text == "Jugador")
            .expect("player panel should render the Jugador label");
        assert_eq!(player_label.3, "center");
        assert!((player_label.0 - player_center_x).abs() < 0.05);
    }

    #[test]
    fn player_panel_stats_and_meta_lines_are_centered() {
        let mut app = active_combat_fixture();
        app.combat.player.energy = 2;
        app.combat.deck.draw_pile = vec![CardId::FlareSlash, CardId::GuardStep];
        app.combat.deck.discard_pile = vec![CardId::QuickStrike];
        app.sync_combat_feedback_to_combat();
        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        let icons = frame_tinted_image_entries(&frame);
        let texts = frame_text_entries(&frame);
        let player_panel = app.layout().player_rect;
        let layout = app.layout();
        let player_metrics = player_panel_metrics(
            &app,
            layout.low_hand_layout,
            layout.tile_scale,
            layout.tile_insets,
        );
        let player_center_x = player_panel.x + player_panel.w * 0.5;

        let heart_icon = icons
            .iter()
            .find(|entry| {
                entry.src == COMBAT_HEART_ICON_ASSET_PATH
                    && rect_contains_rect(player_panel, entry.rect)
            })
            .expect("player panel should render a heart icon");
        let stats_baseline_y = heart_icon.rect.y + heart_icon.rect.h;
        let stats_line_end = texts
            .iter()
            .filter(|(x, y, _, _, _, _, _)| {
                *x >= player_panel.x - 0.01
                    && *x <= player_panel.x + player_panel.w + 0.01
                    && *y >= player_panel.y - 0.01
                    && *y <= player_panel.y + player_panel.h + 0.01
                    && (*y - stats_baseline_y).abs() < 0.01
            })
            .map(|(x, _, size, _, _, _, text)| *x + text_width(text, *size))
            .fold(0.0, f32::max);
        let stats_center_x = (heart_icon.rect.x + stats_line_end) * 0.5;
        assert!((stats_center_x - player_center_x).abs() < 0.05);

        let energy_icon = icons
            .iter()
            .find(|entry| {
                entry.src == COMBAT_ENERGY_ICON_ASSET_PATH
                    && rect_contains_rect(player_panel, entry.rect)
            })
            .expect("player panel should render an energy icon");
        let meta_center_x =
            energy_icon.rect.x + combat_meta_line_width(&app, player_metrics.meta_size) * 0.5;
        assert!((meta_center_x - player_center_x).abs() < 0.05);
    }

    #[test]
    fn player_panel_status_row_is_centered_when_present() {
        let mut app = active_combat_fixture();
        app.combat.player.fighter.statuses.focus = 1;
        app.combat.player.fighter.statuses.rhythm = -1;
        app.sync_combat_feedback_to_combat();
        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        let texts = frame_text_entries(&frame);
        let player_panel = app.layout().player_rect;
        let player_center_x = player_panel.x + player_panel.w * 0.5;
        let status_entries = texts
            .iter()
            .filter(|(x, y, _, _, _, _, text)| {
                *x >= player_panel.x - 0.01
                    && *x <= player_panel.x + player_panel.w + 0.01
                    && *y >= player_panel.y - 0.01
                    && *y <= player_panel.y + player_panel.h + 0.01
                    && (text == "Focus+1" || text == "Rhythm-1")
            })
            .collect::<Vec<_>>();

        assert_eq!(status_entries.len(), 2);
        let row_start = status_entries
            .iter()
            .map(|(x, _, _, _, _, _, _)| *x)
            .fold(f32::INFINITY, f32::min);
        let row_end = status_entries
            .iter()
            .map(|(x, _, size, _, _, _, text)| *x + text_width(text, *size))
            .fold(0.0, f32::max);
        let row_center_x = (row_start + row_end) * 0.5;
        assert!((row_center_x - player_center_x).abs() < 0.05);
    }

    #[test]
    fn player_panel_long_status_label_is_centered() {
        let mut app = active_combat_fixture();
        app.combat.player.fighter.statuses.momentum = 1;
        app.sync_combat_feedback_to_combat();
        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        let texts = frame_text_entries(&frame);
        let player_panel = app.layout().player_rect;
        let player_center_x = player_panel.x + player_panel.w * 0.5;
        let momentum_entry = texts
            .iter()
            .find(|(x, y, _, _, _, _, text)| {
                *x >= player_panel.x - 0.01
                    && *x <= player_panel.x + player_panel.w + 0.01
                    && *y >= player_panel.y - 0.01
                    && *y <= player_panel.y + player_panel.h + 0.01
                    && text == "Momentum+1"
            })
            .expect("player panel should render Momentum+1");

        let label_start = momentum_entry.0;
        let label_end = momentum_entry.0 + text_width(&momentum_entry.6, momentum_entry.2);
        let label_center_x = (label_start + label_end) * 0.5;
        assert!((label_center_x - player_center_x).abs() < 0.05);
    }

    #[test]
    fn player_panel_status_row_wraps_after_two_labels() {
        let mut app = active_combat_fixture();
        app.combat.player.fighter.statuses.focus = 1;
        app.combat.player.fighter.statuses.rhythm = -1;
        app.combat.player.fighter.statuses.momentum = 1;
        app.combat.player.fighter.statuses.bleed = 2;
        app.sync_combat_feedback_to_combat();
        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        let texts = frame_text_entries(&frame);
        let player_panel = app.layout().player_rect;
        let mut status_entries = texts
            .iter()
            .filter(|(x, y, _, _, _, _, text)| {
                *x >= player_panel.x - 0.01
                    && *x <= player_panel.x + player_panel.w + 0.01
                    && *y >= player_panel.y - 0.01
                    && *y <= player_panel.y + player_panel.h + 0.01
                    && matches!(
                        text.as_str(),
                        "Focus+1" | "Rhythm-1" | "Momentum+1" | "Bleed 2"
                    )
            })
            .collect::<Vec<_>>();

        assert_eq!(status_entries.len(), 4);
        status_entries.sort_by(|a, b| a.1.total_cmp(&b.1).then_with(|| a.0.total_cmp(&b.0)));

        let mut rows: Vec<Vec<&FrameTextEntry>> = Vec::new();
        for entry in status_entries {
            if let Some(row) = rows
                .iter_mut()
                .find(|row| (row[0].1 - entry.1).abs() < 0.01)
            {
                row.push(entry);
            } else {
                rows.push(vec![entry]);
            }
        }

        assert_eq!(rows.len(), 2);
        assert!(rows.iter().all(|row| row.len() <= STATUS_ROW_MAX_COLUMNS));
        assert!(rows.iter().all(|row| row.len() == 2));
        assert!(rows[1][0].1 > rows[0][0].1);
    }

    #[test]
    fn player_panel_does_not_render_playback_meta_text_and_keeps_meta_icons() {
        let mut app = active_combat_fixture();
        app.combat_feedback.playback_kind = Some(CombatPlaybackKind::EnemyTurn);
        app.combat.player.energy = 2;
        app.combat.player.max_energy = 3;
        app.combat.deck.draw_pile = vec![CardId::FlareSlash, CardId::GuardStep];
        app.combat.deck.discard_pile = vec![CardId::QuickStrike];
        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        let player_panel = app.layout().player_rect;
        let panel_texts = frame_text_entries(&frame);
        assert!(panel_texts.iter().all(|(x, y, _, _, _, _, text)| {
            !(*x >= player_panel.x - 0.01
                && *x <= player_panel.x + player_panel.w + 0.01
                && *y >= player_panel.y - 0.01
                && *y <= player_panel.y + player_panel.h + 0.01
                && text == "Enemy turn resolving...")
        }));

        let panel_icons = frame_tinted_image_entries(&frame);
        assert!(panel_icons.iter().any(|entry| {
            entry.src == COMBAT_ENERGY_ICON_ASSET_PATH
                && rect_contains_rect(player_panel, entry.rect)
        }));
        assert!(panel_icons.iter().any(|entry| {
            entry.src == COMBAT_DECK_ICON_ASSET_PATH && rect_contains_rect(player_panel, entry.rect)
        }));
    }

    #[test]
    fn combat_panels_do_not_render_empty_status_placeholder_text() {
        let mut app = active_combat_fixture();

        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        assert!(!frame.contains("No active effects"));
        assert!(!frame.contains("Sin efectos activos"));

        app.language = Language::Spanish;
        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        assert!(!frame.contains("No active effects"));
        assert!(!frame.contains("Sin efectos activos"));
    }

    #[test]
    fn combat_panels_collapse_empty_status_rows_and_move_following_content_up() {
        let mut app = active_combat_fixture();
        app.combat.player.energy = 2;
        app.combat.deck.draw_pile = vec![CardId::FlareSlash, CardId::GuardStep];
        app.combat.deck.discard_pile = vec![CardId::QuickStrike];
        app.sync_combat_feedback_to_combat();
        let tile_insets = tile_insets_for_card_width(CARD_WIDTH);

        let player_metrics_without = player_panel_metrics(&app, false, 1.0, tile_insets);
        let enemy_metrics_without = enemy_panel_metrics(&app, 0, false, 1.0, tile_insets);
        assert_eq!(player_metrics_without.status_row_height, 0.0);
        assert_eq!(enemy_metrics_without.status_row_height, 0.0);

        app.rebuild_frame();
        let frame_without = String::from_utf8(app.frame.clone()).unwrap();
        let player_panel_y_without = app.layout().player_rect.y;
        let player_meta_y_without = frame_tinted_image_entries(&frame_without)
            .into_iter()
            .find(|entry| entry.src == COMBAT_ENERGY_ICON_ASSET_PATH)
            .map(|entry| entry.rect.y)
            .unwrap();

        app.combat.player.fighter.statuses.focus = -1;
        primary_enemy_mut(&mut app.combat).fighter.statuses.rhythm = -1;

        let player_metrics_with = player_panel_metrics(&app, false, 1.0, tile_insets);
        let enemy_metrics_with = enemy_panel_metrics(&app, 0, false, 1.0, tile_insets);
        assert_eq!(
            player_metrics_with.status_row_height,
            player_metrics_with.status_size
        );
        assert_eq!(
            enemy_metrics_with.status_row_height,
            enemy_metrics_with.status_size
        );
        assert!(
            (player_metrics_with.height
                - player_metrics_without.height
                - (player_metrics_with.status_size + player_metrics_with.line_gap))
                .abs()
                < 0.1
        );
        assert!(
            (enemy_metrics_with.height
                - enemy_metrics_without.height
                - (enemy_metrics_with.status_size + enemy_metrics_with.line_gap))
                .abs()
                < 0.1
        );

        app.rebuild_frame();
        let frame_with = String::from_utf8(app.frame.clone()).unwrap();
        let player_panel_y_with = app.layout().player_rect.y;
        let player_meta_y_with = frame_tinted_image_entries(&frame_with)
            .into_iter()
            .find(|entry| entry.src == COMBAT_ENERGY_ICON_ASSET_PATH)
            .map(|entry| entry.rect.y)
            .unwrap();

        assert!(
            player_meta_y_without - player_panel_y_without
                < player_meta_y_with - player_panel_y_with
        );
    }

    #[test]
    fn enemy_turn_playback_collapses_the_hidden_hand_gap() {
        let mut app = active_combat_fixture();
        let combat_layout = app.layout();
        let baseline_enemy_rect = primary_enemy_rect(&combat_layout);
        let baseline_gap =
            combat_layout.player_rect.y - (baseline_enemy_rect.y + baseline_enemy_rect.h);

        app.perform_action(CombatAction::EndTurn);
        advance_until(&mut app, |app| {
            app.combat_feedback.playback_kind == Some(CombatPlaybackKind::EnemyTurn)
                && app.layout().hand_rects.is_empty()
                && app.layout_transition.is_none()
        });

        let collapsed_layout = app.layout();
        let collapsed_enemy_rect = primary_enemy_rect(&collapsed_layout);
        let collapsed_gap =
            collapsed_layout.player_rect.y - (collapsed_enemy_rect.y + collapsed_enemy_rect.h);

        assert!(collapsed_gap < baseline_gap);
        assert!(collapsed_layout.hand_rects.is_empty());
    }

    #[test]
    fn enemy_turn_completion_starts_hand_reveal_transition() {
        let mut app = active_combat_fixture();

        app.perform_action(CombatAction::EndTurn);
        advance_until(&mut app, |app| {
            app.combat_feedback.playback_kind.is_none() && app.layout_transition.is_some()
        });

        let transition = app.layout_transition.as_ref().unwrap();
        assert!(transition.from_layout.hand_rects.is_empty());
        assert_eq!(transition.hand_from_rects.len(), app.combat.hand_len());
    }

    #[test]
    fn shield_gain_renders_green_count_up_into_frame() {
        let mut app = active_combat_fixture();
        app.combat.player.energy = 3;
        app.combat.deck.hand = vec![CardId::GuardStep];
        app.combat.deck.draw_pile.clear();
        app.combat.deck.discard_pile.clear();
        app.combat.player.fighter.block = 0;
        app.sync_combat_feedback_to_combat();

        app.perform_action(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(Actor::Player),
        });
        advance_until(&mut app, |app| {
            app.combat_feedback
                .active_stats
                .iter()
                .any(|active| active.actor == Actor::Player && active.stat == CombatStat::Block)
                && app.combat_feedback.displayed.player.block > 0
        });

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        assert!(frame.contains(&format!(
            "|left|{}|body|{}",
            TERM_GREEN_SOFT, app.combat_feedback.displayed.player.block
        )));
    }

    #[test]
    fn defeat_waits_for_playback_before_showing_result() {
        let mut app = active_combat_fixture();
        app.combat.player.fighter.hp = 4;
        app.combat.player.fighter.block = 0;
        set_primary_enemy_intent(&mut app, EnemyProfileId::NeedlerDrone, 0);
        app.sync_combat_feedback_to_combat();
        let snapshot_before = app.run_save_snapshot.clone();

        app.perform_action(CombatAction::EndTurn);

        assert!(matches!(app.screen, AppScreen::Combat));
        assert_eq!(
            app.combat_feedback.pending_outcome,
            Some(CombatOutcome::Defeat)
        );
        assert_eq!(app.run_save_snapshot, snapshot_before);

        advance_until(&mut app, |app| {
            matches!(app.screen, AppScreen::Result(CombatOutcome::Defeat))
        });
    }

    #[test]
    fn defeated_front_enemy_explodes_and_disappears_in_two_enemy_combat() {
        let mut app = active_combat_fixture();
        let mut setup = crate::combat::EncounterSetup {
            player_hp: 32,
            player_max_hp: 32,
            player_max_energy: 3,
            enemies: Vec::new(),
        };
        setup.enemies.push(crate::combat::EncounterEnemySetup {
            hp: 6,
            max_hp: 6,
            block: 0,
            profile: EnemyProfileId::ScoutDrone,
            intent_index: 0,
            on_hit_bleed: 0,
        });
        setup.enemies.push(crate::combat::EncounterEnemySetup {
            hp: 14,
            max_hp: 14,
            block: 0,
            profile: EnemyProfileId::NeedlerDrone,
            intent_index: 1,
            on_hit_bleed: 0,
        });
        app.begin_encounter(setup);
        app.screen_transition = None;
        app.combat.player.energy = 3;
        app.combat.deck.hand = vec![CardId::FlareSlash];
        app.combat.deck.draw_pile.clear();
        app.combat.deck.discard_pile.clear();
        app.sync_combat_feedback_to_combat();

        assert_eq!(app.layout().enemy_indices, vec![0, 1]);

        app.perform_action(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(Actor::Enemy(0)),
        });

        assert_eq!(app.layout().enemy_indices, vec![0, 1]);
        assert!(app.layout().enemy_rect(0).is_some());
        assert!(app.layout().enemy_rect(1).is_some());

        advance_until(&mut app, |app| app.layout().enemy_indices == vec![1]);

        assert_eq!(app.layout().enemy_indices, vec![1]);
        assert!(app.layout().enemy_rect(0).is_none());
        assert!(app.layout().enemy_rect(1).is_some());
        assert!(!app.pixel_shards.is_empty());

        app.rebuild_frame();
        let frame = String::from_utf8(app.frame.clone()).unwrap();
        let sprites = frame_sprite_entries(&frame);
        let expected_codes = enemy_sprite_codes(EnemyProfileId::NeedlerDrone);

        assert_eq!(sprites.len(), expected_codes.len());
        assert_eq!(
            sprites.iter().map(|(code, _)| *code).collect::<Vec<_>>(),
            expected_codes
        );
    }

    #[test]
    fn two_enemy_combat_emits_a_sprite_for_each_visible_enemy_panel() {
        let mut app = active_combat_fixture();
        let mut setup = crate::combat::EncounterSetup {
            player_hp: 32,
            player_max_hp: 32,
            player_max_energy: 3,
            enemies: Vec::new(),
        };
        setup.enemies.push(crate::combat::EncounterEnemySetup {
            hp: 6,
            max_hp: 6,
            block: 0,
            profile: EnemyProfileId::ScoutDrone,
            intent_index: 0,
            on_hit_bleed: 0,
        });
        setup.enemies.push(crate::combat::EncounterEnemySetup {
            hp: 14,
            max_hp: 14,
            block: 0,
            profile: EnemyProfileId::NeedlerDrone,
            intent_index: 1,
            on_hit_bleed: 0,
        });
        app.begin_encounter(setup);
        app.screen_transition = None;
        app.combat.player.energy = 3;
        app.combat.deck.hand = vec![CardId::FlareSlash];
        app.combat.deck.draw_pile.clear();
        app.combat.deck.discard_pile.clear();
        app.sync_combat_feedback_to_combat();

        app.rebuild_frame();
        let frame = String::from_utf8(app.frame.clone()).unwrap();
        let sprites = frame_sprite_entries(&frame);
        let expected_codes = enemy_sprite_codes(EnemyProfileId::ScoutDrone)
            .into_iter()
            .chain(enemy_sprite_codes(EnemyProfileId::NeedlerDrone))
            .collect::<Vec<_>>();

        assert_eq!(sprites.len(), expected_codes.len());
        assert_eq!(
            sprites.iter().map(|(code, _)| *code).collect::<Vec<_>>(),
            expected_codes
        );

        app.perform_action(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(Actor::Enemy(0)),
        });
        advance_until(&mut app, |app| app.layout().enemy_indices == vec![1]);

        app.rebuild_frame();
        let frame = String::from_utf8(app.frame.clone()).unwrap();
        let sprites = frame_sprite_entries(&frame);
        let expected_codes = enemy_sprite_codes(EnemyProfileId::NeedlerDrone);

        assert_eq!(sprites.len(), expected_codes.len());
        assert_eq!(
            sprites.iter().map(|(code, _)| *code).collect::<Vec<_>>(),
            expected_codes
        );
    }

    #[test]
    fn continuing_run_does_not_restore_a_defeated_enemy_panel() {
        let mut app = active_combat_fixture();
        let mut setup = crate::combat::EncounterSetup {
            player_hp: 32,
            player_max_hp: 32,
            player_max_energy: 3,
            enemies: Vec::new(),
        };
        setup.enemies.push(crate::combat::EncounterEnemySetup {
            hp: 6,
            max_hp: 6,
            block: 0,
            profile: EnemyProfileId::ScoutDrone,
            intent_index: 0,
            on_hit_bleed: 0,
        });
        setup.enemies.push(crate::combat::EncounterEnemySetup {
            hp: 14,
            max_hp: 14,
            block: 0,
            profile: EnemyProfileId::NeedlerDrone,
            intent_index: 1,
            on_hit_bleed: 0,
        });
        app.begin_encounter(setup);
        app.screen_transition = None;
        app.combat.player.energy = 3;
        app.combat.deck.hand = vec![CardId::FlareSlash];
        app.combat.deck.draw_pile.clear();
        app.combat.deck.discard_pile.clear();
        app.sync_combat_feedback_to_combat();

        app.perform_action(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(Actor::Enemy(0)),
        });
        advance_until(&mut app, |app| app.layout().enemy_indices == vec![1]);

        let restored = restore_app_from_snapshot(&app.serialize_current_run().unwrap());

        assert!(matches!(restored.screen, AppScreen::Combat));
        assert_eq!(restored.layout().enemy_indices, vec![1]);
        assert!(restored.layout().enemy_rect(0).is_none());
        assert!(restored.layout().enemy_rect(1).is_some());
    }

    #[test]
    fn enemy_pixel_burst_matches_card_pixel_burst() {
        let rect = Rect {
            x: 120.0,
            y: 80.0,
            w: 96.0,
            h: 140.0,
        };
        let mut card_app = App::new();
        let mut enemy_app = App::new();

        card_app.spawn_card_pixel_burst(rect, CardId::FlareSlash);
        enemy_app.spawn_enemy_pixel_burst(rect);

        assert_eq!(card_app.pixel_shards.len(), enemy_app.pixel_shards.len());
        for (card_shard, enemy_shard) in card_app.pixel_shards.iter().zip(&enemy_app.pixel_shards) {
            assert_eq!(card_shard.x, enemy_shard.x);
            assert_eq!(card_shard.y, enemy_shard.y);
            assert_eq!(card_shard.vx, enemy_shard.vx);
            assert_eq!(card_shard.vy, enemy_shard.vy);
            assert_eq!(card_shard.size, enemy_shard.size);
            assert_eq!(card_shard.ttl_ms, enemy_shard.ttl_ms);
            assert_eq!(card_shard.total_ms, enemy_shard.total_ms);
            assert_eq!(card_shard.color, enemy_shard.color);
        }
    }

    #[test]
    fn victory_waits_for_lethal_enemy_vfx_before_resolving() {
        let mut app = active_combat_fixture();
        app.combat.player.energy = 3;
        app.combat.deck.hand = vec![CardId::FlareSlash, CardId::GuardStep];
        app.combat.deck.draw_pile.clear();
        app.combat.deck.discard_pile.clear();
        primary_enemy_mut(&mut app.combat).fighter.hp = 6;
        primary_enemy_mut(&mut app.combat).fighter.block = 0;
        app.sync_combat_feedback_to_combat();
        let snapshot_before = app.run_save_snapshot.clone();

        app.perform_action(CombatAction::PlayCard {
            hand_index: 0,
            target: Some(Actor::Enemy(0)),
        });

        assert!(matches!(app.screen, AppScreen::Combat));
        assert_eq!(
            app.combat_feedback.pending_outcome,
            Some(CombatOutcome::Victory)
        );
        assert_eq!(app.run_save_snapshot, snapshot_before);
        assert!(app.layout_transition.is_some());
        assert_eq!(app.combat.hand_len(), 1);

        advance_until(&mut app, |app| {
            app.layout().enemy_indices.is_empty() && !app.pixel_shards.is_empty()
        });
        assert!(matches!(app.screen, AppScreen::Combat));
        assert!(app.layout().enemy_indices.is_empty());
        assert!(app.layout_transition.is_some());
        app.rebuild_frame();
        let frame = String::from_utf8(app.frame.clone()).unwrap();
        assert!(!frame.contains(localized_enemy_name(
            EnemyProfileId::ScoutDrone,
            Language::English
        )));

        advance_until(&mut app, |app| {
            !app.combat_feedback.auto_playback_active
                && app.combat_feedback.outcome_hold_ms > 0.0
                && !app.pixel_shards.is_empty()
        });
        assert!(matches!(app.screen, AppScreen::Combat));
        assert_eq!(
            app.combat_feedback.pending_outcome,
            Some(CombatOutcome::Victory)
        );

        advance_until(&mut app, |app| !matches!(app.screen, AppScreen::Combat));
        assert!(matches!(
            app.screen,
            AppScreen::Reward | AppScreen::Result(CombatOutcome::Victory)
        ));
    }

    #[test]
    fn returning_to_menu_keeps_continue_available() {
        let mut app = active_module_select_fixture();
        let saved_snapshot = app.run_save_snapshot.clone();

        app.return_to_menu();

        assert!(matches!(app.screen, AppScreen::Boot));
        assert_eq!(app.run_save_snapshot, saved_snapshot);
        assert!(app.has_saved_run);
        let start_button = app.layout().start_button;
        assert_eq!(
            app.hit_test(
                start_button.x + start_button.w * 0.5,
                start_button.y + start_button.h * 0.5
            ),
            Some(HitTarget::Continue)
        );
    }

    #[test]
    fn starting_new_run_does_not_flash_continue_or_restart_during_transition() {
        let mut app = App::new();

        app.start_run();
        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        assert!(frame.contains("|label|Start"));
        assert!(!frame.contains("|label|Continue"));
        assert!(!frame.contains("|label|Restart"));
    }

    #[test]
    fn boot_renders_version_at_the_bottom() {
        let mut app = App::new();
        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        let version_line = visible_game_version_label();
        let version_y = boot_version_baseline_y(app.logical_height());
        let version_size = boot_version_font_size(app.logical_width());

        assert!(frame.contains(&format!(
            "TEXT|{:.2}|{:.2}|{:.2}|center|{}|body|{}",
            app.logical_center_x(),
            version_y,
            version_size,
            TERM_GREEN_DIM,
            version_line
        )));
    }

    #[test]
    fn stable_version_label_uses_semver() {
        assert_eq!(
            format_visible_game_version_label(
                "stable",
                "0.1.0",
                Some(TEST_BUILD_TIMESTAMP),
                Some(TEST_BUILD_SHA)
            ),
            "v0.1.0"
        );
    }

    #[test]
    fn preview_version_label_uses_build_metadata() {
        assert_eq!(
            format_visible_game_version_label(
                "preview",
                "0.1.0",
                Some(TEST_BUILD_TIMESTAMP),
                Some(TEST_BUILD_SHA)
            ),
            "preview BUILD_TS BUILD_SHA"
        );
    }

    #[test]
    fn preview_version_label_handles_missing_metadata() {
        assert_eq!(
            format_visible_game_version_label("preview", "0.1.0", Some(TEST_BUILD_TIMESTAMP), None),
            "preview BUILD_TS"
        );
        assert_eq!(
            format_visible_game_version_label("preview", "0.1.0", None, Some(TEST_BUILD_SHA)),
            "preview BUILD_SHA"
        );
        assert_eq!(
            format_visible_game_version_label("preview", "0.1.0", None, None),
            "preview"
        );
    }

    #[test]
    fn boot_install_button_is_hidden_when_install_is_unavailable() {
        let app = App::new();

        assert!(app.boot_buttons_layout(false).install_button.is_none());
    }

    #[test]
    fn boot_update_button_is_hidden_when_no_update_is_available() {
        let app = App::new();

        assert!(app.boot_buttons_layout(false).update_button.is_none());
    }

    #[test]
    fn boot_install_button_appears_below_settings_when_available() {
        let mut app = App::new();
        app.set_install_capability(InstallCapability::PromptAvailable);

        let buttons = app.boot_buttons_layout(false);
        let install_button = buttons.install_button.unwrap();

        assert!(install_button.y >= buttons.settings_button.y + buttons.settings_button.h);
    }

    #[test]
    fn boot_install_button_queues_native_install_request() {
        let mut app = App::new();
        app.set_install_capability(InstallCapability::PromptAvailable);
        let install_button = app.boot_buttons_layout(false).install_button.unwrap();

        app.handle_boot_pointer(
            install_button.x + install_button.w * 0.5,
            install_button.y + install_button.h * 0.5,
        );

        assert!(app.install_request_pending);
        assert!(!app.ui.install_help_open);
    }

    #[test]
    fn boot_update_button_appears_below_settings_when_available_without_install() {
        let mut app = App::new();
        app.set_update_available(true);

        let buttons = app.boot_buttons_layout(false);
        let update_button = buttons.update_button.unwrap();

        assert!(update_button.y >= buttons.settings_button.y + buttons.settings_button.h);
        assert!(buttons.install_button.is_none());
    }

    #[test]
    fn boot_update_button_appears_below_install_when_both_are_visible() {
        let mut app = App::new();
        app.set_update_available(true);
        app.set_install_capability(InstallCapability::PromptAvailable);

        let buttons = app.boot_buttons_layout(false);
        let install_button = buttons.install_button.unwrap();
        let update_button = buttons.update_button.unwrap();

        assert!(update_button.y >= install_button.y + install_button.h);
    }

    #[test]
    fn boot_update_button_queues_update_request() {
        let mut app = App::new();
        app.set_update_available(true);
        let update_button = app.boot_buttons_layout(false).update_button.unwrap();

        app.handle_boot_pointer(
            update_button.x + update_button.w * 0.5,
            update_button.y + update_button.h * 0.5,
        );

        assert!(app.update_request_pending);
        assert!(!app.install_request_pending);
    }

    #[test]
    fn boot_update_hotkey_queues_update_request() {
        let mut app = App::new();
        app.set_update_available(true);

        app.key_down(85);

        assert!(app.update_request_pending);
    }

    #[test]
    fn boot_update_request_clears_when_update_becomes_unavailable() {
        let mut app = App::new();
        app.set_update_available(true);
        app.request_update();
        assert!(app.update_request_pending);

        app.set_update_available(false);

        assert!(!app.update_request_pending);
    }

    #[test]
    fn start_run_clears_boot_request_flags() {
        let mut app = App::new();
        app.resume_request_pending = true;
        app.install_request_pending = true;
        app.update_request_pending = true;

        app.start_run();

        assert!(!app.resume_request_pending);
        assert!(!app.install_request_pending);
        assert!(!app.update_request_pending);
    }

    #[test]
    fn boot_install_button_opens_ios_install_help_modal() {
        let mut app = App::new();
        app.set_install_capability(InstallCapability::IosGuide);
        let install_button = app.boot_buttons_layout(false).install_button.unwrap();

        app.handle_boot_pointer(
            install_button.x + install_button.w * 0.5,
            install_button.y + install_button.h * 0.5,
        );

        assert!(app.ui.install_help_open);
        assert!(!app.install_request_pending);
    }

    #[test]
    fn boot_screen_flag_tracks_return_to_menu() {
        let mut app = App::new();
        assert!(app.is_boot_screen());

        app.start_run();
        assert!(!app.is_boot_screen());

        app.return_to_menu();
        assert!(app.is_boot_screen());
    }

    #[test]
    fn install_help_modal_closes_on_escape() {
        let mut app = App::new();
        app.set_install_capability(InstallCapability::IosGuide);
        app.open_install_help();
        assert!(app.ui.install_help_open);

        app.key_down(27);

        assert!(!app.ui.install_help_open);
    }

    #[test]
    fn boot_restart_modal_cancels_and_confirms_restart() {
        let mut app = App::new();
        app.start_run();
        skip_opening_intro(&mut app);
        app.return_to_menu();
        app.screen_transition = None;
        let restart_button = app.layout().restart_button;

        assert_eq!(
            app.hit_test(
                restart_button.x + restart_button.w * 0.5,
                restart_button.y + restart_button.h * 0.5,
            ),
            Some(HitTarget::Restart)
        );
        app.open_restart_confirm();
        assert!(app.ui.restart_confirm_open);

        app.key_down(27);
        assert!(!app.ui.restart_confirm_open);

        app.open_restart_confirm();
        app.key_down(13);

        assert!(matches!(app.screen, AppScreen::Boot));
        assert!(app.run_save_snapshot.is_none());
        assert!(!app.has_saved_run);
        let start_button = app.layout().start_button;
        assert_eq!(
            app.hit_test(
                start_button.x + start_button.w * 0.5,
                start_button.y + start_button.h * 0.5,
            ),
            Some(HitTarget::Start)
        );
    }

    #[test]
    fn debug_boot_clear_save_button_removes_saved_run() {
        let mut app = App::new();
        app.debug_mode = true;
        app.start_run();
        skip_opening_intro(&mut app);
        app.return_to_menu();
        app.screen_transition = None;

        let clear_save_button = app
            .layout()
            .clear_save_button
            .expect("debug clear save button should exist");

        assert_eq!(
            app.hit_test(
                clear_save_button.x + clear_save_button.w * 0.5,
                clear_save_button.y + clear_save_button.h * 0.5,
            ),
            Some(HitTarget::DebugClearSave)
        );

        app.debug_clear_saved_run();

        assert!(app.run_save_snapshot.is_none());
        assert!(!app.has_saved_run);
        let start_button = app.layout().start_button;
        assert_eq!(
            app.hit_test(
                start_button.x + start_button.w * 0.5,
                start_button.y + start_button.h * 0.5,
            ),
            Some(HitTarget::Start)
        );
    }

    #[test]
    fn result_screen_clears_saved_run_snapshot() {
        let mut app = App::new();
        app.start_run();
        skip_opening_intro(&mut app);

        app.screen = AppScreen::Result(CombatOutcome::Defeat);
        app.refresh_run_save_snapshot();

        assert!(app.run_save_snapshot.is_none());
        assert!(!app.has_saved_run);
    }

    #[test]
    fn level_intro_continue_returns_to_map() {
        let mut app = App::new();
        app.screen = AppScreen::LevelIntro;
        app.level_intro = Some(LevelIntroState {
            level: 2,
            codename: "Fracture Span",
            summary: "Sharper pressure and exposed openings set up the Hexarch Core's burst turns.",
        });

        app.continue_from_level_intro();

        assert!(matches!(app.screen, AppScreen::Map));
    }

    #[test]
    fn level_intro_continue_button_uses_standard_bottom_margin() {
        let mut app = App::new();
        app.screen = AppScreen::LevelIntro;
        app.level_intro = Some(LevelIntroState {
            level: 2,
            codename: "Fracture Span",
            summary: "Sharper pressure and exposed openings set up the Hexarch Core's burst turns.",
        });

        let button = app.level_intro_continue_button_rect();
        let (_, pad_y) = boot_button_tile_padding();

        assert!(
            (LOGICAL_HEIGHT - (button.y + button.h) - pad_y).abs() < 0.1,
            "expected standard bottom margin for level intro continue button"
        );
    }

    #[test]
    fn debug_map_level_buttons_shift_the_current_level() {
        let mut app = App::new();
        app.screen = AppScreen::Map;
        app.debug_mode = true;
        app.dungeon = Some(DungeonRun::new(TEST_RUN_SEED));

        app.rebuild_frame();
        let frame = String::from_utf8(app.frame.clone()).unwrap();
        let expected_label = debug_map_label(app.dungeon.as_ref().unwrap(), Language::English);
        assert!(frame.contains(&format!("|body|{expected_label}")));

        let layout = app.map_layout().unwrap();
        let up = layout.debug_level_up_button.unwrap();
        let (debug_text_x, _) = layout.debug_level_text_position.unwrap();
        assert!(up.x > debug_text_x);
        assert!(up.y >= layout.legend_button.y + layout.legend_button.h + HAND_MIN_GAP - 0.01);
        app.handle_map_pointer(up.x + up.w * 0.5, up.y + up.h * 0.5);
        assert_eq!(app.dungeon.as_ref().unwrap().current_level(), 2);

        let layout = app.map_layout().unwrap();
        let down = layout.debug_level_down_button.unwrap();
        let (debug_text_x, _) = layout.debug_level_text_position.unwrap();
        assert!(down.x + down.w < debug_text_x);
        assert!(down.y >= layout.legend_button.y + layout.legend_button.h + HAND_MIN_GAP - 0.01);
        app.handle_map_pointer(down.x + down.w * 0.5, down.y + down.h * 0.5);
        assert_eq!(app.dungeon.as_ref().unwrap().current_level(), 1);
    }

    #[test]
    fn debug_map_fill_deck_button_renders_below_level_controls() {
        let mut app = App::new();
        app.screen = AppScreen::Map;
        app.debug_mode = true;
        app.dungeon = Some(DungeonRun::new(TEST_RUN_SEED));

        app.rebuild_frame();
        let frame = String::from_utf8(app.frame.clone()).unwrap();
        assert!(frame.contains("|label|Fill Deck"));

        let layout = app.map_layout().unwrap();
        let up = layout.debug_level_up_button.unwrap();
        let fill = layout.debug_fill_deck_button.unwrap();
        assert!(fill.y >= up.y + up.h + HAND_MIN_GAP - 0.01);
        assert!(fill.y >= layout.legend_button.y + layout.legend_button.h + HAND_MIN_GAP - 0.01);
        assert_eq!(
            app.hit_test(fill.x + fill.w * 0.5, fill.y + fill.h * 0.5),
            Some(HitTarget::DebugFillDeck)
        );
    }

    #[test]
    fn debug_fill_deck_appends_missing_base_cards_and_updates_run_save() {
        let mut app = App::new();
        app.screen = AppScreen::Map;
        app.debug_mode = true;
        app.language = Language::Spanish;
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.deck = vec![
            CardId::GuardStep,
            CardId::FlareSlashPlus,
            CardId::AnchorLoop,
        ];
        app.dungeon = Some(dungeon);
        app.refresh_run_save_snapshot();

        let before_snapshot = app.run_save_snapshot.clone();
        let before_deck = app.dungeon.as_ref().unwrap().deck.clone();
        let expected_missing: Vec<CardId> = all_base_cards()
            .iter()
            .copied()
            .filter(|card| !before_deck.contains(card))
            .collect();

        app.debug_fill_deck();

        let deck = &app.dungeon.as_ref().unwrap().deck;
        assert_eq!(&deck[..before_deck.len()], before_deck.as_slice());
        assert_eq!(&deck[before_deck.len()..], expected_missing.as_slice());
        assert_ne!(app.run_save_snapshot, before_snapshot);
        let expected_log = format!("Se llenó el mazo con {} cartas.", expected_missing.len());
        assert_eq!(
            app.log.back().map(String::as_str),
            Some(expected_log.as_str())
        );
    }

    #[test]
    fn debug_fill_deck_is_idempotent_once_all_base_cards_are_present() {
        let mut app = App::new();
        app.screen = AppScreen::Map;
        app.debug_mode = true;
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.deck = vec![CardId::GuardStep];
        app.dungeon = Some(dungeon);

        app.debug_fill_deck();
        let filled_deck = app.dungeon.as_ref().unwrap().deck.clone();

        app.debug_fill_deck();

        assert_eq!(app.dungeon.as_ref().unwrap().deck, filled_deck);
        assert_eq!(
            app.log.back().map(String::as_str),
            Some("Deck already contains all base cards.")
        );
    }

    #[test]
    fn combat_victory_awards_credits_and_logs_the_gain() {
        let mut app = active_combat_fixture();
        app.debug_mode = true;

        app.debug_end_battle();

        assert!(matches!(
            app.screen,
            AppScreen::Result(CombatOutcome::Victory)
        ));
        assert_eq!(app.dungeon.as_ref().unwrap().credits, 6);
        assert!(app.log.iter().any(|entry| entry == "Gained 6 Credits."));
    }

    #[test]
    fn boss_victory_opens_reward_before_level_intro() {
        let mut app = App::new();
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.nodes = vec![
            crate::dungeon::DungeonNode {
                id: 0,
                depth: 0,
                lane: 3,
                kind: RoomKind::Start,
                next: vec![1],
            },
            crate::dungeon::DungeonNode {
                id: 1,
                depth: 1,
                lane: 3,
                kind: RoomKind::Boss,
                next: vec![],
            },
        ];
        dungeon.current_node = Some(1);
        dungeon.available_nodes.clear();
        app.dungeon = Some(dungeon);
        app.screen = AppScreen::Combat;
        app.debug_mode = true;

        app.debug_end_battle();

        assert!(matches!(app.screen, AppScreen::Reward));
        assert!(app.reward.is_some());
        assert!(app.level_intro.is_none());
    }

    #[test]
    fn boss_reward_claim_opens_module_select_before_level_intro() {
        let mut app = active_boss_module_select_fixture(1);
        let initial_deck_len = app.dungeon.as_ref().unwrap().deck.len();

        app.claim_reward(0);

        assert!(matches!(app.screen, AppScreen::ModuleSelect));
        assert_eq!(
            app.dungeon.as_ref().unwrap().deck.len(),
            initial_deck_len + 1
        );
        let module_select = app.module_select.as_ref().unwrap();
        assert_eq!(module_select.options, boss_module_choices(1));
        assert_eq!(
            module_select.context,
            ModuleSelectContext::BossReward { boss_level: 1 }
        );
    }

    #[test]
    fn boss_reward_skip_opens_module_select_before_level_intro() {
        let mut app = active_boss_module_select_fixture(1);
        let initial_deck_len = app.dungeon.as_ref().unwrap().deck.len();

        app.skip_reward();

        assert!(matches!(app.screen, AppScreen::ModuleSelect));
        assert_eq!(app.dungeon.as_ref().unwrap().deck.len(), initial_deck_len);
        let module_select = app.module_select.as_ref().unwrap();
        assert_eq!(module_select.options, boss_module_choices(1));
        assert_eq!(
            module_select.context,
            ModuleSelectContext::BossReward { boss_level: 1 }
        );
    }

    #[test]
    fn boss_module_claim_opens_level_intro_and_adds_the_module() {
        let mut app = active_boss_module_select_fixture(1);
        app.skip_reward();
        let expected = app.module_select.as_ref().unwrap().options[2];

        app.claim_module_select(2);

        assert!(matches!(app.screen, AppScreen::LevelIntro));
        assert!(app.dungeon.as_ref().unwrap().modules.contains(&expected));
        let level_intro = app.level_intro.as_ref().unwrap();
        assert_eq!(level_intro.level, 2);
        assert_eq!(level_intro.codename, "Fracture Span");
    }

    #[test]
    fn second_boss_reward_offers_three_unique_modules() {
        let mut app = active_boss_module_select_fixture(2);
        app.skip_reward();
        let module_select = app.module_select.as_ref().unwrap();
        let mut unique = module_select.options.clone();
        unique.sort_by_key(|module| *module as u8);
        unique.dedup();

        assert_eq!(module_select.options, boss_module_choices(2));
        assert_eq!(module_select.options.len(), 3);
        assert_eq!(unique.len(), 3);
        assert_eq!(
            module_select.context,
            ModuleSelectContext::BossReward { boss_level: 2 }
        );
    }

    #[test]
    fn boss_module_select_save_round_trip_preserves_context() {
        let mut app = active_boss_module_select_fixture(1);
        app.skip_reward();

        let restored = restore_app_from_snapshot(&app.serialize_current_run().unwrap());
        let module_select = restored.module_select.as_ref().unwrap();

        assert!(matches!(restored.screen, AppScreen::ModuleSelect));
        assert_eq!(module_select.options, boss_module_choices(1));
        assert_eq!(
            module_select.context,
            ModuleSelectContext::BossReward { boss_level: 1 }
        );
    }

    #[test]
    fn completed_run_reward_skip_opens_final_victory_without_adding_a_card() {
        let mut app = App::new();
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.current_level = dungeon.total_levels();
        let initial_deck_len = dungeon.deck.len();
        app.dungeon = Some(dungeon);
        app.screen = AppScreen::Reward;
        app.reward = Some(RewardState {
            tier: RewardTier::Boss,
            options: vec![CardId::OverwatchGrid],
            followup: RewardFollowup {
                completed_run: true,
            },
            seed: TEST_RUN_SEED,
        });

        app.skip_reward();

        assert!(matches!(
            app.screen,
            AppScreen::Result(CombatOutcome::Victory)
        ));
        assert!(app.reward.is_none());
        assert_eq!(app.dungeon.as_ref().unwrap().deck.len(), initial_deck_len);
        assert_eq!(
            app.log.back().map(String::as_str),
            Some("Skipped card reward.")
        );
    }

    #[test]
    fn final_victory_summary_is_available_after_completed_run() {
        let mut app = App::new();
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.current_level = dungeon.total_levels();
        dungeon.available_nodes.clear();
        dungeon.player_hp = 27;
        dungeon.deck.push(CardId::QuickStrike);
        app.dungeon = Some(dungeon);
        app.screen = AppScreen::Result(CombatOutcome::Victory);

        let summary = app.final_victory_summary().unwrap();

        assert_eq!(summary.total_levels, 3);
        assert_eq!(summary.player_hp, 27);
        assert_eq!(summary.player_max_hp, 32);
        assert_eq!(summary.deck_count, app.dungeon.as_ref().unwrap().deck.len());
        assert_eq!(
            summary.act_names,
            vec!["Relay Fringe", "Fracture Span", "Null Vault"]
        );
    }

    #[test]
    fn final_victory_screen_uses_updated_stat_copy() {
        let mut app = App::new();
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.current_level = dungeon.total_levels();
        dungeon.available_nodes.clear();
        dungeon.deck.push(CardId::QuickStrike);
        let expected_deck_count = dungeon.deck.len();
        app.dungeon = Some(dungeon);
        app.screen = AppScreen::Result(CombatOutcome::Victory);

        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        assert!(frame.contains(&format!(
            "|center|#c9ffd7|body|32 max HP    {expected_deck_count} card deck"
        )));
        assert!(!frame.contains("Max HP 40"));
        assert!(!frame.contains("Deck "));
    }

    #[test]
    fn final_victory_summary_is_absent_for_nonfinal_results() {
        let mut app = App::new();
        app.dungeon = Some(DungeonRun::new(TEST_RUN_SEED));
        app.screen = AppScreen::Result(CombatOutcome::Defeat);

        assert!(app.final_victory_summary().is_none());
    }

    #[test]
    fn defeat_summary_uses_run_progress_and_failure_context() {
        let mut app = App::new();
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.current_level = 2;
        dungeon.nodes = vec![
            crate::dungeon::DungeonNode {
                id: 0,
                depth: 0,
                lane: 3,
                kind: RoomKind::Start,
                next: vec![1],
            },
            crate::dungeon::DungeonNode {
                id: 1,
                depth: 1,
                lane: 3,
                kind: RoomKind::Elite,
                next: vec![],
            },
        ];
        dungeon.current_node = Some(1);
        dungeon.player_hp = 0;
        dungeon.combats_cleared = 4;
        dungeon.elites_cleared = 1;
        dungeon.rests_completed = 1;
        dungeon.bosses_cleared = 1;
        dungeon.deck.push(CardId::QuickStrike);
        app.dungeon = Some(dungeon);
        app.screen = AppScreen::Result(CombatOutcome::Defeat);

        let summary = app.defeat_summary().unwrap();
        let expected_enemy = app
            .dungeon
            .as_ref()
            .and_then(|dungeon| dungeon.current_encounter_setup())
            .and_then(|setup| {
                setup
                    .enemies
                    .first()
                    .map(|enemy| localized_enemy_name(enemy.profile, Language::English))
            });

        assert_eq!(summary.current_level, 2);
        assert_eq!(summary.total_levels, 3);
        assert_eq!(summary.sector_name, "Fracture Span");
        assert_eq!(summary.failure_room, Some(RoomKind::Elite));
        assert_eq!(summary.failure_enemy, expected_enemy);
        assert_eq!(summary.player_hp, 0);
        assert_eq!(summary.player_max_hp, 32);
        assert_eq!(
            defeat_by_text(&summary, Language::English),
            format!("by {}", expected_enemy.unwrap())
        );
        assert_eq!(summary.deck_count, app.dungeon.as_ref().unwrap().deck.len());
    }

    #[test]
    fn result_screen_escape_returns_to_main_menu() {
        let mut app = App::new();
        app.dungeon = Some(DungeonRun::new(TEST_RUN_SEED));
        app.screen = AppScreen::Result(CombatOutcome::Defeat);

        app.key_down(27);

        assert!(matches!(app.screen, AppScreen::Boot));
        assert!(app.dungeon.is_none());
    }

    #[test]
    fn defeat_screen_renders_requested_summary_lines() {
        let mut app = App::new();
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.current_level = 2;
        dungeon.nodes = vec![
            crate::dungeon::DungeonNode {
                id: 0,
                depth: 0,
                lane: 3,
                kind: RoomKind::Start,
                next: vec![1],
            },
            crate::dungeon::DungeonNode {
                id: 1,
                depth: 1,
                lane: 3,
                kind: RoomKind::Combat,
                next: vec![],
            },
        ];
        dungeon.current_node = Some(1);
        dungeon.combats_cleared = 4;
        dungeon.elites_cleared = 1;
        dungeon.bosses_cleared = 1;
        dungeon.deck.push(CardId::QuickStrike);
        let expected_deck_count = dungeon.deck.len();
        app.dungeon = Some(dungeon);
        app.screen = AppScreen::Result(CombatOutcome::Defeat);

        app.rebuild_frame();

        let frame = String::from_utf8(app.frame.clone()).unwrap();
        assert!(frame.contains("|center|#ff9cf0|body|Defeated by Volt Mantis"));
        assert!(frame.contains("|center|#c9ffd7|body|1 level cleared"));
        assert!(frame.contains("|center|#c9ffd7|body|4 fights cleared"));
        assert!(frame.contains("|center|#c9ffd7|body|1 elite cleared"));
        assert!(frame.contains("|center|#c9ffd7|body|1 boss cleared"));
        assert!(frame.contains("|center|#c9ffd7|body|32 max HP"));
        assert!(frame.contains(&format!(
            "|center|#c9ffd7|body|{expected_deck_count} Card Deck"
        )));
        assert!(frame.contains(&format!(
            "|center|#6f9f7b|body|Seed {}",
            display_seed(TEST_RUN_SEED)
        )));
        assert!(!frame.contains("Run Summary"));
        assert!(!frame.contains("Enter or Esc returns to menu"));
        assert!(!frame.contains("Levels cleared:"));
        assert!(!frame.contains("Defeat "));
        assert!(!frame.contains("Cleared"));
        assert!(!frame.contains("Levels cleared 1"));
        assert!(!frame.contains("Max HP "));
        assert!(!frame.contains("Seed:"));
    }

    #[test]
    fn result_buttons_anchor_main_menu_to_bottom_and_share_above() {
        let buttons = result_button_layout(LOGICAL_WIDTH, LOGICAL_HEIGHT, true, Language::English);
        let share = buttons.share_button.unwrap();
        let (_, pad_y) = boot_button_tile_padding();

        assert_eq!(
            buttons.menu_button.x + buttons.menu_button.w * 0.5,
            LOGICAL_WIDTH * 0.5
        );
        assert_eq!(share.x + share.w * 0.5, LOGICAL_WIDTH * 0.5);
        assert!(
            (LOGICAL_HEIGHT - (buttons.menu_button.y + buttons.menu_button.h) - pad_y).abs() < 0.1
        );
        assert!(share.y + share.h <= buttons.menu_button.y);
    }

    #[test]
    fn start_run_limits_visible_seed_to_32_bits() {
        let mut app = App::new();

        app.start_run();

        let seed = app.dungeon.as_ref().unwrap().seed;
        assert_eq!(seed, limit_run_seed(seed));
        assert_eq!(display_seed(seed).len(), 8);
    }

    #[test]
    fn queue_share_request_formats_final_victory_payload() {
        let mut app = App::new();
        let mut dungeon = DungeonRun::new(TEST_RUN_SEED);
        dungeon.current_level = dungeon.total_levels();
        dungeon.available_nodes.clear();
        dungeon.player_hp = 19;
        dungeon.deck.push(CardId::QuickStrike);
        let expected_deck_count = dungeon.deck.len();
        app.dungeon = Some(dungeon);
        app.screen = AppScreen::Result(CombatOutcome::Victory);

        app.queue_share_request();

        let share = app.share_request.as_ref().unwrap();
        assert!(share.contains(r#""kind":"final_victory_card""#));
        assert!(share.contains(r#""title":"Mazocarta""#));
        assert!(share.contains(r#""max_hp":32"#));
        assert!(share.contains(&format!(r#""deck_size":{expected_deck_count}"#)));
        assert!(share.contains(&format!(r#""seed":"{}""#, display_seed(TEST_RUN_SEED))));
        assert!(share.contains(&format!(r#""version":"{}""#, GAME_VERSION)));
        assert!(share.contains(r#""share_text":"I cleared all 3 sectors in Mazocarta."#));
    }
}
