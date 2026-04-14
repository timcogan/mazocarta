#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CardId {
    FlareSlash,
    FlareSlashPlus,
    GuardStep,
    GuardStepPlus,
    Slipstream,
    SlipstreamPlus,
    QuickStrike,
    QuickStrikePlus,
    Reinforce,
    ReinforcePlus,
    PressurePoint,
    PressurePointPlus,
    SunderingArc,
    SunderingArcPlus,
    TwinStrike,
    TwinStrikePlus,
    BarrierField,
    BarrierFieldPlus,
    TacticalBurst,
    TacticalBurstPlus,
    RazorNet,
    RazorNetPlus,
    BreachSignal,
    BreachSignalPlus,
    AnchorLoop,
    AnchorLoopPlus,
    ExecutionBeam,
    ExecutionBeamPlus,
    FortressMatrix,
    FortressMatrixPlus,
    ZeroPoint,
    ZeroPointPlus,
    PinpointJab,
    PinpointJabPlus,
    SignalTap,
    SignalTapPlus,
    BurstArray,
    BurstArrayPlus,
    CoverPulse,
    CoverPulsePlus,
    FracturePulse,
    FracturePulsePlus,
    VectorLock,
    VectorLockPlus,
    ChainBarrage,
    ChainBarragePlus,
    OverwatchGrid,
    OverwatchGridPlus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CardTarget {
    Enemy,
    SelfOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CardDef {
    pub(crate) id: CardId,
    pub(crate) name: &'static str,
    pub(crate) cost: u8,
    pub(crate) target: CardTarget,
    pub(crate) description: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RewardTier {
    Combat,
    Elite,
    Boss,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ShopOffer {
    pub(crate) card: CardId,
    pub(crate) price: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EventId {
    SalvageCache,
    ClinicPod,
    PrototypeRack,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct EventDef {
    pub(crate) id: EventId,
    pub(crate) title: &'static str,
    pub(crate) flavor: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EventChoiceEffect {
    GainCredits(u32),
    LoseHpGainCredits { lose_hp: i32, gain_credits: u32 },
    Heal(i32),
    LoseHpGainMaxHp { lose_hp: i32, gain_max_hp: i32 },
    AddCard(CardId),
    LoseHpAddCard { lose_hp: i32, card: CardId },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ModuleId {
    AegisDrive,
    TargetingRelay,
    Nanoforge,
    CapacitorBank,
    PrismScope,
    SalvageLedger,
    OverclockCore,
    SuppressionField,
    RecoveryMatrix,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ModuleDef {
    pub(crate) id: ModuleId,
    pub(crate) name: &'static str,
    pub(crate) description: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EnemyProfileId {
    ScoutDrone,
    NeedlerDrone,
    RampartDrone,
    SpineSentry,
    PentaCore,
    VoltMantis,
    ShardWeaver,
    PrismArray,
    GlassBishop,
    HexarchCore,
    NullRaider,
    RiftStalker,
    BulwarkDrone,
    RiftBastion,
    HeptarchCore,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct EnemyIntent {
    pub(crate) name: &'static str,
    pub(crate) summary: &'static str,
    pub(crate) damage: i32,
    pub(crate) hits: u8,
    pub(crate) gain_block: i32,
    pub(crate) gain_strength: u8,
    pub(crate) prime_bleed: u8,
    pub(crate) apply_expose: u8,
    pub(crate) apply_weak: u8,
    pub(crate) apply_frail: u8,
    pub(crate) apply_bleed: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Language {
    #[default]
    English,
    Spanish,
}

impl Language {
    pub(crate) fn from_code(code: u32) -> Self {
        match code {
            1 => Self::Spanish,
            _ => Self::English,
        }
    }

    pub(crate) fn code(self) -> u32 {
        match self {
            Self::English => 0,
            Self::Spanish => 1,
        }
    }
}

pub(crate) fn localized_text<'a>(
    language: Language,
    english: &'a str,
    spanish: &'a str,
) -> &'a str {
    match language {
        Language::English => english,
        Language::Spanish => spanish,
    }
}

pub(crate) fn card_def(id: CardId) -> CardDef {
    match id {
        CardId::FlareSlash => CardDef {
            id,
            name: "Strike",
            cost: 1,
            target: CardTarget::Enemy,
            description: "Deal 6 damage.",
        },
        CardId::FlareSlashPlus => CardDef {
            id,
            name: "Strike+",
            cost: 1,
            target: CardTarget::Enemy,
            description: "Deal 9 damage.",
        },
        CardId::GuardStep => CardDef {
            id,
            name: "Defend",
            cost: 1,
            target: CardTarget::SelfOnly,
            description: "Gain 5 Shield.",
        },
        CardId::GuardStepPlus => CardDef {
            id,
            name: "Defend+",
            cost: 1,
            target: CardTarget::SelfOnly,
            description: "Gain 8 Shield.",
        },
        CardId::Slipstream => CardDef {
            id,
            name: "Reposition",
            cost: 0,
            target: CardTarget::SelfOnly,
            description: "Draw 1. Gain 2 Shield.",
        },
        CardId::SlipstreamPlus => CardDef {
            id,
            name: "Reposition+",
            cost: 0,
            target: CardTarget::SelfOnly,
            description: "Draw 1. Gain 4 Shield.",
        },
        CardId::QuickStrike => CardDef {
            id,
            name: "Quick Strike",
            cost: 1,
            target: CardTarget::Enemy,
            description: "Deal 5 damage. Draw 1.",
        },
        CardId::QuickStrikePlus => CardDef {
            id,
            name: "Quick Strike+",
            cost: 1,
            target: CardTarget::Enemy,
            description: "Deal 7 damage. Draw 1.",
        },
        CardId::PinpointJab => CardDef {
            id,
            name: "Pinpoint Jab",
            cost: 1,
            target: CardTarget::Enemy,
            description: "Deal 5 damage. Apply Bleed 1.",
        },
        CardId::PinpointJabPlus => CardDef {
            id,
            name: "Pinpoint Jab+",
            cost: 1,
            target: CardTarget::Enemy,
            description: "Deal 7 damage. Apply Bleed 1.",
        },
        CardId::SignalTap => CardDef {
            id,
            name: "Signal Tap",
            cost: 0,
            target: CardTarget::Enemy,
            description: "Apply 1 Vulnerable. Draw 1.",
        },
        CardId::SignalTapPlus => CardDef {
            id,
            name: "Signal Tap+",
            cost: 0,
            target: CardTarget::Enemy,
            description: "Apply 1 Vulnerable. Draw 1. Gain 3 Shield.",
        },
        CardId::Reinforce => CardDef {
            id,
            name: "Reinforce",
            cost: 1,
            target: CardTarget::SelfOnly,
            description: "Gain 8 Shield.",
        },
        CardId::ReinforcePlus => CardDef {
            id,
            name: "Reinforce+",
            cost: 1,
            target: CardTarget::SelfOnly,
            description: "Gain 11 Shield.",
        },
        CardId::PressurePoint => CardDef {
            id,
            name: "Pressure Point",
            cost: 1,
            target: CardTarget::Enemy,
            description: "Deal 4 damage. Apply Weak 1.",
        },
        CardId::PressurePointPlus => CardDef {
            id,
            name: "Pressure Point+",
            cost: 1,
            target: CardTarget::Enemy,
            description: "Deal 6 damage. Apply Weak 2.",
        },
        CardId::BurstArray => CardDef {
            id,
            name: "Burst Array",
            cost: 1,
            target: CardTarget::Enemy,
            description: "Deal 3 damage three times.",
        },
        CardId::BurstArrayPlus => CardDef {
            id,
            name: "Burst Array+",
            cost: 1,
            target: CardTarget::Enemy,
            description: "Deal 4 damage three times.",
        },
        CardId::CoverPulse => CardDef {
            id,
            name: "Cover Pulse",
            cost: 1,
            target: CardTarget::SelfOnly,
            description: "Gain 6 Shield. Draw 1.",
        },
        CardId::CoverPulsePlus => CardDef {
            id,
            name: "Cover Pulse+",
            cost: 1,
            target: CardTarget::SelfOnly,
            description: "Gain 8 Shield. Draw 1.",
        },
        CardId::SunderingArc => CardDef {
            id,
            name: "Precise Strike",
            cost: 2,
            target: CardTarget::Enemy,
            description: "Deal 12 damage. Apply 1 Vulnerable.",
        },
        CardId::SunderingArcPlus => CardDef {
            id,
            name: "Precise Strike+",
            cost: 2,
            target: CardTarget::Enemy,
            description: "Deal 16 damage. Apply 1 Vulnerable.",
        },
        CardId::TwinStrike => CardDef {
            id,
            name: "Twin Strike",
            cost: 1,
            target: CardTarget::Enemy,
            description: "Deal 4 damage twice.",
        },
        CardId::TwinStrikePlus => CardDef {
            id,
            name: "Twin Strike+",
            cost: 1,
            target: CardTarget::Enemy,
            description: "Deal 5 damage twice.",
        },
        CardId::BarrierField => CardDef {
            id,
            name: "Barrier Field",
            cost: 1,
            target: CardTarget::Enemy,
            description: "Gain 10 Shield. Apply Frail 1.",
        },
        CardId::BarrierFieldPlus => CardDef {
            id,
            name: "Barrier Field+",
            cost: 1,
            target: CardTarget::Enemy,
            description: "Gain 13 Shield. Apply Frail 2.",
        },
        CardId::TacticalBurst => CardDef {
            id,
            name: "Tactical Burst",
            cost: 1,
            target: CardTarget::SelfOnly,
            description: "Draw 2. Gain Strength 1.",
        },
        CardId::TacticalBurstPlus => CardDef {
            id,
            name: "Tactical Burst+",
            cost: 1,
            target: CardTarget::SelfOnly,
            description: "Draw 2. Gain Strength 2.",
        },
        CardId::RazorNet => CardDef {
            id,
            name: "Razor Net",
            cost: 1,
            target: CardTarget::Enemy,
            description: "Deal 4 damage twice. Apply Bleed 2.",
        },
        CardId::RazorNetPlus => CardDef {
            id,
            name: "Razor Net+",
            cost: 1,
            target: CardTarget::Enemy,
            description: "Deal 5 damage twice. Apply Bleed 2.",
        },
        CardId::FracturePulse => CardDef {
            id,
            name: "Fracture Pulse",
            cost: 2,
            target: CardTarget::Enemy,
            description: "Deal 9 damage. Apply Bleed 3.",
        },
        CardId::FracturePulsePlus => CardDef {
            id,
            name: "Fracture Pulse+",
            cost: 2,
            target: CardTarget::Enemy,
            description: "Deal 12 damage. Apply Bleed 3.",
        },
        CardId::VectorLock => CardDef {
            id,
            name: "Vector Lock",
            cost: 1,
            target: CardTarget::Enemy,
            description: "Deal 6 damage. Apply 2 Vulnerable. Gain 5 Shield.",
        },
        CardId::VectorLockPlus => CardDef {
            id,
            name: "Vector Lock+",
            cost: 1,
            target: CardTarget::Enemy,
            description: "Deal 8 damage. Apply 2 Vulnerable. Gain 6 Shield.",
        },
        CardId::BreachSignal => CardDef {
            id,
            name: "Breach Signal",
            cost: 1,
            target: CardTarget::Enemy,
            description: "Deal 7 damage. Draw 1. Apply 2 Vulnerable.",
        },
        CardId::BreachSignalPlus => CardDef {
            id,
            name: "Breach Signal+",
            cost: 1,
            target: CardTarget::Enemy,
            description: "Deal 9 damage. Draw 1. Apply 2 Vulnerable.",
        },
        CardId::AnchorLoop => CardDef {
            id,
            name: "Anchor Loop",
            cost: 2,
            target: CardTarget::SelfOnly,
            description: "Gain 14 Shield. Draw 2.",
        },
        CardId::AnchorLoopPlus => CardDef {
            id,
            name: "Anchor Loop+",
            cost: 2,
            target: CardTarget::SelfOnly,
            description: "Gain 17 Shield. Draw 2.",
        },
        CardId::ExecutionBeam => CardDef {
            id,
            name: "Execution Beam",
            cost: 3,
            target: CardTarget::Enemy,
            description: "Deal 20 damage.",
        },
        CardId::ExecutionBeamPlus => CardDef {
            id,
            name: "Execution Beam+",
            cost: 3,
            target: CardTarget::Enemy,
            description: "Deal 26 damage.",
        },
        CardId::ChainBarrage => CardDef {
            id,
            name: "Chain Barrage",
            cost: 2,
            target: CardTarget::Enemy,
            description: "Deal 8 damage twice. Apply Bleed 2.",
        },
        CardId::ChainBarragePlus => CardDef {
            id,
            name: "Chain Barrage+",
            cost: 2,
            target: CardTarget::Enemy,
            description: "Deal 10 damage twice. Apply Bleed 2.",
        },
        CardId::FortressMatrix => CardDef {
            id,
            name: "Fortress Matrix",
            cost: 2,
            target: CardTarget::SelfOnly,
            description: "Gain 16 Shield. Draw 1.",
        },
        CardId::FortressMatrixPlus => CardDef {
            id,
            name: "Fortress Matrix+",
            cost: 2,
            target: CardTarget::SelfOnly,
            description: "Gain 20 Shield. Draw 1.",
        },
        CardId::OverwatchGrid => CardDef {
            id,
            name: "Overwatch Grid",
            cost: 2,
            target: CardTarget::SelfOnly,
            description: "Gain 18 Shield. Draw 2.",
        },
        CardId::OverwatchGridPlus => CardDef {
            id,
            name: "Overwatch Grid+",
            cost: 2,
            target: CardTarget::SelfOnly,
            description: "Gain 22 Shield. Draw 2.",
        },
        CardId::ZeroPoint => CardDef {
            id,
            name: "Zero Point",
            cost: 2,
            target: CardTarget::Enemy,
            description: "Deal 10 damage. Draw 1. Apply 2 Vulnerable.",
        },
        CardId::ZeroPointPlus => CardDef {
            id,
            name: "Zero Point+",
            cost: 2,
            target: CardTarget::Enemy,
            description: "Deal 14 damage. Draw 1. Apply 2 Vulnerable.",
        },
    }
}

pub(crate) fn upgraded_card(id: CardId) -> Option<CardId> {
    match id {
        CardId::FlareSlash => Some(CardId::FlareSlashPlus),
        CardId::GuardStep => Some(CardId::GuardStepPlus),
        CardId::Slipstream => Some(CardId::SlipstreamPlus),
        CardId::QuickStrike => Some(CardId::QuickStrikePlus),
        CardId::PinpointJab => Some(CardId::PinpointJabPlus),
        CardId::SignalTap => Some(CardId::SignalTapPlus),
        CardId::Reinforce => Some(CardId::ReinforcePlus),
        CardId::PressurePoint => Some(CardId::PressurePointPlus),
        CardId::BurstArray => Some(CardId::BurstArrayPlus),
        CardId::CoverPulse => Some(CardId::CoverPulsePlus),
        CardId::SunderingArc => Some(CardId::SunderingArcPlus),
        CardId::TwinStrike => Some(CardId::TwinStrikePlus),
        CardId::BarrierField => Some(CardId::BarrierFieldPlus),
        CardId::TacticalBurst => Some(CardId::TacticalBurstPlus),
        CardId::RazorNet => Some(CardId::RazorNetPlus),
        CardId::FracturePulse => Some(CardId::FracturePulsePlus),
        CardId::VectorLock => Some(CardId::VectorLockPlus),
        CardId::BreachSignal => Some(CardId::BreachSignalPlus),
        CardId::AnchorLoop => Some(CardId::AnchorLoopPlus),
        CardId::ExecutionBeam => Some(CardId::ExecutionBeamPlus),
        CardId::ChainBarrage => Some(CardId::ChainBarragePlus),
        CardId::FortressMatrix => Some(CardId::FortressMatrixPlus),
        CardId::OverwatchGrid => Some(CardId::OverwatchGridPlus),
        CardId::ZeroPoint => Some(CardId::ZeroPointPlus),
        CardId::FlareSlashPlus
        | CardId::GuardStepPlus
        | CardId::SlipstreamPlus
        | CardId::QuickStrikePlus
        | CardId::PinpointJabPlus
        | CardId::SignalTapPlus
        | CardId::ReinforcePlus
        | CardId::PressurePointPlus
        | CardId::BurstArrayPlus
        | CardId::CoverPulsePlus
        | CardId::SunderingArcPlus
        | CardId::TwinStrikePlus
        | CardId::BarrierFieldPlus
        | CardId::TacticalBurstPlus
        | CardId::RazorNetPlus
        | CardId::FracturePulsePlus
        | CardId::VectorLockPlus
        | CardId::BreachSignalPlus
        | CardId::AnchorLoopPlus
        | CardId::ExecutionBeamPlus
        | CardId::ChainBarragePlus
        | CardId::FortressMatrixPlus
        | CardId::OverwatchGridPlus
        | CardId::ZeroPointPlus => None,
    }
}

pub(crate) fn starter_deck() -> Vec<CardId> {
    let mut cards = Vec::with_capacity(12);
    cards.extend(std::iter::repeat_n(CardId::FlareSlash, 5));
    cards.extend(std::iter::repeat_n(CardId::GuardStep, 4));
    cards.extend(std::iter::repeat_n(CardId::Slipstream, 2));
    cards.push(CardId::SunderingArc);
    cards
}

pub(crate) fn module_def(id: ModuleId) -> ModuleDef {
    match id {
        ModuleId::AegisDrive => ModuleDef {
            id,
            name: "Aegis Drive",
            description: "Start each combat with 5 Shield.",
        },
        ModuleId::TargetingRelay => ModuleDef {
            id,
            name: "Targeting Relay",
            description: "Start each combat by applying Vulnerable 1 to the enemy.",
        },
        ModuleId::Nanoforge => ModuleDef {
            id,
            name: "Nanoforge",
            description: "After each victory, recover 3 HP.",
        },
        ModuleId::CapacitorBank => ModuleDef {
            id,
            name: "Capacitor Bank",
            description: "Start each combat with Strength 1.",
        },
        ModuleId::PrismScope => ModuleDef {
            id,
            name: "Prism Scope",
            description: "Start each combat by applying Vulnerable 1 to all enemies.",
        },
        ModuleId::SalvageLedger => ModuleDef {
            id,
            name: "Salvage Ledger",
            description: "After each victory, gain 4 additional Credits.",
        },
        ModuleId::OverclockCore => ModuleDef {
            id,
            name: "Overclock Core",
            description: "Start each combat with 1 extra Energy.",
        },
        ModuleId::SuppressionField => ModuleDef {
            id,
            name: "Suppression Field",
            description: "Start each combat by applying Weak 1 to all enemies.",
        },
        ModuleId::RecoveryMatrix => ModuleDef {
            id,
            name: "Recovery Matrix",
            description: "After each victory, recover 5 HP.",
        },
    }
}

pub(crate) fn default_starter_module() -> ModuleId {
    ModuleId::AegisDrive
}

pub(crate) fn starter_module_choices(_seed: u64) -> Vec<ModuleId> {
    vec![
        ModuleId::Nanoforge,
        ModuleId::AegisDrive,
        ModuleId::TargetingRelay,
    ]
}

pub(crate) fn boss_module_choices(boss_level: usize) -> Vec<ModuleId> {
    match boss_level.clamp(1, 2) {
        1 => vec![
            ModuleId::CapacitorBank,
            ModuleId::PrismScope,
            ModuleId::SalvageLedger,
        ],
        _ => vec![
            ModuleId::OverclockCore,
            ModuleId::SuppressionField,
            ModuleId::RecoveryMatrix,
        ],
    }
}

pub(crate) fn reward_choices(seed: u64, tier: RewardTier, level: usize) -> Vec<CardId> {
    let mut cards = reward_pool(tier, level).to_vec();
    cards.sort_by_key(|card| reward_roll_key(seed, *card));
    cards.truncate(cards.len().min(3));
    cards
}

pub(crate) fn shop_offers(seed: u64, level: usize) -> Vec<ShopOffer> {
    let mut chosen = Vec::new();
    let mut offers = Vec::with_capacity(3);

    for (index, tier) in shop_offer_tiers(level).into_iter().enumerate() {
        let offer_seed = seed ^ (index as u64 + 1).wrapping_mul(0xD6E8_FD50_19E3_7C4B);
        let mut cards = reward_pool(tier, level).to_vec();
        cards.sort_by_key(|card| reward_roll_key(offer_seed, *card));
        let card = cards
            .iter()
            .copied()
            .find(|card| !chosen.contains(card))
            .or_else(|| cards.first().copied())
            .expect("shop tier should always have at least one card");
        chosen.push(card);
        offers.push(ShopOffer {
            card,
            price: shop_price_for_tier(tier),
        });
    }

    offers
}

fn reward_pool(tier: RewardTier, level: usize) -> &'static [CardId] {
    match tier {
        RewardTier::Combat => &[
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
        ],
        RewardTier::Elite => match level.clamp(1, 3) {
            1 => &[
                CardId::SunderingArc,
                CardId::TwinStrike,
                CardId::BarrierField,
                CardId::TacticalBurst,
                CardId::RazorNet,
                CardId::FracturePulse,
                CardId::VectorLock,
            ],
            2 => &[
                CardId::SunderingArc,
                CardId::TwinStrike,
                CardId::BarrierField,
                CardId::TacticalBurst,
                CardId::RazorNet,
                CardId::FracturePulse,
                CardId::VectorLock,
                CardId::BreachSignal,
            ],
            _ => &[
                CardId::SunderingArc,
                CardId::TwinStrike,
                CardId::BarrierField,
                CardId::TacticalBurst,
                CardId::RazorNet,
                CardId::FracturePulse,
                CardId::VectorLock,
                CardId::BreachSignal,
                CardId::AnchorLoop,
            ],
        },
        RewardTier::Boss => &[
            CardId::ExecutionBeam,
            CardId::ChainBarrage,
            CardId::FortressMatrix,
            CardId::OverwatchGrid,
            CardId::ZeroPoint,
        ],
    }
}

fn shop_offer_tiers(level: usize) -> [RewardTier; 3] {
    match level.clamp(1, 3) {
        1 => [RewardTier::Combat, RewardTier::Elite, RewardTier::Elite],
        2 => [RewardTier::Combat, RewardTier::Elite, RewardTier::Boss],
        _ => [RewardTier::Elite, RewardTier::Elite, RewardTier::Boss],
    }
}

pub(crate) fn shop_price_for_tier(tier: RewardTier) -> u32 {
    match tier {
        RewardTier::Combat => 16,
        RewardTier::Elite => 24,
        RewardTier::Boss => 40,
    }
}

pub(crate) fn event_def(id: EventId) -> EventDef {
    match id {
        EventId::SalvageCache => EventDef {
            id,
            title: "Salvage Cache",
            flavor: "A drift crate hums beneath a half-collapsed service gantry.",
        },
        EventId::ClinicPod => EventDef {
            id,
            title: "Clinic Pod",
            flavor: "An intact med pod still cycles a pale diagnostic glow.",
        },
        EventId::PrototypeRack => EventDef {
            id,
            title: "Prototype Rack",
            flavor: "A sealed rack flickers between proven gear and live-fire prototypes.",
        },
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn event_choice_title(event: EventId, choice_index: usize) -> Option<&'static str> {
    match (event, choice_index) {
        (EventId::SalvageCache, 0) => Some("Take the clean parts"),
        (EventId::SalvageCache, 1) => Some("Cut the safety seals"),
        (EventId::ClinicPod, 0) => Some("Run the recovery cycle"),
        (EventId::ClinicPod, 1) => Some("Overclock the chassis"),
        (EventId::PrototypeRack, 0) => Some("Take the stable shell"),
        (EventId::PrototypeRack, 1) => Some("Take the live prototype"),
        _ => None,
    }
}

pub(crate) fn event_choice_effect(
    event: EventId,
    choice_index: usize,
    level: usize,
) -> Option<EventChoiceEffect> {
    let level = level.clamp(1, 3);
    match (event, choice_index) {
        (EventId::SalvageCache, 0) => Some(EventChoiceEffect::GainCredits(match level {
            1 => 16,
            2 => 20,
            _ => 24,
        })),
        (EventId::SalvageCache, 1) => Some(EventChoiceEffect::LoseHpGainCredits {
            lose_hp: 6,
            gain_credits: match level {
                1 => 28,
                2 => 34,
                _ => 40,
            },
        }),
        (EventId::ClinicPod, 0) => Some(EventChoiceEffect::Heal(match level {
            1 => 8,
            2 => 10,
            _ => 12,
        })),
        (EventId::ClinicPod, 1) => Some(EventChoiceEffect::LoseHpGainMaxHp {
            lose_hp: 5,
            gain_max_hp: 4,
        }),
        (EventId::PrototypeRack, 0) => Some(EventChoiceEffect::AddCard(match level {
            1 => CardId::CoverPulse,
            2 => CardId::BarrierField,
            _ => CardId::FortressMatrix,
        })),
        (EventId::PrototypeRack, 1) => Some(EventChoiceEffect::LoseHpAddCard {
            lose_hp: 5,
            card: match level {
                1 => CardId::TacticalBurst,
                2 => CardId::VectorLock,
                _ => CardId::ZeroPoint,
            },
        }),
        _ => None,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn event_choice_body(
    event: EventId,
    choice_index: usize,
    level: usize,
) -> Option<String> {
    match event_choice_effect(event, choice_index, level)? {
        EventChoiceEffect::GainCredits(credits) => Some(format!("Gain {credits} Credits.")),
        EventChoiceEffect::LoseHpGainCredits {
            lose_hp,
            gain_credits,
        } => Some(format!("Lose {lose_hp} HP. Gain {gain_credits} Credits.")),
        EventChoiceEffect::Heal(amount) => Some(format!("Recover {amount} HP.")),
        EventChoiceEffect::LoseHpGainMaxHp {
            lose_hp,
            gain_max_hp,
        } => Some(format!("Lose {lose_hp} HP. Gain {gain_max_hp} max HP.")),
        EventChoiceEffect::AddCard(card) => {
            Some(format!("Add {} to your deck.", card_def(card).name))
        }
        EventChoiceEffect::LoseHpAddCard { lose_hp, card } => Some(format!(
            "Lose {lose_hp} HP. Add {} to your deck.",
            card_def(card).name
        )),
    }
}

pub(crate) fn localized_card_def(id: CardId, language: Language) -> CardDef {
    let mut def = card_def(id);
    def.name = localized_card_name(id, language);
    def.description = localized_card_description(id, language);
    def
}

pub(crate) fn localized_card_name(id: CardId, language: Language) -> &'static str {
    match id {
        CardId::FlareSlash => localized_text(language, "Strike", "Golpe"),
        CardId::FlareSlashPlus => localized_text(language, "Strike+", "Golpe+"),
        CardId::GuardStep => localized_text(language, "Defend", "Defensa"),
        CardId::GuardStepPlus => localized_text(language, "Defend+", "Defensa+"),
        CardId::Slipstream => localized_text(language, "Reposition", "Reposición"),
        CardId::SlipstreamPlus => localized_text(language, "Reposition+", "Reposición+"),
        CardId::QuickStrike => localized_text(language, "Quick Strike", "Golpe Veloz"),
        CardId::QuickStrikePlus => localized_text(language, "Quick Strike+", "Golpe Veloz+"),
        CardId::PinpointJab => localized_text(language, "Pinpoint Jab", "Golpe Certero"),
        CardId::PinpointJabPlus => localized_text(language, "Pinpoint Jab+", "Golpe Certero+"),
        CardId::SignalTap => localized_text(language, "Signal Tap", "Pulso de Señal"),
        CardId::SignalTapPlus => localized_text(language, "Signal Tap+", "Pulso de Señal+"),
        CardId::Reinforce => localized_text(language, "Reinforce", "Refuerzo"),
        CardId::ReinforcePlus => localized_text(language, "Reinforce+", "Refuerzo+"),
        CardId::PressurePoint => localized_text(language, "Pressure Point", "Punto de Presión"),
        CardId::PressurePointPlus => {
            localized_text(language, "Pressure Point+", "Punto de Presión+")
        }
        CardId::BurstArray => localized_text(language, "Burst Array", "Ráfaga en Serie"),
        CardId::BurstArrayPlus => localized_text(language, "Burst Array+", "Ráfaga en Serie+"),
        CardId::CoverPulse => localized_text(language, "Cover Pulse", "Pulso de Cobertura"),
        CardId::CoverPulsePlus => localized_text(language, "Cover Pulse+", "Pulso de Cobertura+"),
        CardId::SunderingArc => localized_text(language, "Precise Strike", "Golpe Preciso"),
        CardId::SunderingArcPlus => localized_text(language, "Precise Strike+", "Golpe Preciso+"),
        CardId::TwinStrike => localized_text(language, "Twin Strike", "Golpe Doble"),
        CardId::TwinStrikePlus => localized_text(language, "Twin Strike+", "Golpe Doble+"),
        CardId::BarrierField => localized_text(language, "Barrier Field", "Campo de Barrera"),
        CardId::BarrierFieldPlus => localized_text(language, "Barrier Field+", "Campo de Barrera+"),
        CardId::TacticalBurst => localized_text(language, "Tactical Burst", "Impulso Táctico"),
        CardId::TacticalBurstPlus => {
            localized_text(language, "Tactical Burst+", "Impulso Táctico+")
        }
        CardId::RazorNet => localized_text(language, "Razor Net", "Red Cortante"),
        CardId::RazorNetPlus => localized_text(language, "Razor Net+", "Red Cortante+"),
        CardId::FracturePulse => localized_text(language, "Fracture Pulse", "Pulso de Fractura"),
        CardId::FracturePulsePlus => {
            localized_text(language, "Fracture Pulse+", "Pulso de Fractura+")
        }
        CardId::VectorLock => localized_text(language, "Vector Lock", "Bloqueo Vectorial"),
        CardId::VectorLockPlus => localized_text(language, "Vector Lock+", "Bloqueo Vectorial+"),
        CardId::BreachSignal => localized_text(language, "Breach Signal", "Señal de Brecha"),
        CardId::BreachSignalPlus => localized_text(language, "Breach Signal+", "Señal de Brecha+"),
        CardId::AnchorLoop => localized_text(language, "Anchor Loop", "Bucle de Anclaje"),
        CardId::AnchorLoopPlus => localized_text(language, "Anchor Loop+", "Bucle de Anclaje+"),
        CardId::ExecutionBeam => localized_text(language, "Execution Beam", "Rayo de Ejecución"),
        CardId::ExecutionBeamPlus => {
            localized_text(language, "Execution Beam+", "Rayo de Ejecución+")
        }
        CardId::ChainBarrage => localized_text(language, "Chain Barrage", "Andanada en Cadena"),
        CardId::ChainBarragePlus => {
            localized_text(language, "Chain Barrage+", "Andanada en Cadena+")
        }
        CardId::FortressMatrix => {
            localized_text(language, "Fortress Matrix", "Matriz de Fortaleza")
        }
        CardId::FortressMatrixPlus => {
            localized_text(language, "Fortress Matrix+", "Matriz de Fortaleza+")
        }
        CardId::OverwatchGrid => localized_text(language, "Overwatch Grid", "Red de Vigilancia"),
        CardId::OverwatchGridPlus => {
            localized_text(language, "Overwatch Grid+", "Red de Vigilancia+")
        }
        CardId::ZeroPoint => localized_text(language, "Zero Point", "Punto Cero"),
        CardId::ZeroPointPlus => localized_text(language, "Zero Point+", "Punto Cero+"),
    }
}

pub(crate) fn localized_card_description(id: CardId, language: Language) -> &'static str {
    match id {
        CardId::FlareSlash => localized_text(language, "Deal 6 damage.", "Inflige 6 de daño."),
        CardId::FlareSlashPlus => localized_text(language, "Deal 9 damage.", "Inflige 9 de daño."),
        CardId::GuardStep => localized_text(language, "Gain 5 Shield.", "Gana 5 de Escudo."),
        CardId::GuardStepPlus => localized_text(language, "Gain 8 Shield.", "Gana 8 de Escudo."),
        CardId::Slipstream => localized_text(
            language,
            "Draw 1. Gain 2 Shield.",
            "Roba 1. Gana 2 de Escudo.",
        ),
        CardId::SlipstreamPlus => localized_text(
            language,
            "Draw 1. Gain 4 Shield.",
            "Roba 1. Gana 4 de Escudo.",
        ),
        CardId::QuickStrike => localized_text(
            language,
            "Deal 5 damage. Draw 1.",
            "Inflige 5 de daño. Roba 1.",
        ),
        CardId::QuickStrikePlus => localized_text(
            language,
            "Deal 7 damage. Draw 1.",
            "Inflige 7 de daño. Roba 1.",
        ),
        CardId::PinpointJab => localized_text(
            language,
            "Deal 5 damage. Apply Bleed 1.",
            "Inflige 5 de daño. Aplica Sangrado 1.",
        ),
        CardId::PinpointJabPlus => localized_text(
            language,
            "Deal 7 damage. Apply Bleed 1.",
            "Inflige 7 de daño. Aplica Sangrado 1.",
        ),
        CardId::SignalTap => localized_text(
            language,
            "Apply 1 Vulnerable. Draw 1.",
            "Aplica Vulnerable 1. Roba 1.",
        ),
        CardId::SignalTapPlus => localized_text(
            language,
            "Apply 1 Vulnerable. Draw 1. Gain 3 Shield.",
            "Aplica Vulnerable 1. Roba 1. Gana 3 de Escudo.",
        ),
        CardId::Reinforce => localized_text(language, "Gain 8 Shield.", "Gana 8 de Escudo."),
        CardId::ReinforcePlus => localized_text(language, "Gain 11 Shield.", "Gana 11 de Escudo."),
        CardId::PressurePoint => localized_text(
            language,
            "Deal 4 damage. Apply Weak 1.",
            "Inflige 4 de daño. Aplica Débil 1.",
        ),
        CardId::PressurePointPlus => localized_text(
            language,
            "Deal 6 damage. Apply Weak 2.",
            "Inflige 6 de daño. Aplica Débil 2.",
        ),
        CardId::BurstArray => localized_text(
            language,
            "Deal 3 damage three times.",
            "Inflige 3 de daño tres veces.",
        ),
        CardId::BurstArrayPlus => localized_text(
            language,
            "Deal 4 damage three times.",
            "Inflige 4 de daño tres veces.",
        ),
        CardId::CoverPulse => localized_text(
            language,
            "Gain 6 Shield. Draw 1.",
            "Gana 6 de Escudo. Roba 1.",
        ),
        CardId::CoverPulsePlus => localized_text(
            language,
            "Gain 8 Shield. Draw 1.",
            "Gana 8 de Escudo. Roba 1.",
        ),
        CardId::SunderingArc => localized_text(
            language,
            "Deal 12 damage. Apply 1 Vulnerable.",
            "Inflige 12 de daño. Aplica Vulnerable 1.",
        ),
        CardId::SunderingArcPlus => localized_text(
            language,
            "Deal 16 damage. Apply 1 Vulnerable.",
            "Inflige 16 de daño. Aplica Vulnerable 1.",
        ),
        CardId::TwinStrike => localized_text(
            language,
            "Deal 4 damage twice.",
            "Inflige 4 de daño dos veces.",
        ),
        CardId::TwinStrikePlus => localized_text(
            language,
            "Deal 5 damage twice.",
            "Inflige 5 de daño dos veces.",
        ),
        CardId::BarrierField => localized_text(
            language,
            "Gain 10 Shield. Apply Frail 1.",
            "Gana 10 de Escudo. Aplica Frágil 1.",
        ),
        CardId::BarrierFieldPlus => localized_text(
            language,
            "Gain 13 Shield. Apply Frail 2.",
            "Gana 13 de Escudo. Aplica Frágil 2.",
        ),
        CardId::TacticalBurst => localized_text(
            language,
            "Draw 2. Gain Strength 1.",
            "Roba 2. Gana Fuerza 1.",
        ),
        CardId::TacticalBurstPlus => localized_text(
            language,
            "Draw 2. Gain Strength 2.",
            "Roba 2. Gana Fuerza 2.",
        ),
        CardId::RazorNet => localized_text(
            language,
            "Deal 4 damage twice. Apply Bleed 2.",
            "Inflige 4 de daño dos veces. Aplica Sangrado 2.",
        ),
        CardId::RazorNetPlus => localized_text(
            language,
            "Deal 5 damage twice. Apply Bleed 2.",
            "Inflige 5 de daño dos veces. Aplica Sangrado 2.",
        ),
        CardId::FracturePulse => localized_text(
            language,
            "Deal 9 damage. Apply Bleed 3.",
            "Inflige 9 de daño. Aplica Sangrado 3.",
        ),
        CardId::FracturePulsePlus => localized_text(
            language,
            "Deal 12 damage. Apply Bleed 3.",
            "Inflige 12 de daño. Aplica Sangrado 3.",
        ),
        CardId::VectorLock => localized_text(
            language,
            "Deal 6 damage. Apply 2 Vulnerable. Gain 5 Shield.",
            "Inflige 6 de daño. Aplica Vulnerable 2. Gana 5 de Escudo.",
        ),
        CardId::VectorLockPlus => localized_text(
            language,
            "Deal 8 damage. Apply 2 Vulnerable. Gain 6 Shield.",
            "Inflige 8 de daño. Aplica Vulnerable 2. Gana 6 de Escudo.",
        ),
        CardId::BreachSignal => localized_text(
            language,
            "Deal 7 damage. Draw 1. Apply 2 Vulnerable.",
            "Inflige 7 de daño. Roba 1. Aplica Vulnerable 2.",
        ),
        CardId::BreachSignalPlus => localized_text(
            language,
            "Deal 9 damage. Draw 1. Apply 2 Vulnerable.",
            "Inflige 9 de daño. Roba 1. Aplica Vulnerable 2.",
        ),
        CardId::AnchorLoop => localized_text(
            language,
            "Gain 14 Shield. Draw 2.",
            "Gana 14 de Escudo. Roba 2.",
        ),
        CardId::AnchorLoopPlus => localized_text(
            language,
            "Gain 17 Shield. Draw 2.",
            "Gana 17 de Escudo. Roba 2.",
        ),
        CardId::ExecutionBeam => localized_text(language, "Deal 20 damage.", "Inflige 20 de daño."),
        CardId::ExecutionBeamPlus => {
            localized_text(language, "Deal 26 damage.", "Inflige 26 de daño.")
        }
        CardId::ChainBarrage => localized_text(
            language,
            "Deal 8 damage twice. Apply Bleed 2.",
            "Inflige 8 de daño dos veces. Aplica Sangrado 2.",
        ),
        CardId::ChainBarragePlus => localized_text(
            language,
            "Deal 10 damage twice. Apply Bleed 2.",
            "Inflige 10 de daño dos veces. Aplica Sangrado 2.",
        ),
        CardId::FortressMatrix => localized_text(
            language,
            "Gain 16 Shield. Draw 1.",
            "Gana 16 de Escudo. Roba 1.",
        ),
        CardId::FortressMatrixPlus => localized_text(
            language,
            "Gain 20 Shield. Draw 1.",
            "Gana 20 de Escudo. Roba 1.",
        ),
        CardId::OverwatchGrid => localized_text(
            language,
            "Gain 18 Shield. Draw 2.",
            "Gana 18 de Escudo. Roba 2.",
        ),
        CardId::OverwatchGridPlus => localized_text(
            language,
            "Gain 22 Shield. Draw 2.",
            "Gana 22 de Escudo. Roba 2.",
        ),
        CardId::ZeroPoint => localized_text(
            language,
            "Deal 10 damage. Draw 1. Apply 2 Vulnerable.",
            "Inflige 10 de daño. Roba 1. Aplica Vulnerable 2.",
        ),
        CardId::ZeroPointPlus => localized_text(
            language,
            "Deal 14 damage. Draw 1. Apply 2 Vulnerable.",
            "Inflige 14 de daño. Roba 1. Aplica Vulnerable 2.",
        ),
    }
}

pub(crate) fn localized_module_def(id: ModuleId, language: Language) -> ModuleDef {
    let mut def = module_def(id);
    def.name = localized_module_name(id, language);
    def.description = localized_module_description(id, language);
    def
}

pub(crate) fn localized_module_name(id: ModuleId, language: Language) -> &'static str {
    match id {
        ModuleId::AegisDrive => localized_text(language, "Aegis Drive", "Aegis Drive"),
        ModuleId::TargetingRelay => {
            localized_text(language, "Targeting Relay", "Relé de Apuntamiento")
        }
        ModuleId::Nanoforge => localized_text(language, "Nanoforge", "Nanoforge"),
        ModuleId::CapacitorBank => {
            localized_text(language, "Capacitor Bank", "Banco de Capacitores")
        }
        ModuleId::PrismScope => localized_text(language, "Prism Scope", "Visor Prisma"),
        ModuleId::SalvageLedger => {
            localized_text(language, "Salvage Ledger", "Registro de Chatarra")
        }
        ModuleId::OverclockCore => localized_text(language, "Overclock Core", "Núcleo Overclock"),
        ModuleId::SuppressionField => {
            localized_text(language, "Suppression Field", "Campo de Supresión")
        }
        ModuleId::RecoveryMatrix => {
            localized_text(language, "Recovery Matrix", "Matriz de Recuperación")
        }
    }
}

pub(crate) fn localized_module_description(id: ModuleId, language: Language) -> &'static str {
    match id {
        ModuleId::AegisDrive => localized_text(
            language,
            "Start each combat with 5 Shield.",
            "Comienzas cada combate con 5 de Escudo.",
        ),
        ModuleId::TargetingRelay => localized_text(
            language,
            "Start each combat by applying Vulnerable 1 to the first enemy.",
            "Al comienzo de cada combate, aplica Vulnerable 1 al primer enemigo.",
        ),
        ModuleId::Nanoforge => localized_text(
            language,
            "After each victory, recover 3 HP.",
            "Tras cada victoria, recupera 3 HP.",
        ),
        ModuleId::CapacitorBank => localized_text(
            language,
            "Start each combat with Strength 1.",
            "Comienzas cada combate con Fuerza 1.",
        ),
        ModuleId::PrismScope => localized_text(
            language,
            "Start each combat by applying Vulnerable 1 to all enemies.",
            "Al comienzo de cada combate, aplica Vulnerable 1 a todos los enemigos.",
        ),
        ModuleId::SalvageLedger => localized_text(
            language,
            "After each victory, gain 4 additional Credits.",
            "Tras cada victoria, gana 4 Créditos adicionales.",
        ),
        ModuleId::OverclockCore => localized_text(
            language,
            "Start each combat with 1 extra Energy.",
            "Comienzas cada combate con 1 de Energía extra.",
        ),
        ModuleId::SuppressionField => localized_text(
            language,
            "Start each combat by applying Weak 1 to all enemies.",
            "Al comienzo de cada combate, aplica Débil 1 a todos los enemigos.",
        ),
        ModuleId::RecoveryMatrix => localized_text(
            language,
            "After each victory, recover 5 HP.",
            "Tras cada victoria, recupera 5 HP.",
        ),
    }
}

pub(crate) fn localized_event_def(id: EventId, language: Language) -> EventDef {
    let mut def = event_def(id);
    def.title = localized_event_title(id, language);
    def.flavor = localized_event_flavor(id, language);
    def
}

pub(crate) fn localized_event_title(id: EventId, language: Language) -> &'static str {
    match id {
        EventId::SalvageCache => localized_text(language, "Salvage Cache", "Alijo de Chatarra"),
        EventId::ClinicPod => localized_text(language, "Clinic Pod", "Cápsula Clínica"),
        EventId::PrototypeRack => {
            localized_text(language, "Prototype Rack", "Bastidor de Prototipos")
        }
    }
}

pub(crate) fn localized_event_flavor(id: EventId, language: Language) -> &'static str {
    match id {
        EventId::SalvageCache => localized_text(
            language,
            "A drift crate hums beneath a half-collapsed service gantry.",
            "Un contenedor a la deriva zumba bajo una pasarela de servicio medio derrumbada.",
        ),
        EventId::ClinicPod => localized_text(
            language,
            "An intact med pod still cycles a pale diagnostic glow.",
            "Una cápsula médica intacta aún emite un tenue resplandor de diagnóstico.",
        ),
        EventId::PrototypeRack => localized_text(
            language,
            "A sealed rack flickers between proven gear and live-fire prototypes.",
            "Un bastidor sellado alterna entre equipo probado y prototipos de fuego real.",
        ),
    }
}

pub(crate) fn localized_event_choice_title(
    event: EventId,
    choice_index: usize,
    language: Language,
) -> Option<&'static str> {
    Some(match (event, choice_index) {
        (EventId::SalvageCache, 0) => {
            localized_text(language, "Take the clean parts", "Tomar las piezas útiles")
        }
        (EventId::SalvageCache, 1) => localized_text(
            language,
            "Cut the safety seals",
            "Abrir los sellos de seguridad",
        ),
        (EventId::ClinicPod, 0) => localized_text(
            language,
            "Run the recovery cycle",
            "Activar el ciclo de recuperación",
        ),
        (EventId::ClinicPod, 1) => {
            localized_text(language, "Overclock the chassis", "Sobrecargar el armazón")
        }
        (EventId::PrototypeRack, 0) => localized_text(
            language,
            "Take the stable shell",
            "Tomar el armazón estable",
        ),
        (EventId::PrototypeRack, 1) => localized_text(
            language,
            "Take the live prototype",
            "Tomar el prototipo en pruebas",
        ),
        _ => return None,
    })
}

pub(crate) fn localized_event_choice_body(
    event: EventId,
    choice_index: usize,
    level: usize,
    language: Language,
) -> Option<String> {
    match event_choice_effect(event, choice_index, level)? {
        EventChoiceEffect::GainCredits(credits) => Some(match language {
            Language::English => format!("Gain {credits} Credits."),
            Language::Spanish => format!("Gana {credits} Créditos."),
        }),
        EventChoiceEffect::LoseHpGainCredits {
            lose_hp,
            gain_credits,
        } => Some(match language {
            Language::English => format!("Lose {lose_hp} HP. Gain {gain_credits} Credits."),
            Language::Spanish => format!("Pierde {lose_hp} HP. Gana {gain_credits} Créditos."),
        }),
        EventChoiceEffect::Heal(amount) => Some(match language {
            Language::English => format!("Recover {amount} HP."),
            Language::Spanish => format!("Recupera {amount} HP."),
        }),
        EventChoiceEffect::LoseHpGainMaxHp {
            lose_hp,
            gain_max_hp,
        } => Some(match language {
            Language::English => format!("Lose {lose_hp} HP. Gain {gain_max_hp} max HP."),
            Language::Spanish => {
                format!("Pierde {lose_hp} HP. Gana {gain_max_hp} de HP máximo.")
            }
        }),
        EventChoiceEffect::AddCard(card) => Some(match language {
            Language::English => {
                format!("Add {} to your deck.", localized_card_name(card, language))
            }
            Language::Spanish => {
                format!("Añade {} a tu mazo.", localized_card_name(card, language))
            }
        }),
        EventChoiceEffect::LoseHpAddCard { lose_hp, card } => Some(match language {
            Language::English => format!(
                "Lose {lose_hp} HP. Add {} to your deck.",
                localized_card_name(card, language)
            ),
            Language::Spanish => format!(
                "Pierde {lose_hp} HP. Añade {} a tu mazo.",
                localized_card_name(card, language)
            ),
        }),
    }
}

pub(crate) fn localized_enemy_name(profile: EnemyProfileId, language: Language) -> &'static str {
    match profile {
        EnemyProfileId::ScoutDrone => localized_text(language, "Scout Drone", "Dron Explorador"),
        EnemyProfileId::NeedlerDrone => localized_text(language, "Needler Drone", "Dron Aguijón"),
        EnemyProfileId::RampartDrone => localized_text(language, "Rampart Drone", "Dron Bastión"),
        EnemyProfileId::SpineSentry => {
            localized_text(language, "Spine Sentry", "Centinela de Púas")
        }
        EnemyProfileId::PentaCore => localized_text(language, "Penta Core", "Núcleo Penta"),
        EnemyProfileId::VoltMantis => localized_text(language, "Volt Mantis", "Mantis de Voltaje"),
        EnemyProfileId::ShardWeaver => {
            localized_text(language, "Shard Weaver", "Tejedor de Fragmentos")
        }
        EnemyProfileId::PrismArray => localized_text(language, "Prism Array", "Matriz Prisma"),
        EnemyProfileId::GlassBishop => localized_text(language, "Glass Bishop", "Obispo de Vidrio"),
        EnemyProfileId::HexarchCore => localized_text(language, "Hexarch Core", "Núcleo Hexarch"),
        EnemyProfileId::NullRaider => localized_text(language, "Null Raider", "Asaltante Null"),
        EnemyProfileId::RiftStalker => {
            localized_text(language, "Rift Stalker", "Acechador de la Grieta")
        }
        EnemyProfileId::BulwarkDrone => localized_text(language, "Bulwark Drone", "Dron Baluarte"),
        EnemyProfileId::RiftBastion => {
            localized_text(language, "Rift Bastion", "Bastión de la Grieta")
        }
        EnemyProfileId::HeptarchCore => {
            localized_text(language, "Heptarch Core", "Núcleo Heptarch")
        }
    }
}

pub(crate) fn localized_enemy_intent(
    profile: EnemyProfileId,
    index: usize,
    language: Language,
) -> EnemyIntent {
    let mut intent = enemy_intent(profile, index);
    let translated = match (profile, index % 3) {
        (EnemyProfileId::ScoutDrone, 0) => (
            localized_text(language, "Shock Needle", "Aguja de Choque"),
            localized_text(language, "Deal 5 damage.", "Inflige 5 de daño."),
        ),
        (EnemyProfileId::ScoutDrone, 1) => (
            localized_text(language, "Crossfire", "Fuego Cruzado"),
            localized_text(
                language,
                "Deal 3 damage twice.",
                "Inflige 3 de daño dos veces.",
            ),
        ),
        (EnemyProfileId::ScoutDrone, _) => (
            localized_text(language, "Brace Cycle", "Ciclo de Refuerzo"),
            localized_text(
                language,
                "Gain 4 Shield. Gain Strength 1.",
                "Gana 4 de Escudo. Gana Fuerza 1.",
            ),
        ),
        (EnemyProfileId::NeedlerDrone, 0) => (
            localized_text(language, "Needle Tap", "Toque de Aguijón"),
            localized_text(
                language,
                "Deal 4 damage. Apply Bleed 1.",
                "Inflige 4 de daño. Aplica Sangrado 1.",
            ),
        ),
        (EnemyProfileId::NeedlerDrone, 1) => (
            localized_text(language, "Split Sting", "Picadura Múltiple"),
            localized_text(
                language,
                "Deal 2 damage three times.",
                "Inflige 2 de daño tres veces.",
            ),
        ),
        (EnemyProfileId::NeedlerDrone, _) => (
            localized_text(language, "Stabilize", "Estabilizar"),
            localized_text(language, "Gain 4 Shield.", "Gana 4 de Escudo."),
        ),
        (EnemyProfileId::RampartDrone, 0) => (
            localized_text(language, "Ram Plate", "Placa de Choque"),
            localized_text(language, "Deal 8 damage.", "Inflige 8 de daño."),
        ),
        (EnemyProfileId::RampartDrone, 1) => (
            localized_text(language, "Pressure Clamp", "Mordaza de Presión"),
            localized_text(
                language,
                "Deal 5 damage. Apply Weak 1.",
                "Inflige 5 de daño. Aplica Débil 1.",
            ),
        ),
        (EnemyProfileId::RampartDrone, _) => (
            localized_text(language, "Reinforce Wall", "Muro Reforzado"),
            localized_text(
                language,
                "Gain 8 Shield. Next hit applies Bleed 2.",
                "Gana 8 de Escudo. El siguiente golpe aplica Sangrado 2.",
            ),
        ),
        (EnemyProfileId::SpineSentry, 0) => (
            localized_text(language, "Spine Rack", "Bastidor de Púas"),
            localized_text(
                language,
                "Deal 4 damage twice. Apply Bleed 1.",
                "Inflige 4 de daño dos veces. Aplica Sangrado 1.",
            ),
        ),
        (EnemyProfileId::SpineSentry, 1) => (
            localized_text(language, "Target Lock", "Fijación de Blanco"),
            localized_text(
                language,
                "Deal 7 damage. Apply 1 Vulnerable.",
                "Inflige 7 de daño. Aplica Vulnerable 1.",
            ),
        ),
        (EnemyProfileId::SpineSentry, _) => (
            localized_text(language, "Quill Plating", "Blindaje de Púas"),
            localized_text(language, "Gain 9 Shield.", "Gana 9 de Escudo."),
        ),
        (EnemyProfileId::PentaCore, 0) => (
            localized_text(language, "Target Prism", "Prisma de Fijación"),
            localized_text(
                language,
                "Deal 7 damage. Apply 1 Vulnerable.",
                "Inflige 7 de daño. Aplica Vulnerable 1.",
            ),
        ),
        (EnemyProfileId::PentaCore, 1) => (
            localized_text(language, "Penta Bulwark", "Baluarte Penta"),
            localized_text(
                language,
                "Gain 10 Shield. Next hit applies Bleed 2.",
                "Gana 10 de Escudo. El siguiente golpe aplica Sangrado 2.",
            ),
        ),
        (EnemyProfileId::PentaCore, _) => (
            localized_text(language, "Split Lattice", "Trama Fragmentada"),
            localized_text(
                language,
                "Deal 4 damage three times.",
                "Inflige 4 de daño tres veces.",
            ),
        ),
        (EnemyProfileId::VoltMantis, 0) => (
            localized_text(language, "Arc Cut", "Corte de Arco"),
            localized_text(language, "Deal 8 damage.", "Inflige 8 de daño."),
        ),
        (EnemyProfileId::VoltMantis, 1) => (
            localized_text(language, "Arc Lash", "Látigo de Arco"),
            localized_text(
                language,
                "Deal 4 damage twice.",
                "Inflige 4 de daño dos veces.",
            ),
        ),
        (EnemyProfileId::VoltMantis, _) => (
            localized_text(language, "Charge Shell", "Caparazón de Carga"),
            localized_text(language, "Gain 7 Shield.", "Gana 7 de Escudo."),
        ),
        (EnemyProfileId::ShardWeaver, 0) => (
            localized_text(language, "Glass Cut", "Corte de Vidrio"),
            localized_text(
                language,
                "Deal 6 damage. Apply 1 Vulnerable.",
                "Inflige 6 de daño. Aplica Vulnerable 1.",
            ),
        ),
        (EnemyProfileId::ShardWeaver, 1) => (
            localized_text(language, "Mirror Volley", "Andanada Reflejada"),
            localized_text(
                language,
                "Deal 3 damage twice. Gain 4 Shield.",
                "Inflige 3 de daño dos veces. Gana 4 de Escudo.",
            ),
        ),
        (EnemyProfileId::ShardWeaver, _) => (
            localized_text(language, "Refocus", "Reenfocar"),
            localized_text(
                language,
                "Gain 8 Shield. Apply Frail 1.",
                "Gana 8 de Escudo. Aplica Frágil 1.",
            ),
        ),
        (EnemyProfileId::PrismArray, 0) => (
            localized_text(language, "Prism Bite", "Mordida Prisma"),
            localized_text(
                language,
                "Deal 7 damage. Apply 1 Vulnerable.",
                "Inflige 7 de daño. Aplica Vulnerable 1.",
            ),
        ),
        (EnemyProfileId::PrismArray, 1) => (
            localized_text(language, "Ion Salvo", "Salva Iónica"),
            localized_text(
                language,
                "Deal 5 damage twice.",
                "Inflige 5 de daño dos veces.",
            ),
        ),
        (EnemyProfileId::PrismArray, _) => (
            localized_text(language, "Prism Guard", "Guardia Prismática"),
            localized_text(language, "Gain 10 Shield.", "Gana 10 de Escudo."),
        ),
        (EnemyProfileId::GlassBishop, 0) => (
            localized_text(language, "Shatter Beam", "Rayo Astillado"),
            localized_text(
                language,
                "Deal 8 damage. Apply 1 Vulnerable.",
                "Inflige 8 de daño. Aplica Vulnerable 1.",
            ),
        ),
        (EnemyProfileId::GlassBishop, 1) => (
            localized_text(language, "Split Halo", "Halo Partido"),
            localized_text(
                language,
                "Deal 5 damage twice. Gain 4 Shield.",
                "Inflige 5 de daño dos veces. Gana 4 de Escudo.",
            ),
        ),
        (EnemyProfileId::GlassBishop, _) => (
            localized_text(language, "Faceted Ward", "Barrera Facetada"),
            localized_text(
                language,
                "Gain 10 Shield. Apply Bleed 1.",
                "Gana 10 de Escudo. Aplica Sangrado 1.",
            ),
        ),
        (EnemyProfileId::HexarchCore, 0) => (
            localized_text(language, "Hex Shell", "Coraza Hex"),
            localized_text(
                language,
                "Gain 12 Shield. Apply 2 Vulnerable.",
                "Gana 12 de Escudo. Aplica Vulnerable 2.",
            ),
        ),
        (EnemyProfileId::HexarchCore, 1) => (
            localized_text(language, "Hex Breaker", "Ruptor Hex"),
            localized_text(language, "Deal 15 damage.", "Inflige 15 de daño."),
        ),
        (EnemyProfileId::HexarchCore, _) => (
            localized_text(language, "Hex Volley", "Andanada Hex"),
            localized_text(
                language,
                "Deal 5 damage three times.",
                "Inflige 5 de daño tres veces.",
            ),
        ),
        (EnemyProfileId::NullRaider, 0) => (
            localized_text(language, "Null Shot", "Disparo Null"),
            localized_text(language, "Deal 10 damage.", "Inflige 10 de daño."),
        ),
        (EnemyProfileId::NullRaider, 1) => (
            localized_text(language, "Chain Burst", "Ráfaga en Cadena"),
            localized_text(
                language,
                "Deal 5 damage twice.",
                "Inflige 5 de daño dos veces.",
            ),
        ),
        (EnemyProfileId::NullRaider, _) => (
            localized_text(language, "Null Guard", "Guardia Null"),
            localized_text(language, "Gain 9 Shield.", "Gana 9 de Escudo."),
        ),
        (EnemyProfileId::RiftStalker, 0) => (
            localized_text(language, "Rift Claw", "Garra de la Grieta"),
            localized_text(
                language,
                "Deal 9 damage. Apply Bleed 1.",
                "Inflige 9 de daño. Aplica Sangrado 1.",
            ),
        ),
        (EnemyProfileId::RiftStalker, 1) => (
            localized_text(language, "Rend Salvo", "Salva Desgarradora"),
            localized_text(
                language,
                "Deal 5 damage twice.",
                "Inflige 5 de daño dos veces.",
            ),
        ),
        (EnemyProfileId::RiftStalker, _) => (
            localized_text(language, "Lock Anchor", "Ancla de Fijación"),
            localized_text(
                language,
                "Gain 10 Shield. Apply 1 Vulnerable.",
                "Gana 10 de Escudo. Aplica Vulnerable 1.",
            ),
        ),
        (EnemyProfileId::BulwarkDrone, 0) => (
            localized_text(language, "Bulwark Hammer", "Martillo Baluarte"),
            localized_text(language, "Deal 11 damage.", "Inflige 11 de daño."),
        ),
        (EnemyProfileId::BulwarkDrone, 1) => (
            localized_text(language, "Lock Volley", "Andanada de Fijación"),
            localized_text(
                language,
                "Deal 6 damage twice.",
                "Inflige 6 de daño dos veces.",
            ),
        ),
        (EnemyProfileId::BulwarkDrone, _) => (
            localized_text(language, "Bulwark Seal", "Sello Baluarte"),
            localized_text(
                language,
                "Gain 12 Shield. Apply 1 Vulnerable.",
                "Gana 12 de Escudo. Aplica Vulnerable 1.",
            ),
        ),
        (EnemyProfileId::RiftBastion, 0) => (
            localized_text(language, "Grav Hammer", "Martillo Gravitatorio"),
            localized_text(language, "Deal 12 damage.", "Inflige 12 de daño."),
        ),
        (EnemyProfileId::RiftBastion, 1) => (
            localized_text(language, "Collapse Grid", "Malla de Colapso"),
            localized_text(
                language,
                "Deal 6 damage twice. Apply 1 Vulnerable.",
                "Inflige 6 de daño dos veces. Aplica Vulnerable 1.",
            ),
        ),
        (EnemyProfileId::RiftBastion, _) => (
            localized_text(language, "Anchor Field", "Campo de Anclaje"),
            localized_text(
                language,
                "Gain 12 Shield. Next hit applies Bleed 3.",
                "Gana 12 de Escudo. El siguiente golpe aplica Sangrado 3.",
            ),
        ),
        (EnemyProfileId::HeptarchCore, 0) => (
            localized_text(language, "Singularity Shell", "Coraza de Singularidad"),
            localized_text(
                language,
                "Gain 16 Shield. Next hit applies Bleed 3.",
                "Gana 16 de Escudo. El siguiente golpe aplica Sangrado 3.",
            ),
        ),
        (EnemyProfileId::HeptarchCore, 1) => (
            localized_text(language, "Array Collapse", "Colapso de Matriz"),
            localized_text(
                language,
                "Deal 8 damage twice. Apply 1 Vulnerable.",
                "Inflige 8 de daño dos veces. Aplica Vulnerable 1.",
            ),
        ),
        (EnemyProfileId::HeptarchCore, _) => (
            localized_text(language, "Crown Breaker", "Quebracoronas"),
            localized_text(language, "Deal 20 damage.", "Inflige 20 de daño."),
        ),
    };
    intent.name = translated.0;
    intent.summary = translated.1;
    intent
}

pub(crate) fn event_for_level(seed: u64, level: usize) -> EventId {
    let mut events = [
        EventId::SalvageCache,
        EventId::ClinicPod,
        EventId::PrototypeRack,
    ];
    events.sort_by_key(|event| event_roll_key(seed, *event));
    events[level.clamp(1, events.len()) - 1]
}

fn reward_roll_key(seed: u64, card: CardId) -> u64 {
    let mut x = seed ^ (card as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

fn event_roll_key(seed: u64, event: EventId) -> u64 {
    let mut x = seed ^ (event as u64 + 1).wrapping_mul(0x517C_C1B7_2722_0A95);
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn enemy_name(profile: EnemyProfileId) -> &'static str {
    match profile {
        EnemyProfileId::ScoutDrone => "Scout Drone",
        EnemyProfileId::NeedlerDrone => "Needler Drone",
        EnemyProfileId::RampartDrone => "Rampart Drone",
        EnemyProfileId::SpineSentry => "Spine Sentry",
        EnemyProfileId::PentaCore => "Penta Core",
        EnemyProfileId::VoltMantis => "Volt Mantis",
        EnemyProfileId::ShardWeaver => "Shard Weaver",
        EnemyProfileId::PrismArray => "Prism Array",
        EnemyProfileId::GlassBishop => "Glass Bishop",
        EnemyProfileId::HexarchCore => "Hexarch Core",
        EnemyProfileId::NullRaider => "Null Raider",
        EnemyProfileId::RiftStalker => "Rift Stalker",
        EnemyProfileId::BulwarkDrone => "Bulwark Drone",
        EnemyProfileId::RiftBastion => "Rift Bastion",
        EnemyProfileId::HeptarchCore => "Heptarch Core",
    }
}

pub(crate) fn enemy_intent(profile: EnemyProfileId, index: usize) -> EnemyIntent {
    match (profile, index % 3) {
        (EnemyProfileId::ScoutDrone, 0) => EnemyIntent {
            name: "Shock Needle",
            summary: "Deal 5 damage.",
            damage: 5,
            hits: 1,
            gain_block: 0,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::ScoutDrone, 1) => EnemyIntent {
            name: "Crossfire",
            summary: "Deal 3 damage twice.",
            damage: 3,
            hits: 2,
            gain_block: 0,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::ScoutDrone, _) => EnemyIntent {
            name: "Brace Cycle",
            summary: "Gain 4 Shield. Gain Strength 1.",
            damage: 0,
            hits: 0,
            gain_block: 4,
            gain_strength: 1,
            prime_bleed: 0,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::NeedlerDrone, 0) => EnemyIntent {
            name: "Needle Tap",
            summary: "Deal 4 damage. Apply Bleed 1.",
            damage: 4,
            hits: 1,
            gain_block: 0,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 1,
        },
        (EnemyProfileId::NeedlerDrone, 1) => EnemyIntent {
            name: "Split Sting",
            summary: "Deal 2 damage three times.",
            damage: 2,
            hits: 3,
            gain_block: 0,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::NeedlerDrone, _) => EnemyIntent {
            name: "Stabilize",
            summary: "Gain 4 Shield.",
            damage: 0,
            hits: 0,
            gain_block: 4,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::RampartDrone, 0) => EnemyIntent {
            name: "Ram Plate",
            summary: "Deal 8 damage.",
            damage: 8,
            hits: 1,
            gain_block: 0,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::RampartDrone, 1) => EnemyIntent {
            name: "Pressure Clamp",
            summary: "Deal 5 damage. Apply Weak 1.",
            damage: 5,
            hits: 1,
            gain_block: 0,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 0,
            apply_weak: 1,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::RampartDrone, _) => EnemyIntent {
            name: "Reinforce Wall",
            summary: "Gain 8 Shield. Next hit applies Bleed 2.",
            damage: 0,
            hits: 0,
            gain_block: 8,
            gain_strength: 0,
            prime_bleed: 2,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::SpineSentry, 0) => EnemyIntent {
            name: "Spine Rack",
            summary: "Deal 4 damage twice. Apply Bleed 1.",
            damage: 4,
            hits: 2,
            gain_block: 0,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 1,
        },
        (EnemyProfileId::SpineSentry, 1) => EnemyIntent {
            name: "Target Lock",
            summary: "Deal 7 damage. Apply 1 Vulnerable.",
            damage: 7,
            hits: 1,
            gain_block: 0,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 1,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::SpineSentry, _) => EnemyIntent {
            name: "Quill Plating",
            summary: "Gain 9 Shield.",
            damage: 0,
            hits: 0,
            gain_block: 9,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::PentaCore, 0) => EnemyIntent {
            name: "Target Prism",
            summary: "Deal 7 damage. Apply 1 Vulnerable.",
            damage: 7,
            hits: 1,
            gain_block: 0,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 1,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::PentaCore, 1) => EnemyIntent {
            name: "Penta Bulwark",
            summary: "Gain 10 Shield. Next hit applies Bleed 2.",
            damage: 0,
            hits: 0,
            gain_block: 10,
            gain_strength: 0,
            prime_bleed: 2,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::PentaCore, _) => EnemyIntent {
            name: "Split Lattice",
            summary: "Deal 4 damage three times.",
            damage: 4,
            hits: 3,
            gain_block: 0,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::VoltMantis, 0) => EnemyIntent {
            name: "Arc Cut",
            summary: "Deal 8 damage.",
            damage: 8,
            hits: 1,
            gain_block: 0,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::VoltMantis, 1) => EnemyIntent {
            name: "Arc Lash",
            summary: "Deal 4 damage twice.",
            damage: 4,
            hits: 2,
            gain_block: 0,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::VoltMantis, _) => EnemyIntent {
            name: "Charge Shell",
            summary: "Gain 7 Shield.",
            damage: 0,
            hits: 0,
            gain_block: 7,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::ShardWeaver, 0) => EnemyIntent {
            name: "Glass Cut",
            summary: "Deal 6 damage. Apply 1 Vulnerable.",
            damage: 6,
            hits: 1,
            gain_block: 0,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 1,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::ShardWeaver, 1) => EnemyIntent {
            name: "Mirror Volley",
            summary: "Deal 3 damage twice. Gain 4 Shield.",
            damage: 3,
            hits: 2,
            gain_block: 4,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::ShardWeaver, _) => EnemyIntent {
            name: "Refocus",
            summary: "Gain 8 Shield. Apply Frail 1.",
            damage: 0,
            hits: 0,
            gain_block: 8,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 1,
            apply_bleed: 0,
        },
        (EnemyProfileId::PrismArray, 0) => EnemyIntent {
            name: "Prism Bite",
            summary: "Deal 7 damage. Apply 1 Vulnerable.",
            damage: 7,
            hits: 1,
            gain_block: 0,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 1,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::PrismArray, 1) => EnemyIntent {
            name: "Ion Salvo",
            summary: "Deal 5 damage twice.",
            damage: 5,
            hits: 2,
            gain_block: 0,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::PrismArray, _) => EnemyIntent {
            name: "Prism Guard",
            summary: "Gain 10 Shield.",
            damage: 0,
            hits: 0,
            gain_block: 10,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::GlassBishop, 0) => EnemyIntent {
            name: "Shatter Beam",
            summary: "Deal 8 damage. Apply 1 Vulnerable.",
            damage: 8,
            hits: 1,
            gain_block: 0,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 1,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::GlassBishop, 1) => EnemyIntent {
            name: "Split Halo",
            summary: "Deal 5 damage twice. Gain 4 Shield.",
            damage: 5,
            hits: 2,
            gain_block: 4,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::GlassBishop, _) => EnemyIntent {
            name: "Faceted Ward",
            summary: "Gain 10 Shield. Apply Bleed 1.",
            damage: 0,
            hits: 0,
            gain_block: 10,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 1,
        },
        (EnemyProfileId::HexarchCore, 0) => EnemyIntent {
            name: "Hex Shell",
            summary: "Gain 12 Shield. Apply 2 Vulnerable.",
            damage: 0,
            hits: 0,
            gain_block: 12,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 2,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::HexarchCore, 1) => EnemyIntent {
            name: "Hex Breaker",
            summary: "Deal 15 damage.",
            damage: 15,
            hits: 1,
            gain_block: 0,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::HexarchCore, _) => EnemyIntent {
            name: "Hex Volley",
            summary: "Deal 5 damage three times.",
            damage: 5,
            hits: 3,
            gain_block: 0,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::NullRaider, 0) => EnemyIntent {
            name: "Null Shot",
            summary: "Deal 10 damage.",
            damage: 10,
            hits: 1,
            gain_block: 0,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::NullRaider, 1) => EnemyIntent {
            name: "Chain Burst",
            summary: "Deal 5 damage twice.",
            damage: 5,
            hits: 2,
            gain_block: 0,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::NullRaider, _) => EnemyIntent {
            name: "Null Guard",
            summary: "Gain 9 Shield.",
            damage: 0,
            hits: 0,
            gain_block: 9,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::RiftStalker, 0) => EnemyIntent {
            name: "Rift Claw",
            summary: "Deal 9 damage. Apply Bleed 1.",
            damage: 9,
            hits: 1,
            gain_block: 0,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 1,
        },
        (EnemyProfileId::RiftStalker, 1) => EnemyIntent {
            name: "Rend Salvo",
            summary: "Deal 5 damage twice.",
            damage: 5,
            hits: 2,
            gain_block: 0,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::RiftStalker, _) => EnemyIntent {
            name: "Lock Anchor",
            summary: "Gain 10 Shield. Apply 1 Vulnerable.",
            damage: 0,
            hits: 0,
            gain_block: 10,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 1,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::BulwarkDrone, 0) => EnemyIntent {
            name: "Bulwark Hammer",
            summary: "Deal 11 damage.",
            damage: 11,
            hits: 1,
            gain_block: 0,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::BulwarkDrone, 1) => EnemyIntent {
            name: "Lock Volley",
            summary: "Deal 6 damage twice.",
            damage: 6,
            hits: 2,
            gain_block: 0,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::BulwarkDrone, _) => EnemyIntent {
            name: "Bulwark Seal",
            summary: "Gain 12 Shield. Apply 1 Vulnerable.",
            damage: 0,
            hits: 0,
            gain_block: 12,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 1,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::RiftBastion, 0) => EnemyIntent {
            name: "Grav Hammer",
            summary: "Deal 12 damage.",
            damage: 12,
            hits: 1,
            gain_block: 0,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::RiftBastion, 1) => EnemyIntent {
            name: "Collapse Grid",
            summary: "Deal 6 damage twice. Apply 1 Vulnerable.",
            damage: 6,
            hits: 2,
            gain_block: 0,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 1,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::RiftBastion, _) => EnemyIntent {
            name: "Anchor Field",
            summary: "Gain 12 Shield. Next hit applies Bleed 3.",
            damage: 0,
            hits: 0,
            gain_block: 12,
            gain_strength: 0,
            prime_bleed: 3,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::HeptarchCore, 0) => EnemyIntent {
            name: "Singularity Shell",
            summary: "Gain 16 Shield. Next hit applies Bleed 3.",
            damage: 0,
            hits: 0,
            gain_block: 16,
            gain_strength: 0,
            prime_bleed: 3,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::HeptarchCore, 1) => EnemyIntent {
            name: "Array Collapse",
            summary: "Deal 8 damage twice. Apply 1 Vulnerable.",
            damage: 8,
            hits: 2,
            gain_block: 0,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 1,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
        (EnemyProfileId::HeptarchCore, _) => EnemyIntent {
            name: "Crown Breaker",
            summary: "Deal 20 damage.",
            damage: 20,
            hits: 1,
            gain_block: 0,
            gain_strength: 0,
            prime_bleed: 0,
            apply_expose: 0,
            apply_weak: 0,
            apply_frail: 0,
            apply_bleed: 0,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PRIMARY_SEED: u64 = 0x0BAD_5EED;
    const TEST_ALT_SEED: u64 = 0xDEAD_BEEF;
    const TEST_MODULE_SEED: u64 = 0xC0DE_CAFE;
    const TEST_BOSS_REWARD_SEED: u64 = 0xBAAD_F00D;
    const TEST_ELITE_REWARD_SEED: u64 = 0xFACE_FEED;
    const TEST_SHOP_SEED: u64 = 0xD15C_A11C;

    #[test]
    fn event_order_is_deterministic_and_non_repeating() {
        let a = [
            event_for_level(TEST_PRIMARY_SEED, 1),
            event_for_level(TEST_PRIMARY_SEED, 2),
            event_for_level(TEST_PRIMARY_SEED, 3),
        ];
        let b = [
            event_for_level(TEST_PRIMARY_SEED, 1),
            event_for_level(TEST_PRIMARY_SEED, 2),
            event_for_level(TEST_PRIMARY_SEED, 3),
        ];
        let mut unique = a.to_vec();
        unique.sort_by_key(|event| *event as u8);
        unique.dedup();

        assert_eq!(a, b);
        assert_eq!(unique.len(), 3);
    }

    #[test]
    fn event_choices_scale_with_level() {
        assert_eq!(
            event_choice_effect(EventId::SalvageCache, 0, 1),
            Some(EventChoiceEffect::GainCredits(16))
        );
        assert_eq!(
            event_choice_effect(EventId::SalvageCache, 0, 3),
            Some(EventChoiceEffect::GainCredits(24))
        );
        assert_eq!(
            event_choice_effect(EventId::PrototypeRack, 0, 2),
            Some(EventChoiceEffect::AddCard(CardId::BarrierField))
        );
        assert_eq!(
            event_choice_effect(EventId::PrototypeRack, 1, 3),
            Some(EventChoiceEffect::LoseHpAddCard {
                lose_hp: 5,
                card: CardId::ZeroPoint,
            })
        );
    }

    #[test]
    fn starter_module_choices_always_include_all_three_modules() {
        let choices = starter_module_choices(TEST_MODULE_SEED);
        let mut sorted = choices.clone();
        sorted.sort_by_key(|module| *module as u8);
        sorted.dedup();

        assert_eq!(choices.len(), 3);
        assert_eq!(sorted.len(), 3);
        assert!(sorted.contains(&ModuleId::AegisDrive));
        assert!(sorted.contains(&ModuleId::TargetingRelay));
        assert!(sorted.contains(&ModuleId::Nanoforge));
    }

    #[test]
    fn starter_module_choice_order_is_fixed() {
        let a = starter_module_choices(TEST_PRIMARY_SEED);
        let b = starter_module_choices(TEST_ALT_SEED);

        assert_eq!(
            a,
            vec![
                ModuleId::Nanoforge,
                ModuleId::AegisDrive,
                ModuleId::TargetingRelay,
            ]
        );
        assert_eq!(a, b);
    }

    #[test]
    fn boss_module_choices_offer_three_unique_options_per_boss() {
        let boss_one = boss_module_choices(1);
        let boss_two = boss_module_choices(2);
        let mut boss_one_sorted = boss_one.clone();
        let mut boss_two_sorted = boss_two.clone();
        boss_one_sorted.sort_by_key(|module| *module as u8);
        boss_two_sorted.sort_by_key(|module| *module as u8);
        boss_one_sorted.dedup();
        boss_two_sorted.dedup();

        assert_eq!(boss_one.len(), 3);
        assert_eq!(boss_two.len(), 3);
        assert_eq!(boss_one_sorted.len(), 3);
        assert_eq!(boss_two_sorted.len(), 3);
        assert!(boss_one.iter().all(|module| !boss_two.contains(module)));
    }

    #[test]
    fn combat_rewards_include_the_new_staples() {
        let pool = reward_pool(RewardTier::Combat, 1);

        assert!(pool.contains(&CardId::PinpointJab));
        assert!(pool.contains(&CardId::SignalTap));
        assert!(pool.contains(&CardId::BurstArray));
        assert!(pool.contains(&CardId::CoverPulse));
    }

    #[test]
    fn boss_rewards_only_offer_boss_cards() {
        let choices = reward_choices(TEST_BOSS_REWARD_SEED, RewardTier::Boss, 3);
        let pool = reward_pool(RewardTier::Boss, 3);

        assert_eq!(choices.len(), 3);
        assert!(pool.contains(&CardId::ExecutionBeam));
        assert!(pool.contains(&CardId::ChainBarrage));
        assert!(pool.contains(&CardId::FortressMatrix));
        assert!(pool.contains(&CardId::OverwatchGrid));
        assert!(pool.contains(&CardId::ZeroPoint));
        assert!(choices.iter().all(|card| pool.contains(card)));
    }

    #[test]
    fn boss_rewards_vary_across_seeds_once_the_pool_is_expanded() {
        let mut seen = Vec::new();

        for seed in 0..16u64 {
            let choices = reward_choices(
                seed.wrapping_mul(0x9E37_79B9_7F4A_7C15),
                RewardTier::Boss,
                3,
            );
            if !seen.contains(&choices) {
                seen.push(choices);
            }
        }

        assert!(seen.len() > 1);
    }

    #[test]
    fn elite_rewards_are_distinct() {
        let choices = reward_choices(TEST_ELITE_REWARD_SEED, RewardTier::Elite, 2);
        let mut sorted = choices.clone();
        sorted.sort_by_key(|card| *card as u8);
        sorted.dedup();

        assert_eq!(choices.len(), 3);
        assert_eq!(sorted.len(), 3);
    }

    #[test]
    fn elite_rewards_scale_by_level() {
        let act_one_pool = reward_pool(RewardTier::Elite, 1);
        let act_two_pool = reward_pool(RewardTier::Elite, 2);
        let act_three_pool = reward_pool(RewardTier::Elite, 3);

        assert!(act_one_pool.contains(&CardId::RazorNet));
        assert!(act_one_pool.contains(&CardId::FracturePulse));
        assert!(act_one_pool.contains(&CardId::VectorLock));
        assert!(!act_one_pool.contains(&CardId::BreachSignal));
        assert!(!act_one_pool.contains(&CardId::AnchorLoop));

        assert!(act_two_pool.contains(&CardId::RazorNet));
        assert!(act_two_pool.contains(&CardId::FracturePulse));
        assert!(act_two_pool.contains(&CardId::VectorLock));
        assert!(act_two_pool.contains(&CardId::BreachSignal));
        assert!(!act_two_pool.contains(&CardId::AnchorLoop));

        assert!(act_three_pool.contains(&CardId::RazorNet));
        assert!(act_three_pool.contains(&CardId::FracturePulse));
        assert!(act_three_pool.contains(&CardId::VectorLock));
        assert!(act_three_pool.contains(&CardId::BreachSignal));
        assert!(act_three_pool.contains(&CardId::AnchorLoop));
    }

    #[test]
    fn shop_offers_stay_distinct_when_act_one_repeats_elite_tiers() {
        let offers = shop_offers(TEST_SHOP_SEED, 1);
        let mut cards: Vec<_> = offers.iter().map(|offer| offer.card).collect();
        cards.sort_by_key(|card| *card as u8);
        cards.dedup();

        assert_eq!(offers.len(), 3);
        assert_eq!(cards.len(), 3);
        assert_eq!(offers[0].price, 16);
        assert_eq!(offers[1].price, 24);
        assert_eq!(offers[2].price, 24);
    }

    #[test]
    fn later_act_shop_offers_include_boss_cards() {
        let act_two_offers = shop_offers(TEST_BOSS_REWARD_SEED, 2);
        let act_three_offers = shop_offers(TEST_BOSS_REWARD_SEED, 3);
        let boss_pool = reward_pool(RewardTier::Boss, 3);

        assert!(boss_pool.contains(&act_two_offers[2].card));
        assert!(boss_pool.contains(&act_three_offers[2].card));
        assert_eq!(act_two_offers[2].price, 40);
        assert_eq!(act_three_offers[2].price, 40);
    }

    #[test]
    fn base_cards_have_upgrades_but_upgraded_cards_do_not_chain() {
        assert_eq!(
            upgraded_card(CardId::FlareSlash),
            Some(CardId::FlareSlashPlus)
        );
        assert_eq!(
            upgraded_card(CardId::GuardStep),
            Some(CardId::GuardStepPlus)
        );
        assert_eq!(
            upgraded_card(CardId::ZeroPoint),
            Some(CardId::ZeroPointPlus)
        );
        assert_eq!(
            upgraded_card(CardId::PinpointJab),
            Some(CardId::PinpointJabPlus)
        );
        assert_eq!(
            upgraded_card(CardId::FracturePulse),
            Some(CardId::FracturePulsePlus)
        );
        assert_eq!(
            upgraded_card(CardId::ChainBarrage),
            Some(CardId::ChainBarragePlus)
        );
        assert_eq!(upgraded_card(CardId::FlareSlashPlus), None);
        assert_eq!(upgraded_card(CardId::ZeroPointPlus), None);
        assert_eq!(upgraded_card(CardId::OverwatchGridPlus), None);
    }

    #[test]
    fn new_cards_expose_expected_defs() {
        let pinpoint = card_def(CardId::PinpointJab);
        let signal = card_def(CardId::SignalTap);
        let pressure = card_def(CardId::PressurePoint);
        let barrier = card_def(CardId::BarrierField);
        let tactical = card_def(CardId::TacticalBurst);
        let fracture = card_def(CardId::FracturePulse);
        let overwatch = card_def(CardId::OverwatchGrid);

        assert_eq!(pinpoint.name, "Pinpoint Jab");
        assert_eq!(pinpoint.cost, 1);
        assert_eq!(pinpoint.target, CardTarget::Enemy);
        assert_eq!(signal.cost, 0);
        assert_eq!(signal.target, CardTarget::Enemy);
        assert_eq!(pressure.description, "Deal 4 damage. Apply Weak 1.");
        assert_eq!(barrier.target, CardTarget::Enemy);
        assert_eq!(barrier.description, "Gain 10 Shield. Apply Frail 1.");
        assert_eq!(tactical.target, CardTarget::SelfOnly);
        assert_eq!(tactical.description, "Draw 2. Gain Strength 1.");
        assert_eq!(fracture.cost, 2);
        assert_eq!(fracture.target, CardTarget::Enemy);
        assert_eq!(overwatch.cost, 2);
        assert_eq!(overwatch.target, CardTarget::SelfOnly);
    }

    #[test]
    fn early_enemy_intents_include_the_new_status_effects() {
        let scout_brace = enemy_intent(EnemyProfileId::ScoutDrone, 2);
        let rampart_clamp = enemy_intent(EnemyProfileId::RampartDrone, 1);
        let shard_refocus = enemy_intent(EnemyProfileId::ShardWeaver, 2);

        assert_eq!(scout_brace.gain_block, 4);
        assert_eq!(scout_brace.gain_strength, 1);
        assert_eq!(scout_brace.summary, "Gain 4 Shield. Gain Strength 1.");

        assert_eq!(rampart_clamp.damage, 5);
        assert_eq!(rampart_clamp.apply_weak, 1);
        assert_eq!(rampart_clamp.apply_expose, 0);
        assert_eq!(rampart_clamp.summary, "Deal 5 damage. Apply Weak 1.");

        assert_eq!(shard_refocus.gain_block, 8);
        assert_eq!(shard_refocus.apply_frail, 1);
        assert_eq!(shard_refocus.summary, "Gain 8 Shield. Apply Frail 1.");
    }

    #[test]
    fn enemy_profiles_expose_distinct_names_and_intents() {
        let act_one = enemy_intent(EnemyProfileId::PentaCore, 0);
        let act_three = enemy_intent(EnemyProfileId::HeptarchCore, 0);
        let elite = enemy_intent(EnemyProfileId::SpineSentry, 0);

        assert_eq!(enemy_name(EnemyProfileId::PentaCore), "Penta Core");
        assert_eq!(enemy_name(EnemyProfileId::HeptarchCore), "Heptarch Core");
        assert_eq!(enemy_name(EnemyProfileId::GlassBishop), "Glass Bishop");
        assert_ne!(act_one.name, act_three.name);
        assert_ne!(act_one.summary, act_three.summary);
        assert_ne!(elite.name, act_one.name);
    }

    #[test]
    fn boss_profiles_have_reworked_rotations() {
        let penta_open = enemy_intent(EnemyProfileId::PentaCore, 0);
        let penta_mid = enemy_intent(EnemyProfileId::PentaCore, 1);
        let penta_close = enemy_intent(EnemyProfileId::PentaCore, 2);
        let hex_open = enemy_intent(EnemyProfileId::HexarchCore, 0);
        let hex_mid = enemy_intent(EnemyProfileId::HexarchCore, 1);
        let hex_close = enemy_intent(EnemyProfileId::HexarchCore, 2);
        let hept_open = enemy_intent(EnemyProfileId::HeptarchCore, 0);
        let hept_mid = enemy_intent(EnemyProfileId::HeptarchCore, 1);
        let hept_close = enemy_intent(EnemyProfileId::HeptarchCore, 2);

        assert_eq!(penta_open.name, "Target Prism");
        assert_eq!(penta_open.damage, 7);
        assert_eq!(penta_open.apply_expose, 1);
        assert_eq!(penta_mid.name, "Penta Bulwark");
        assert_eq!(penta_mid.gain_block, 10);
        assert_eq!(penta_mid.prime_bleed, 2);
        assert_eq!(penta_close.name, "Split Lattice");
        assert_eq!(penta_close.damage, 4);
        assert_eq!(penta_close.hits, 3);

        assert_eq!(hex_open.name, "Hex Shell");
        assert_eq!(hex_open.gain_block, 12);
        assert_eq!(hex_open.apply_expose, 2);
        assert_eq!(hex_mid.name, "Hex Breaker");
        assert_eq!(hex_mid.damage, 15);
        assert_eq!(hex_close.name, "Hex Volley");
        assert_eq!(hex_close.damage, 5);
        assert_eq!(hex_close.hits, 3);

        assert_eq!(hept_open.name, "Singularity Shell");
        assert_eq!(hept_open.gain_block, 16);
        assert_eq!(hept_open.prime_bleed, 3);
        assert_eq!(hept_mid.name, "Array Collapse");
        assert_eq!(hept_mid.damage, 8);
        assert_eq!(hept_mid.hits, 2);
        assert_eq!(hept_mid.apply_expose, 1);
        assert_eq!(hept_close.name, "Crown Breaker");
        assert_eq!(hept_close.damage, 20);
    }
}
