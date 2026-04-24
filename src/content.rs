mod cards;

#[allow(unused_imports)]
pub(crate) use cards::{
    AxisKind, CardArchetype, CardDef, CardId, CardRequirement, CardTarget, CardTraits, RewardTier,
    ShopOffer, all_base_cards, card_def, card_requirement, card_slug, resolve_card_slug,
    reward_choices, reward_pool, shop_offers, starter_deck, upgraded_card,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EventId {
    SalvageCache,
    RelayTerminal,
    ClinicPod,
    ExchangeConsole,
    PrototypeRack,
    CoolingVault,
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
    SiegeSpider,
    RiftBastion,
    HeptarchCore,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EnemySpriteLayerTone {
    Base,
    DetailA,
    DetailB,
    DetailC,
    DetailD,
    DetailE,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct EnemySpriteLayerDef {
    pub(crate) code: u8,
    pub(crate) width: u8,
    pub(crate) height: u8,
    pub(crate) tone: EnemySpriteLayerTone,
    pub(crate) bits: &'static [u8],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct EnemySpriteDef {
    pub(crate) width: u8,
    pub(crate) height: u8,
    pub(crate) layers: &'static [EnemySpriteLayerDef],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct EnemySpriteBounds {
    pub(crate) left: u8,
    pub(crate) top: u8,
    pub(crate) width: u8,
    pub(crate) height: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct EnemyIntent {
    pub(crate) name: &'static str,
    pub(crate) summary: &'static str,
    pub(crate) damage: i32,
    pub(crate) hits: u8,
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
            description: "Start each combat with Focus +1.",
        },
        ModuleId::Nanoforge => ModuleDef {
            id,
            name: "Nanoforge",
            description: "After each victory, recover 2 HP.",
        },
        ModuleId::CapacitorBank => ModuleDef {
            id,
            name: "Capacitor Bank",
            description: "Start each combat with Momentum +1.",
        },
        ModuleId::PrismScope => ModuleDef {
            id,
            name: "Prism Scope",
            description: "Start each combat by applying Rhythm -1 to all enemies.",
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
            description: "Start each combat by applying Focus -1 to all enemies.",
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

pub(crate) fn starter_module_choices() -> Vec<ModuleId> {
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

pub(crate) fn event_def(id: EventId) -> EventDef {
    match id {
        EventId::SalvageCache => EventDef {
            id,
            title: "Salvage Cache",
            flavor: "A drift crate hums beneath a half-collapsed service gantry.",
        },
        EventId::RelayTerminal => EventDef {
            id,
            title: "Relay Terminal",
            flavor: "A damaged relay terminal still shows a few readable combat routines.",
        },
        EventId::ClinicPod => EventDef {
            id,
            title: "Clinic Pod",
            flavor: "An intact med pod still cycles a pale diagnostic glow.",
        },
        EventId::ExchangeConsole => EventDef {
            id,
            title: "Exchange Console",
            flavor: "An exchange console still lets you trade useful parts for credits.",
        },
        EventId::PrototypeRack => EventDef {
            id,
            title: "Prototype Rack",
            flavor: "A sealed rack flickers between proven gear and live-fire prototypes.",
        },
        EventId::CoolingVault => EventDef {
            id,
            title: "Cooling Vault",
            flavor: "A cooling vault is still running, though the temperature controls are unstable.",
        },
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn event_choice_title(event: EventId, choice_index: usize) -> Option<&'static str> {
    match (event, choice_index) {
        (EventId::SalvageCache, 0) => Some("Take the clean parts"),
        (EventId::SalvageCache, 1) => Some("Cut the safety seals"),
        (EventId::RelayTerminal, 0) => Some("Copy the shield routine"),
        (EventId::RelayTerminal, 1) => Some("Pull the attack routine"),
        (EventId::ClinicPod, 0) => Some("Run the recovery cycle"),
        (EventId::ClinicPod, 1) => Some("Overclock the chassis"),
        (EventId::ExchangeConsole, 0) => Some("Sell the spare plating"),
        (EventId::ExchangeConsole, 1) => Some("Sell the coolant reserve"),
        (EventId::PrototypeRack, 0) => Some("Take the stable shell"),
        (EventId::PrototypeRack, 1) => Some("Take the live prototype"),
        (EventId::CoolingVault, 0) => Some("Rest by the coolant vent"),
        (EventId::CoolingVault, 1) => Some("Endure the freezing chamber"),
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
        (EventId::RelayTerminal, 0) => Some(EventChoiceEffect::AddCard(CardId::CoverPulse)),
        (EventId::RelayTerminal, 1) => Some(EventChoiceEffect::LoseHpAddCard {
            lose_hp: 4,
            card: CardId::TacticalBurst,
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
        (EventId::ExchangeConsole, 0) => Some(EventChoiceEffect::GainCredits(22)),
        (EventId::ExchangeConsole, 1) => Some(EventChoiceEffect::LoseHpGainCredits {
            lose_hp: 5,
            gain_credits: 36,
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
        (EventId::CoolingVault, 0) => Some(EventChoiceEffect::Heal(12)),
        (EventId::CoolingVault, 1) => Some(EventChoiceEffect::LoseHpGainMaxHp {
            lose_hp: 6,
            gain_max_hp: 5,
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
    cards::localized_card_def(id, language)
}

pub(crate) fn localized_card_name(id: CardId, language: Language) -> &'static str {
    cards::localized_card_name(id, language)
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
    let english = module_def(id).description;
    match id {
        ModuleId::AegisDrive => {
            localized_text(language, english, "Comienzas cada combate con 5 de Escudo.")
        }
        ModuleId::TargetingRelay => localized_text(
            language,
            english,
            "Al comienzo de cada combate, ganas Enfoque +1.",
        ),
        ModuleId::Nanoforge => {
            localized_text(language, english, "Tras cada victoria, recupera 2 HP.")
        }
        ModuleId::CapacitorBank => {
            localized_text(language, english, "Comienzas cada combate con Impulso +1.")
        }
        ModuleId::PrismScope => localized_text(
            language,
            english,
            "Al comienzo de cada combate, aplica Ritmo -1 a todos los enemigos.",
        ),
        ModuleId::SalvageLedger => localized_text(
            language,
            english,
            "Tras cada victoria, gana 4 Créditos adicionales.",
        ),
        ModuleId::OverclockCore => localized_text(
            language,
            english,
            "Comienzas cada combate con 1 de Energía extra.",
        ),
        ModuleId::SuppressionField => localized_text(
            language,
            english,
            "Al comienzo de cada combate, aplica Enfoque -1 a todos los enemigos.",
        ),
        ModuleId::RecoveryMatrix => {
            localized_text(language, english, "Tras cada victoria, recupera 5 HP.")
        }
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
        EventId::RelayTerminal => localized_text(language, "Relay Terminal", "Terminal de Relé"),
        EventId::ClinicPod => localized_text(language, "Clinic Pod", "Cápsula Clínica"),
        EventId::ExchangeConsole => {
            localized_text(language, "Exchange Console", "Consola de Intercambio")
        }
        EventId::PrototypeRack => {
            localized_text(language, "Prototype Rack", "Bastidor de Prototipos")
        }
        EventId::CoolingVault => localized_text(language, "Cooling Vault", "Bóveda Criogénica"),
    }
}

pub(crate) fn localized_event_flavor(id: EventId, language: Language) -> &'static str {
    match id {
        EventId::SalvageCache => localized_text(
            language,
            "A drift crate hums beneath a half-collapsed service gantry.",
            "Un contenedor a la deriva zumba bajo una pasarela de servicio medio derrumbada.",
        ),
        EventId::RelayTerminal => localized_text(
            language,
            "A damaged relay terminal still shows a few readable combat routines.",
            "Un terminal de relé dañado aún muestra algunas rutinas de combate legibles.",
        ),
        EventId::ClinicPod => localized_text(
            language,
            "An intact med pod still cycles a pale diagnostic glow.",
            "Una cápsula médica intacta aún emite un tenue resplandor de diagnóstico.",
        ),
        EventId::ExchangeConsole => localized_text(
            language,
            "An exchange console still lets you trade useful parts for credits.",
            "Una consola de intercambio aún te deja cambiar piezas útiles por créditos.",
        ),
        EventId::PrototypeRack => localized_text(
            language,
            "A sealed rack flickers between proven gear and live-fire prototypes.",
            "Un bastidor sellado alterna entre equipo probado y prototipos de fuego real.",
        ),
        EventId::CoolingVault => localized_text(
            language,
            "A cooling vault is still running, though the temperature controls are unstable.",
            "Una bóveda de enfriamiento aún funciona, aunque los controles de temperatura son inestables.",
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
        (EventId::RelayTerminal, 0) => localized_text(
            language,
            "Copy the shield routine",
            "Copiar la rutina de escudo",
        ),
        (EventId::RelayTerminal, 1) => localized_text(
            language,
            "Pull the attack routine",
            "Extraer la rutina de ataque",
        ),
        (EventId::ClinicPod, 0) => localized_text(
            language,
            "Run the recovery cycle",
            "Activar el ciclo de recuperación",
        ),
        (EventId::ClinicPod, 1) => {
            localized_text(language, "Overclock the chassis", "Sobrecargar el armazón")
        }
        (EventId::ExchangeConsole, 0) => localized_text(
            language,
            "Sell the spare plating",
            "Vender las placas sobrantes",
        ),
        (EventId::ExchangeConsole, 1) => localized_text(
            language,
            "Sell the coolant reserve",
            "Vender la reserva de refrigerante",
        ),
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
        (EventId::CoolingVault, 0) => localized_text(
            language,
            "Rest by the coolant vent",
            "Descansar junto al conducto frío",
        ),
        (EventId::CoolingVault, 1) => localized_text(
            language,
            "Endure the freezing chamber",
            "Aguantar la cámara helada",
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
        EnemyProfileId::SiegeSpider => localized_text(language, "Siege Spider", "Araña de Asedio"),
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
    if matches!(language, Language::English) {
        return enemy_intent(profile, index);
    }

    let mut intent = enemy_intent(profile, index);
    let translated = match (profile, index % 3) {
        (EnemyProfileId::ScoutDrone, 0) => ("Aguja de Choque", "Inflige 5 de daño."),
        (EnemyProfileId::ScoutDrone, 1) => ("Fuego Cruzado", "Inflige 3 de daño dos veces."),
        (EnemyProfileId::ScoutDrone, _) => {
            ("Ciclo de Refuerzo", "Gana 4 de Escudo. Gana Enfoque +1.")
        }
        (EnemyProfileId::NeedlerDrone, 0) => {
            ("Toque de Aguijón", "Inflige 4 de daño. Aplica Sangrado 1.")
        }
        (EnemyProfileId::NeedlerDrone, 1) => ("Picadura Múltiple", "Inflige 2 de daño tres veces."),
        (EnemyProfileId::NeedlerDrone, _) => ("Estabilizar", "Gana 4 de Escudo."),
        (EnemyProfileId::RampartDrone, 0) => ("Placa de Choque", "Inflige 8 de daño."),
        (EnemyProfileId::RampartDrone, 1) => (
            "Mordaza de Presión",
            "Inflige 5 de daño. Aplica Enfoque -1.",
        ),
        (EnemyProfileId::RampartDrone, _) => (
            "Muro Reforzado",
            "Gana 8 de Escudo. El siguiente golpe aplica Sangrado 2.",
        ),
        (EnemyProfileId::SpineSentry, 0) => (
            "Bastidor de Púas",
            "Inflige 4 de daño dos veces. Aplica Sangrado 1.",
        ),
        (EnemyProfileId::SpineSentry, 1) => (
            "Fijación de Blanco",
            "Inflige 7 de daño. Aplica Impulso -1.",
        ),
        (EnemyProfileId::SpineSentry, _) => ("Blindaje de Púas", "Gana 9 de Escudo."),
        (EnemyProfileId::PentaCore, 0) => (
            "Prisma de Fijación",
            "Inflige 7 de daño. Aplica Impulso -1.",
        ),
        (EnemyProfileId::PentaCore, 1) => (
            "Baluarte Penta",
            "Gana 10 de Escudo. El siguiente golpe aplica Sangrado 2.",
        ),
        (EnemyProfileId::PentaCore, _) => ("Trama Fragmentada", "Inflige 4 de daño tres veces."),
        (EnemyProfileId::VoltMantis, 0) => ("Corte de Arco", "Inflige 8 de daño."),
        (EnemyProfileId::VoltMantis, 1) => ("Látigo de Arco", "Inflige 4 de daño dos veces."),
        (EnemyProfileId::VoltMantis, _) => {
            ("Caparazón de Impulso", "Gana 7 de Escudo. Gana Impulso +1.")
        }
        (EnemyProfileId::ShardWeaver, 0) => {
            ("Corte de Vidrio", "Inflige 6 de daño. Aplica Impulso -1.")
        }
        (EnemyProfileId::ShardWeaver, 1) => (
            "Andanada Reflejada",
            "Inflige 3 de daño dos veces. Gana 4 de Escudo.",
        ),
        (EnemyProfileId::ShardWeaver, _) => ("Reenfocar", "Gana 8 de Escudo. Aplica Ritmo -1."),
        (EnemyProfileId::PrismArray, 0) => {
            ("Mordida Prisma", "Inflige 7 de daño. Aplica Impulso -1.")
        }
        (EnemyProfileId::PrismArray, 1) => ("Salva Iónica", "Inflige 5 de daño dos veces."),
        (EnemyProfileId::PrismArray, _) => ("Guardia Prismática", "Gana 10 de Escudo."),
        (EnemyProfileId::GlassBishop, 0) => {
            ("Rayo Astillado", "Inflige 8 de daño. Aplica Impulso -1.")
        }
        (EnemyProfileId::GlassBishop, 1) => (
            "Halo Partido",
            "Inflige 5 de daño dos veces. Gana 4 de Escudo.",
        ),
        (EnemyProfileId::GlassBishop, _) => {
            ("Barrera Facetada", "Gana 10 de Escudo. Aplica Sangrado 1.")
        }
        (EnemyProfileId::HexarchCore, 0) => ("Coraza Hex", "Gana 12 de Escudo. Aplica Enfoque -2."),
        (EnemyProfileId::HexarchCore, 1) => ("Ruptor Hex", "Inflige 15 de daño."),
        (EnemyProfileId::HexarchCore, _) => ("Andanada Hex", "Inflige 5 de daño tres veces."),
        (EnemyProfileId::NullRaider, 0) => ("Disparo Null", "Inflige 10 de daño."),
        (EnemyProfileId::NullRaider, 1) => ("Ráfaga en Cadena", "Inflige 5 de daño dos veces."),
        (EnemyProfileId::NullRaider, _) => ("Guardia Null", "Gana 9 de Escudo."),
        (EnemyProfileId::RiftStalker, 0) => (
            "Garra de la Grieta",
            "Inflige 9 de daño. Aplica Sangrado 1.",
        ),
        (EnemyProfileId::RiftStalker, 1) => ("Salva Desgarradora", "Inflige 5 de daño dos veces."),
        (EnemyProfileId::RiftStalker, _) => {
            ("Ancla de Fijación", "Gana 10 de Escudo. Aplica Enfoque -1.")
        }
        (EnemyProfileId::SiegeSpider, 0) => ("Martillo Baluarte", "Inflige 11 de daño."),
        (EnemyProfileId::SiegeSpider, 1) => {
            ("Andanada de Fijación", "Inflige 6 de daño dos veces.")
        }
        (EnemyProfileId::SiegeSpider, _) => {
            ("Sello Baluarte", "Gana 12 de Escudo. Aplica Enfoque -1.")
        }
        (EnemyProfileId::RiftBastion, 0) => ("Martillo Gravitatorio", "Inflige 12 de daño."),
        (EnemyProfileId::RiftBastion, 1) => (
            "Malla de Colapso",
            "Inflige 6 de daño dos veces. Aplica Impulso -1.",
        ),
        (EnemyProfileId::RiftBastion, _) => (
            "Campo de Anclaje",
            "Gana 12 de Escudo. El siguiente golpe aplica Sangrado 3.",
        ),
        (EnemyProfileId::HeptarchCore, 0) => (
            "Coraza de Singularidad",
            "Gana 16 de Escudo. El siguiente golpe aplica Sangrado 3.",
        ),
        (EnemyProfileId::HeptarchCore, 1) => (
            "Colapso de Matriz",
            "Inflige 8 de daño dos veces. Aplica Impulso -1.",
        ),
        (EnemyProfileId::HeptarchCore, _) => ("Quebracoronas", "Inflige 20 de daño."),
    };
    intent.name = translated.0;
    intent.summary = translated.1;
    intent
}

pub(crate) fn event_pool_for_level(level: usize) -> [EventId; 2] {
    match level.clamp(1, 3) {
        1 => [EventId::SalvageCache, EventId::RelayTerminal],
        2 => [EventId::ClinicPod, EventId::ExchangeConsole],
        _ => [EventId::PrototypeRack, EventId::CoolingVault],
    }
}

pub(crate) fn ordered_events_for_level(seed: u64, level: usize) -> [EventId; 2] {
    let mut events = event_pool_for_level(level);
    events.sort_by_key(|event| event_roll_key(seed, *event));
    events
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
        EnemyProfileId::SiegeSpider => "Siege Spider",
        EnemyProfileId::RiftBastion => "Rift Bastion",
        EnemyProfileId::HeptarchCore => "Heptarch Core",
    }
}

pub(crate) fn enemy_profile_level(profile: EnemyProfileId) -> usize {
    match profile {
        EnemyProfileId::ScoutDrone
        | EnemyProfileId::NeedlerDrone
        | EnemyProfileId::RampartDrone
        | EnemyProfileId::SpineSentry
        | EnemyProfileId::PentaCore => 1,
        EnemyProfileId::VoltMantis
        | EnemyProfileId::ShardWeaver
        | EnemyProfileId::PrismArray
        | EnemyProfileId::GlassBishop
        | EnemyProfileId::HexarchCore => 2,
        EnemyProfileId::NullRaider
        | EnemyProfileId::RiftStalker
        | EnemyProfileId::SiegeSpider
        | EnemyProfileId::RiftBastion
        | EnemyProfileId::HeptarchCore => 3,
    }
}

const ENEMY_SPRITE_WIDTH: u8 = 16;
const ENEMY_SPRITE_HEIGHT: u8 = 16;

const fn pack_enemy_sprite_rows(rows: [u16; 16]) -> [u8; 32] {
    let mut bytes = [0; 32];
    let mut index = 0;
    while index < rows.len() {
        bytes[index * 2] = (rows[index] >> 8) as u8;
        bytes[index * 2 + 1] = rows[index] as u8;
        index += 1;
    }
    bytes
}

// Distinct layered 16x16 enemy icons with abstract mechanical silhouettes.
const SCOUTDRONE_BASE_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000001110000000,
    0b0000011111000000,
    0b0000110001100000,
    0b0000110001100000,
    0b0011100000110000,
    0b0011111111111100,
    0b0111111111101110,
    0b0111011111001110,
    0b0111011111001110,
    0b0011111111001100,
    0b0000111111100000,
    0b0000000001100000,
    0b0000000000110000,
    0b0000000000110000,
    0b0000000001100000,
    0b0000000111000000,
]);
const SCOUTDRONE_DETAIL_A_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b0000000000000000,
    0b0000001110000000,
    0b0000001110000000,
    0b0000011111000000,
    0b0000000000000000,
    0b0000000000010000,
    0b0000100000110000,
    0b0000100000110000,
    0b0000000000110000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
]);
const SCOUTDRONE_DETAIL_B_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000110000000000,
    0b0011100000000000,
    0b0011100000000000,
    0b0000000000000000,
    0b0000000000000000,
]);
const NEEDLERDRONE_BASE_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b0000000000000000,
    0b0000000001100000,
    0b0000000111100000,
    0b0000001110100000,
    0b0000001111100000,
    0b0001111111110000,
    0b0000111111100000,
    0b0000111111100000,
    0b0000011111000000,
    0b0000011111000000,
    0b0000110001100000,
    0b0000111111100000,
    0b0000011111000000,
    0b0000010001100000,
    0b0000100000110000,
]);
const NEEDLERDRONE_DETAIL_A_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000001000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000001110000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
]);
const NEEDLERDRONE_DETAIL_B_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000001000000,
    0b0000000001100000,
    0b0000000110000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
]);
const RAMPARTDRONE_BASE_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b0000111111100000,
    0b0001100000110000,
    0b0011000000011000,
    0b0011000000011000,
    0b0011000000011000,
    0b0011000000011000,
    0b0011000000011000,
    0b0011000000011000,
    0b0011000000011000,
    0b0011000000011000,
    0b0001100000110000,
    0b0000111111100000,
    0b0000000000000000,
    0b0000011111000000,
    0b0000110001100000,
]);
const RAMPARTDRONE_DETAIL_A_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b0000000000000000,
    0b0000011111000000,
    0b0000100000100000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000001110000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000100000100000,
    0b0000011111000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
]);
const RAMPARTDRONE_DETAIL_B_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0001111111110000,
    0b0011000000011000,
    0b0110000000001100,
    0b0100011111000100,
    0b0100111111100100,
    0b0100111111100100,
    0b0100110001100100,
    0b0100111111100100,
    0b0100111111100100,
    0b0100111111100100,
    0b0100011111000100,
    0b0110000000001100,
    0b0011000000011000,
    0b0001111111110000,
    0b0000000000000000,
    0b0000000000000000,
]);
const SPINESENTRY_BASE_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000100000000,
    0b0000001110000000,
    0b0000011011000000,
    0b0000110000100000,
    0b0001110000110000,
    0b0011101100111000,
    0b0111001111001100,
    0b0011101100111000,
    0b0001110000110000,
    0b0000110000100000,
    0b0000011011000000,
    0b0010001110001000,
    0b0110011111001100,
    0b0010011011001000,
    0b0000011011000000,
    0b0000110001100000,
]);
const SPINESENTRY_DETAIL_A_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b0000000000000000,
    0b0000000100000000,
    0b0000001111000000,
    0b0000001111000000,
    0b0000010011000000,
    0b0000110000110000,
    0b0000010011000000,
    0b0000001111000000,
    0b0000001111000000,
    0b0000000100000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
]);
const PENTACORE_BASE_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000110000000,
    0b0000001001000000,
    0b0000010000100000,
    0b0001110000111000,
    0b0001110000111000,
    0b0011100000011100,
    0b0110000000000110,
    0b0110000000000110,
    0b0110000000000110,
    0b0110000000000110,
    0b0110000000000110,
    0b0111100000011110,
    0b0111100000011110,
    0b0001111111111000,
    0b0000011001100000,
    0b0000000000000000,
]);
const PENTACORE_DETAIL_A_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000110000000,
    0b0000000110000000,
    0b0000001111000000,
    0b0000011001100000,
    0b0000011001100000,
    0b0000011111100000,
    0b0000001111000000,
    0b0000000110000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
]);
const PENTACORE_DETAIL_B_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b0000000000000000,
    0b0000001111000000,
    0b0000010000100000,
    0b0000010000100000,
    0b0001110000111000,
    0b0001100000011000,
    0b0001100000011000,
    0b0001100000011000,
    0b0001100000011000,
    0b0000011001100000,
    0b0000001111000000,
    0b0000001111000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
]);
const PENTACORE_DETAIL_C_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000110000000,
    0b0000000110000000,
    0b0000010000100000,
    0b0000000000000000,
    0b0000000000000000,
    0b0100000000000010,
    0b0000000000000000,
    0b0000000110000000,
    0b1000000000000001,
    0b0000000000000000,
    0b0010000000000100,
    0b0000000000000000,
    0b0000000000000000,
    0b0001100000011000,
    0b0000010000100000,
    0b0000000110000000,
]);
const VOLTMANTIS_BASE_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b0000000000000000,
    0b0001100000011000,
    0b0001110011100000,
    0b0001110011100000,
    0b0001110000111000,
    0b0000010000100000,
    0b0000011001100000,
    0b0000001111000000,
    0b0000000110000000,
    0b0000000110000000,
    0b0000000110000000,
    0b0000000110000000,
    0b0000001001000000,
    0b0000010000100000,
    0b0001100000011000,
]);
const VOLTMANTIS_DETAIL_A_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000001100000000,
    0b0000001100000000,
    0b0000001111000000,
    0b0000001111000000,
    0b0000000110000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
]);
const VOLTMANTIS_DETAIL_B_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0001100000011000,
    0b0011110000111000,
    0b0110011001100000,
    0b0010000000011000,
    0b0010000000011000,
    0b0000000000000000,
    0b0000000000000000,
    0b0011100000000000,
    0b0110000000000110,
    0b1100000000000011,
    0b0100000000000001,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
]);
const SHARDWEAVER_BASE_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000100,
    0b0000000000111000,
    0b0000000001100000,
    0b0100000000000010,
    0b0100000000000010,
    0b0011000000011100,
    0b0001000000011000,
    0b0110000000000000,
    0b0000000000000110,
    0b0001000000011000,
    0b0011000000011100,
    0b0100000000000010,
    0b0100000000000010,
    0b0000111000000000,
    0b0001110000000000,
    0b0010000000000000,
]);
const SHARDWEAVER_DETAIL_A_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0010000000000000,
    0b0001110000000000,
    0b0000111000000000,
    0b0000001111000000,
    0b0000001111000000,
    0b0000111001100000,
    0b0000111001100000,
    0b0001110000111000,
    0b0001110000111000,
    0b0000111001100000,
    0b0000111001100000,
    0b0000001111000000,
    0b0000001111000000,
    0b0000000001100000,
    0b0000000000111000,
    0b0000000000000100,
]);
const SHARDWEAVER_DETAIL_B_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000110000000,
    0b0000000110000000,
    0b0000001111000000,
    0b0000001111000000,
    0b0000000110000000,
    0b0000000110000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
]);
const PRISMARRAY_BASE_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b0000000000000000,
    0b0000000110000000,
    0b0000000110000000,
    0b0000001111000000,
    0b0000000110000000,
    0b0000000000000000,
    0b0000000110000000,
    0b0000000110000000,
    0b0000001111000000,
    0b0000111011110000,
    0b0000001111000000,
    0b0000000110000000,
    0b0000000110000000,
    0b0000000000000000,
    0b0000000000000000,
]);
const PRISMARRAY_DETAIL_A_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000110000000,
    0b0000001111000000,
    0b0000111001110000,
    0b0000111001110000,
    0b0001110000111000,
    0b0000111001110000,
    0b0000001010000000,
    0b0000111001110000,
    0b0000111001110000,
    0b0001110000111000,
    0b0011000000001100,
    0b0001110000111000,
    0b0000111001110000,
    0b0000111001110000,
    0b0000001111000000,
    0b0000000110000000,
]);
const PRISMARRAY_DETAIL_B_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000100000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000100000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
]);
const GLASSBISHOP_BASE_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000001000000000,
    0b0000011110000000,
    0b0000111111000000,
    0b0000111111000000,
    0b0000011111000000,
    0b0000011111000000,
    0b0000011111000000,
    0b0000011111000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
]);
const GLASSBISHOP_DETAIL_A_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000001000000000,
    0b0000011110000000,
    0b0000011110000000,
    0b0000011110000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
]);
const GLASSBISHOP_DETAIL_B_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b0000000000000000,
    0b0000100001000000,
    0b0000100001000000,
    0b0000011110000000,
    0b0000110111000000,
    0b0001100001100000,
    0b0001000000000000,
    0b0001000000000000,
    0b0000100000100000,
    0b0000000000100000,
    0b0000100000100000,
    0b0001100000110000,
    0b0001111111110000,
    0b0000110001100000,
    0b0001100000110000,
]);
const HEXARCHCORE_BASE_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000001111000000,
    0b0000111001100000,
    0b0001110000111000,
    0b0011000000011100,
    0b0011000000011100,
    0b0110000000000110,
    0b0110000000000110,
    0b0110000000000110,
    0b0110000000000110,
    0b0110000000000110,
    0b0011000000011100,
    0b0001110000111000,
    0b0001110000111000,
    0b0000111001100000,
    0b0000001111000000,
    0b0000000000000000,
]);
const HEXARCHCORE_DETAIL_A_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000001111000000,
    0b0000001111000000,
    0b0000111001100000,
    0b0001110000111000,
    0b0001110000111000,
    0b0001110000111000,
    0b0001110000111000,
    0b0000111001100000,
    0b0000001111000000,
    0b0000001111000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
]);
const HEXARCHCORE_DETAIL_B_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b0000001111000000,
    0b0000111001100000,
    0b0001110000111000,
    0b0001110000111000,
    0b0011000000011100,
    0b0010000000000100,
    0b0010000000000100,
    0b0011000000011100,
    0b0011000000011100,
    0b0001110000111000,
    0b0000111001100000,
    0b0000111001100000,
    0b0000001111000000,
    0b0000000000000000,
    0b0000000000000000,
]);
const HEXARCHCORE_DETAIL_C_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000110000000,
    0b0000001111000000,
    0b0000111001100000,
    0b0000001111000000,
    0b0000000110000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000100000000,
]);
const NULLRAIDER_BASE_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000011111000000,
    0b0000110000010000,
    0b0011100000001100,
    0b0011100000001100,
    0b0111101110001110,
    0b0111001000000010,
    0b0111010000010010,
    0b0111100000001110,
    0b0111100000001110,
    0b0111010000011110,
    0b0111001000001110,
    0b0011100000011100,
    0b0000111111110000,
    0b0000111111110000,
    0b0000011110110000,
    0b0000110000011100,
]);
const NULLRAIDER_DETAIL_A_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000111000000,
    0b0000001111100000,
    0b0000001111100000,
    0b0000000111000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
]);
const NULLRAIDER_DETAIL_B_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b0000001111100000,
    0b0000011111110000,
    0b0000011111110000,
    0b0000010000010000,
    0b0000110000011100,
    0b0000100000001100,
    0b0000000000000000,
    0b0000000000000000,
    0b0000100000000000,
    0b0000110000010000,
    0b0000010000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
]);
const NULLRAIDER_DETAIL_C_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000001100000,
    0b0000000111100000,
    0b0000001000100000,
    0b0000010000010000,
    0b0000010000010000,
    0b0000001000100000,
    0b0000000111100000,
    0b0000001111100000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
]);
const RIFTSTALKER_BASE_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000011100,
    0b0000000000111110,
    0b0000000001111110,
    0b0000000001111100,
    0b0000000001111100,
    0b0000000001110000,
    0b0000000001110000,
    0b0000000001100000,
    0b0000000001100000,
    0b0000000001100000,
    0b0000000000000000,
    0b0000000000000000,
]);
const RIFTSTALKER_DETAIL_A_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b0000000000000000,
    0b0000000111000000,
    0b0000000111000000,
    0b0000001111100000,
    0b0000011110000000,
    0b0000110000000000,
    0b0011100000000000,
    0b0011100000000000,
    0b0011100000000000,
    0b0000110000000000,
    0b0000011110000000,
    0b0000000110000000,
    0b0000000110000000,
    0b0000000000000000,
    0b0000000000000000,
]);
const RIFTSTALKER_DETAIL_B_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000111000000,
    0b0000011111110000,
    0b0000111000111100,
    0b0000111000111100,
    0b0011110000000000,
    0b0111100000000000,
    0b0111000000000000,
    0b0100000000000000,
    0b0100000000000000,
    0b0100000000000000,
    0b0111000000000000,
    0b0011100000000000,
    0b0000111000000000,
    0b0000111000000000,
    0b0000011111000000,
    0b0000000111000000,
]);
const RIFTSTALKER_DETAIL_C_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000001000000,
    0b0000001110000000,
    0b0000011110000000,
    0b0000011110000000,
    0b0000011110000000,
    0b0000001110000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
]);
const SIEGESPIDER_BASE_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b1000000000000001,
    0b0100100000010010,
    0b0010010000100100,
    0b0001010000101000,
    0b0000100110010000,
    0b0000011111100000,
    0b0000010110100000,
    0b0000011111100000,
    0b0000100110010000,
    0b0001010000101000,
    0b0010001001000100,
    0b0100010000100010,
    0b1000100000010001,
    0b0001000000001000,
    0b0000000000000000,
]);
const SIEGESPIDER_DETAIL_A_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000110000000,
    0b0000001111000000,
    0b0000001001000000,
    0b0000001111000000,
    0b0000001001000000,
    0b0000000110000000,
    0b0000000010000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
]);
const SIEGESPIDER_DETAIL_B_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b1000000000000001,
    0b0100100000010010,
    0b0010010000100100,
    0b0001010000101000,
    0b0000101111010000,
    0b0000011001100000,
    0b0000001111000000,
    0b0000011001100000,
    0b0000101111010000,
    0b0001000000001000,
    0b0010000000000100,
    0b0100010000100010,
    0b0000100000010000,
    0b0000000000000000,
    0b0000000000000000,
]);
const RIFTBASTION_BASE_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000011111100000,
    0b0001110000111000,
    0b0011100000011100,
    0b0110000000000110,
    0b0110000000000110,
    0b0110000000000110,
    0b0110000000000110,
    0b0110000000000110,
    0b0110000000000110,
    0b0110000000000110,
    0b0110000000000110,
    0b0011100000011100,
    0b0011100000011100,
    0b0001111111111000,
    0b0000011001100000,
    0b0000000000000000,
]);
const RIFTBASTION_DETAIL_A_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000011000000,
    0b0000000110000000,
    0b0000001100000000,
    0b0000011000000000,
    0b0000001100000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
]);
const RIFTBASTION_DETAIL_B_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000011111100000,
    0b0011111111111100,
    0b0111100000011110,
    0b0110001111000110,
    0b0110001111000110,
    0b1100001111000011,
    0b1100011001100011,
    0b1100011001100011,
    0b1100011001100011,
    0b1100011001100011,
    0b1100011001100011,
    0b1100011111100011,
    0b1100011111100011,
    0b0110001111000110,
    0b0111100000011110,
    0b0011111111111100,
]);
const RIFTBASTION_DETAIL_C_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000001111000000,
    0b0000001111000000,
    0b0000001111000000,
    0b0000001100000000,
    0b0000001001000000,
    0b0000000011000000,
    0b0000001111000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
]);
const HEPTARCHCORE_BASE_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b0000000110000000,
    0b0000111111111000,
    0b0001100000000100,
    0b0011000001000010,
    0b0010000000000010,
    0b0110000000000011,
    0b0110000000000001,
    0b0110000000000001,
    0b0110000000000011,
    0b0010000000000010,
    0b0011000001000010,
    0b0001100000000100,
    0b0000111111111000,
    0b0000000110000000,
    0b0000000000000000,
]);
const HEPTARCHCORE_DETAIL_A_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000001110110000,
    0b0000010000001000,
    0b0000100000001000,
    0b0000100000001000,
    0b0000100000001000,
    0b0000100000001000,
    0b0000010000001000,
    0b0000001110110000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
]);
const HEPTARCHCORE_DETAIL_B_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000011111111000,
    0b0000110000001100,
    0b0001100000000100,
    0b0001000000000100,
    0b0001000000000110,
    0b0001000000000110,
    0b0001000000000100,
    0b0001100000000100,
    0b0000110000001100,
    0b0000011111111000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
]);
const HEPTARCHCORE_DETAIL_C_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000001111110000,
    0b0000011111110000,
    0b0000011000110000,
    0b0000011000110000,
    0b0000011111110000,
    0b0000001111110000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
    0b0000000000000000,
]);
const HEPTARCHCORE_DETAIL_D_SPRITE_BITS: [u8; 32] = pack_enemy_sprite_rows([
    0b0000000110000000,
    0b0000111001110000,
    0b0001000000000100,
    0b0010000000000010,
    0b0000000000000000,
    0b0100000000000001,
    0b0000000000000000,
    0b1000000111000000,
    0b1000000111000000,
    0b0000000000000000,
    0b0100000000000001,
    0b0000000000000000,
    0b0010000000000010,
    0b0001000000000100,
    0b0000111001110000,
    0b0000000110000000,
]);

const SCOUTDRONE_BASE_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 1,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::Base,
    bits: &SCOUTDRONE_BASE_SPRITE_BITS,
};
const SCOUTDRONE_DETAIL_A_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 2,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailB,
    bits: &SCOUTDRONE_DETAIL_A_SPRITE_BITS,
};
const SCOUTDRONE_DETAIL_B_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 3,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailE,
    bits: &SCOUTDRONE_DETAIL_B_SPRITE_BITS,
};
const NEEDLERDRONE_BASE_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 4,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::Base,
    bits: &NEEDLERDRONE_BASE_SPRITE_BITS,
};
const NEEDLERDRONE_DETAIL_A_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 5,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailA,
    bits: &NEEDLERDRONE_DETAIL_A_SPRITE_BITS,
};
const NEEDLERDRONE_DETAIL_B_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 6,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailC,
    bits: &NEEDLERDRONE_DETAIL_B_SPRITE_BITS,
};
const RAMPARTDRONE_BASE_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 7,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::Base,
    bits: &RAMPARTDRONE_BASE_SPRITE_BITS,
};
const RAMPARTDRONE_DETAIL_A_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 8,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailE,
    bits: &RAMPARTDRONE_DETAIL_A_SPRITE_BITS,
};
const RAMPARTDRONE_DETAIL_B_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 9,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailB,
    bits: &RAMPARTDRONE_DETAIL_B_SPRITE_BITS,
};
const SPINESENTRY_BASE_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 10,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::Base,
    bits: &SPINESENTRY_BASE_SPRITE_BITS,
};
const SPINESENTRY_DETAIL_A_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 11,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailC,
    bits: &SPINESENTRY_DETAIL_A_SPRITE_BITS,
};
const PENTACORE_BASE_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 12,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::Base,
    bits: &PENTACORE_BASE_SPRITE_BITS,
};
const PENTACORE_DETAIL_A_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 13,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailC,
    bits: &PENTACORE_DETAIL_A_SPRITE_BITS,
};
const PENTACORE_DETAIL_B_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 14,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailE,
    bits: &PENTACORE_DETAIL_B_SPRITE_BITS,
};
const PENTACORE_DETAIL_C_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 15,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailA,
    bits: &PENTACORE_DETAIL_C_SPRITE_BITS,
};
const VOLTMANTIS_BASE_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 16,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::Base,
    bits: &VOLTMANTIS_BASE_SPRITE_BITS,
};
const VOLTMANTIS_DETAIL_A_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 17,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailC,
    bits: &VOLTMANTIS_DETAIL_A_SPRITE_BITS,
};
const VOLTMANTIS_DETAIL_B_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 18,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailE,
    bits: &VOLTMANTIS_DETAIL_B_SPRITE_BITS,
};
const SHARDWEAVER_BASE_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 19,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::Base,
    bits: &SHARDWEAVER_BASE_SPRITE_BITS,
};
const SHARDWEAVER_DETAIL_A_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 20,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailA,
    bits: &SHARDWEAVER_DETAIL_A_SPRITE_BITS,
};
const SHARDWEAVER_DETAIL_B_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 21,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailC,
    bits: &SHARDWEAVER_DETAIL_B_SPRITE_BITS,
};
const PRISMARRAY_BASE_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 22,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::Base,
    bits: &PRISMARRAY_BASE_SPRITE_BITS,
};
const PRISMARRAY_DETAIL_A_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 23,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailC,
    bits: &PRISMARRAY_DETAIL_A_SPRITE_BITS,
};
const PRISMARRAY_DETAIL_B_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 24,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailE,
    bits: &PRISMARRAY_DETAIL_B_SPRITE_BITS,
};
const GLASSBISHOP_BASE_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 25,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::Base,
    bits: &GLASSBISHOP_BASE_SPRITE_BITS,
};
const GLASSBISHOP_DETAIL_A_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 26,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailE,
    bits: &GLASSBISHOP_DETAIL_A_SPRITE_BITS,
};
const GLASSBISHOP_DETAIL_B_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 27,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailC,
    bits: &GLASSBISHOP_DETAIL_B_SPRITE_BITS,
};
const HEXARCHCORE_BASE_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 28,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::Base,
    bits: &HEXARCHCORE_BASE_SPRITE_BITS,
};
const HEXARCHCORE_DETAIL_A_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 29,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailA,
    bits: &HEXARCHCORE_DETAIL_A_SPRITE_BITS,
};
const HEXARCHCORE_DETAIL_B_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 30,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailE,
    bits: &HEXARCHCORE_DETAIL_B_SPRITE_BITS,
};
const HEXARCHCORE_DETAIL_C_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 31,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailC,
    bits: &HEXARCHCORE_DETAIL_C_SPRITE_BITS,
};
const NULLRAIDER_BASE_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 32,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::Base,
    bits: &NULLRAIDER_BASE_SPRITE_BITS,
};
const NULLRAIDER_DETAIL_A_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 33,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailE,
    bits: &NULLRAIDER_DETAIL_A_SPRITE_BITS,
};
const NULLRAIDER_DETAIL_B_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 34,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailC,
    bits: &NULLRAIDER_DETAIL_B_SPRITE_BITS,
};
const NULLRAIDER_DETAIL_C_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 35,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailD,
    bits: &NULLRAIDER_DETAIL_C_SPRITE_BITS,
};
const RIFTSTALKER_BASE_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 36,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::Base,
    bits: &RIFTSTALKER_BASE_SPRITE_BITS,
};
const RIFTSTALKER_DETAIL_A_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 37,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailE,
    bits: &RIFTSTALKER_DETAIL_A_SPRITE_BITS,
};
const RIFTSTALKER_DETAIL_B_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 38,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailC,
    bits: &RIFTSTALKER_DETAIL_B_SPRITE_BITS,
};
const RIFTSTALKER_DETAIL_C_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 39,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailD,
    bits: &RIFTSTALKER_DETAIL_C_SPRITE_BITS,
};
const SIEGESPIDER_BASE_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 40,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::Base,
    bits: &SIEGESPIDER_BASE_SPRITE_BITS,
};
const SIEGESPIDER_DETAIL_A_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 41,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailC,
    bits: &SIEGESPIDER_DETAIL_A_SPRITE_BITS,
};
const SIEGESPIDER_DETAIL_B_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 42,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailE,
    bits: &SIEGESPIDER_DETAIL_B_SPRITE_BITS,
};
const RIFTBASTION_BASE_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 43,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::Base,
    bits: &RIFTBASTION_BASE_SPRITE_BITS,
};
const RIFTBASTION_DETAIL_A_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 44,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailE,
    bits: &RIFTBASTION_DETAIL_A_SPRITE_BITS,
};
const RIFTBASTION_DETAIL_B_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 45,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailD,
    bits: &RIFTBASTION_DETAIL_B_SPRITE_BITS,
};
const RIFTBASTION_DETAIL_C_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 46,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailC,
    bits: &RIFTBASTION_DETAIL_C_SPRITE_BITS,
};
const HEPTARCHCORE_BASE_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 47,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::Base,
    bits: &HEPTARCHCORE_BASE_SPRITE_BITS,
};
const HEPTARCHCORE_DETAIL_A_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 48,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailA,
    bits: &HEPTARCHCORE_DETAIL_A_SPRITE_BITS,
};
const HEPTARCHCORE_DETAIL_B_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 49,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailE,
    bits: &HEPTARCHCORE_DETAIL_B_SPRITE_BITS,
};
const HEPTARCHCORE_DETAIL_C_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 50,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailC,
    bits: &HEPTARCHCORE_DETAIL_C_SPRITE_BITS,
};
const HEPTARCHCORE_DETAIL_D_LAYER: EnemySpriteLayerDef = EnemySpriteLayerDef {
    code: 51,
    width: ENEMY_SPRITE_WIDTH,
    height: ENEMY_SPRITE_HEIGHT,
    tone: EnemySpriteLayerTone::DetailD,
    bits: &HEPTARCHCORE_DETAIL_D_SPRITE_BITS,
};

const SCOUTDRONE_SPRITE_LAYERS: &[EnemySpriteLayerDef] = &[
    SCOUTDRONE_BASE_LAYER,
    SCOUTDRONE_DETAIL_A_LAYER,
    SCOUTDRONE_DETAIL_B_LAYER,
];
const NEEDLERDRONE_SPRITE_LAYERS: &[EnemySpriteLayerDef] = &[
    NEEDLERDRONE_BASE_LAYER,
    NEEDLERDRONE_DETAIL_A_LAYER,
    NEEDLERDRONE_DETAIL_B_LAYER,
];
const RAMPARTDRONE_SPRITE_LAYERS: &[EnemySpriteLayerDef] = &[
    RAMPARTDRONE_BASE_LAYER,
    RAMPARTDRONE_DETAIL_A_LAYER,
    RAMPARTDRONE_DETAIL_B_LAYER,
];
const SPINESENTRY_SPRITE_LAYERS: &[EnemySpriteLayerDef] =
    &[SPINESENTRY_BASE_LAYER, SPINESENTRY_DETAIL_A_LAYER];
const PENTACORE_SPRITE_LAYERS: &[EnemySpriteLayerDef] = &[
    PENTACORE_BASE_LAYER,
    PENTACORE_DETAIL_A_LAYER,
    PENTACORE_DETAIL_B_LAYER,
    PENTACORE_DETAIL_C_LAYER,
];
const VOLTMANTIS_SPRITE_LAYERS: &[EnemySpriteLayerDef] = &[
    VOLTMANTIS_BASE_LAYER,
    VOLTMANTIS_DETAIL_A_LAYER,
    VOLTMANTIS_DETAIL_B_LAYER,
];
const SHARDWEAVER_SPRITE_LAYERS: &[EnemySpriteLayerDef] = &[
    SHARDWEAVER_BASE_LAYER,
    SHARDWEAVER_DETAIL_A_LAYER,
    SHARDWEAVER_DETAIL_B_LAYER,
];
const PRISMARRAY_SPRITE_LAYERS: &[EnemySpriteLayerDef] = &[
    PRISMARRAY_BASE_LAYER,
    PRISMARRAY_DETAIL_A_LAYER,
    PRISMARRAY_DETAIL_B_LAYER,
];
const GLASSBISHOP_SPRITE_LAYERS: &[EnemySpriteLayerDef] = &[
    GLASSBISHOP_BASE_LAYER,
    GLASSBISHOP_DETAIL_A_LAYER,
    GLASSBISHOP_DETAIL_B_LAYER,
];
const HEXARCHCORE_SPRITE_LAYERS: &[EnemySpriteLayerDef] = &[
    HEXARCHCORE_BASE_LAYER,
    HEXARCHCORE_DETAIL_A_LAYER,
    HEXARCHCORE_DETAIL_B_LAYER,
    HEXARCHCORE_DETAIL_C_LAYER,
];
const NULLRAIDER_SPRITE_LAYERS: &[EnemySpriteLayerDef] = &[
    NULLRAIDER_BASE_LAYER,
    NULLRAIDER_DETAIL_A_LAYER,
    NULLRAIDER_DETAIL_B_LAYER,
    NULLRAIDER_DETAIL_C_LAYER,
];
const RIFTSTALKER_SPRITE_LAYERS: &[EnemySpriteLayerDef] = &[
    RIFTSTALKER_BASE_LAYER,
    RIFTSTALKER_DETAIL_A_LAYER,
    RIFTSTALKER_DETAIL_B_LAYER,
    RIFTSTALKER_DETAIL_C_LAYER,
];
const SIEGESPIDER_SPRITE_LAYERS: &[EnemySpriteLayerDef] = &[
    SIEGESPIDER_BASE_LAYER,
    SIEGESPIDER_DETAIL_A_LAYER,
    SIEGESPIDER_DETAIL_B_LAYER,
];
const RIFTBASTION_SPRITE_LAYERS: &[EnemySpriteLayerDef] = &[
    RIFTBASTION_BASE_LAYER,
    RIFTBASTION_DETAIL_A_LAYER,
    RIFTBASTION_DETAIL_B_LAYER,
    RIFTBASTION_DETAIL_C_LAYER,
];
const HEPTARCHCORE_SPRITE_LAYERS: &[EnemySpriteLayerDef] = &[
    HEPTARCHCORE_BASE_LAYER,
    HEPTARCHCORE_DETAIL_A_LAYER,
    HEPTARCHCORE_DETAIL_B_LAYER,
    HEPTARCHCORE_DETAIL_C_LAYER,
    HEPTARCHCORE_DETAIL_D_LAYER,
];

pub(crate) fn enemy_sprite_def(profile: EnemyProfileId) -> EnemySpriteDef {
    let layers = match profile {
        EnemyProfileId::ScoutDrone => SCOUTDRONE_SPRITE_LAYERS,
        EnemyProfileId::NeedlerDrone => NEEDLERDRONE_SPRITE_LAYERS,
        EnemyProfileId::RampartDrone => RAMPARTDRONE_SPRITE_LAYERS,
        EnemyProfileId::SpineSentry => SPINESENTRY_SPRITE_LAYERS,
        EnemyProfileId::PentaCore => PENTACORE_SPRITE_LAYERS,
        EnemyProfileId::VoltMantis => VOLTMANTIS_SPRITE_LAYERS,
        EnemyProfileId::ShardWeaver => SHARDWEAVER_SPRITE_LAYERS,
        EnemyProfileId::PrismArray => PRISMARRAY_SPRITE_LAYERS,
        EnemyProfileId::GlassBishop => GLASSBISHOP_SPRITE_LAYERS,
        EnemyProfileId::HexarchCore => HEXARCHCORE_SPRITE_LAYERS,
        EnemyProfileId::NullRaider => NULLRAIDER_SPRITE_LAYERS,
        EnemyProfileId::RiftStalker => RIFTSTALKER_SPRITE_LAYERS,
        EnemyProfileId::SiegeSpider => SIEGESPIDER_SPRITE_LAYERS,
        EnemyProfileId::RiftBastion => RIFTBASTION_SPRITE_LAYERS,
        EnemyProfileId::HeptarchCore => HEPTARCHCORE_SPRITE_LAYERS,
    };

    EnemySpriteDef {
        width: ENEMY_SPRITE_WIDTH,
        height: ENEMY_SPRITE_HEIGHT,
        layers,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn enemy_sprite_union_bounds(sprite: EnemySpriteDef) -> EnemySpriteBounds {
    let mut min_x = sprite.width as usize;
    let mut max_x = 0usize;
    let mut min_y = sprite.height as usize;
    let mut max_y = 0usize;
    let mut any_active = false;

    for y in 0..sprite.height as usize {
        for x in 0..sprite.width as usize {
            if !sprite
                .layers
                .iter()
                .any(|layer| enemy_sprite_layer_has_pixel(*layer, x, y))
            {
                continue;
            }
            any_active = true;
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }
    }

    if !any_active {
        return EnemySpriteBounds {
            left: 0,
            top: 0,
            width: sprite.width.max(1),
            height: sprite.height.max(1),
        };
    }

    EnemySpriteBounds {
        left: min_x as u8,
        top: min_y as u8,
        width: (max_x - min_x + 1) as u8,
        height: (max_y - min_y + 1) as u8,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn enemy_sprite_bounds(profile: EnemyProfileId) -> EnemySpriteBounds {
    enemy_sprite_union_bounds(enemy_sprite_def(profile))
}

#[cfg_attr(not(test), allow(dead_code))]
fn enemy_profile_for_sprite_code(code: u8) -> Option<EnemyProfileId> {
    match code {
        1..=3 => Some(EnemyProfileId::ScoutDrone),
        4..=6 => Some(EnemyProfileId::NeedlerDrone),
        7..=9 => Some(EnemyProfileId::RampartDrone),
        10..=11 => Some(EnemyProfileId::SpineSentry),
        12..=15 => Some(EnemyProfileId::PentaCore),
        16..=18 => Some(EnemyProfileId::VoltMantis),
        19..=21 => Some(EnemyProfileId::ShardWeaver),
        22..=24 => Some(EnemyProfileId::PrismArray),
        25..=27 => Some(EnemyProfileId::GlassBishop),
        28..=31 => Some(EnemyProfileId::HexarchCore),
        32..=35 => Some(EnemyProfileId::NullRaider),
        36..=39 => Some(EnemyProfileId::RiftStalker),
        40..=42 => Some(EnemyProfileId::SiegeSpider),
        43..=46 => Some(EnemyProfileId::RiftBastion),
        47..=51 => Some(EnemyProfileId::HeptarchCore),
        _ => None,
    }
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn enemy_sprite_bounds_by_code(code: u8) -> Option<EnemySpriteBounds> {
    enemy_profile_for_sprite_code(code).map(enemy_sprite_bounds)
}

#[cfg_attr(not(test), allow(dead_code))]
fn enemy_sprite_layer_has_pixel(layer: EnemySpriteLayerDef, x: usize, y: usize) -> bool {
    let width = layer.width as usize;
    let bit_index = y * width + x;
    let byte = layer.bits[bit_index >> 3];
    let mask = 0x80 >> (bit_index & 7);
    (byte & mask) != 0
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub(crate) fn enemy_sprite_layer_def_by_code(code: u8) -> Option<EnemySpriteLayerDef> {
    match code {
        1 => Some(SCOUTDRONE_BASE_LAYER),
        2 => Some(SCOUTDRONE_DETAIL_A_LAYER),
        3 => Some(SCOUTDRONE_DETAIL_B_LAYER),
        4 => Some(NEEDLERDRONE_BASE_LAYER),
        5 => Some(NEEDLERDRONE_DETAIL_A_LAYER),
        6 => Some(NEEDLERDRONE_DETAIL_B_LAYER),
        7 => Some(RAMPARTDRONE_BASE_LAYER),
        8 => Some(RAMPARTDRONE_DETAIL_A_LAYER),
        9 => Some(RAMPARTDRONE_DETAIL_B_LAYER),
        10 => Some(SPINESENTRY_BASE_LAYER),
        11 => Some(SPINESENTRY_DETAIL_A_LAYER),
        12 => Some(PENTACORE_BASE_LAYER),
        13 => Some(PENTACORE_DETAIL_A_LAYER),
        14 => Some(PENTACORE_DETAIL_B_LAYER),
        15 => Some(PENTACORE_DETAIL_C_LAYER),
        16 => Some(VOLTMANTIS_BASE_LAYER),
        17 => Some(VOLTMANTIS_DETAIL_A_LAYER),
        18 => Some(VOLTMANTIS_DETAIL_B_LAYER),
        19 => Some(SHARDWEAVER_BASE_LAYER),
        20 => Some(SHARDWEAVER_DETAIL_A_LAYER),
        21 => Some(SHARDWEAVER_DETAIL_B_LAYER),
        22 => Some(PRISMARRAY_BASE_LAYER),
        23 => Some(PRISMARRAY_DETAIL_A_LAYER),
        24 => Some(PRISMARRAY_DETAIL_B_LAYER),
        25 => Some(GLASSBISHOP_BASE_LAYER),
        26 => Some(GLASSBISHOP_DETAIL_A_LAYER),
        27 => Some(GLASSBISHOP_DETAIL_B_LAYER),
        28 => Some(HEXARCHCORE_BASE_LAYER),
        29 => Some(HEXARCHCORE_DETAIL_A_LAYER),
        30 => Some(HEXARCHCORE_DETAIL_B_LAYER),
        31 => Some(HEXARCHCORE_DETAIL_C_LAYER),
        32 => Some(NULLRAIDER_BASE_LAYER),
        33 => Some(NULLRAIDER_DETAIL_A_LAYER),
        34 => Some(NULLRAIDER_DETAIL_B_LAYER),
        35 => Some(NULLRAIDER_DETAIL_C_LAYER),
        36 => Some(RIFTSTALKER_BASE_LAYER),
        37 => Some(RIFTSTALKER_DETAIL_A_LAYER),
        38 => Some(RIFTSTALKER_DETAIL_B_LAYER),
        39 => Some(RIFTSTALKER_DETAIL_C_LAYER),
        40 => Some(SIEGESPIDER_BASE_LAYER),
        41 => Some(SIEGESPIDER_DETAIL_A_LAYER),
        42 => Some(SIEGESPIDER_DETAIL_B_LAYER),
        43 => Some(RIFTBASTION_BASE_LAYER),
        44 => Some(RIFTBASTION_DETAIL_A_LAYER),
        45 => Some(RIFTBASTION_DETAIL_B_LAYER),
        46 => Some(RIFTBASTION_DETAIL_C_LAYER),
        47 => Some(HEPTARCHCORE_BASE_LAYER),
        48 => Some(HEPTARCHCORE_DETAIL_A_LAYER),
        49 => Some(HEPTARCHCORE_DETAIL_B_LAYER),
        50 => Some(HEPTARCHCORE_DETAIL_C_LAYER),
        51 => Some(HEPTARCHCORE_DETAIL_D_LAYER),
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
fn intent_def(
    name: &'static str,
    summary: &'static str,
    damage: i32,
    hits: u8,
    gain_block: i32,
    prime_bleed: u8,
    self_focus: i8,
    self_rhythm: i8,
    self_momentum: i8,
    target_focus: i8,
    target_rhythm: i8,
    target_momentum: i8,
    apply_bleed: u8,
) -> EnemyIntent {
    EnemyIntent {
        name,
        summary,
        damage,
        hits,
        gain_block,
        prime_bleed,
        self_focus,
        self_rhythm,
        self_momentum,
        target_focus,
        target_rhythm,
        target_momentum,
        apply_bleed,
    }
}

pub(crate) fn enemy_intent(profile: EnemyProfileId, index: usize) -> EnemyIntent {
    match (profile, index % 3) {
        (EnemyProfileId::ScoutDrone, 0) => intent_def(
            "Shock Needle",
            "Deal 5 damage.",
            5,
            1,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ),
        (EnemyProfileId::ScoutDrone, 1) => intent_def(
            "Crossfire",
            "Deal 3 damage twice.",
            3,
            2,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ),
        (EnemyProfileId::ScoutDrone, _) => intent_def(
            "Brace Cycle",
            "Gain 4 Shield. Gain Focus +1.",
            0,
            0,
            4,
            0,
            1,
            0,
            0,
            0,
            0,
            0,
            0,
        ),
        (EnemyProfileId::NeedlerDrone, 0) => intent_def(
            "Needle Tap",
            "Deal 4 damage. Apply Bleed 1.",
            4,
            1,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            1,
        ),
        (EnemyProfileId::NeedlerDrone, 1) => intent_def(
            "Split Sting",
            "Deal 2 damage three times.",
            2,
            3,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ),
        (EnemyProfileId::NeedlerDrone, _) => intent_def(
            "Stabilize",
            "Gain 4 Shield.",
            0,
            0,
            4,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ),
        (EnemyProfileId::RampartDrone, 0) => intent_def(
            "Ram Plate",
            "Deal 8 damage.",
            8,
            1,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ),
        (EnemyProfileId::RampartDrone, 1) => intent_def(
            "Pressure Clamp",
            "Deal 5 damage. Apply Focus -1.",
            5,
            1,
            0,
            0,
            0,
            0,
            0,
            -1,
            0,
            0,
            0,
        ),
        (EnemyProfileId::RampartDrone, _) => intent_def(
            "Reinforce Wall",
            "Gain 8 Shield. Next hit applies Bleed 2.",
            0,
            0,
            8,
            2,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ),
        (EnemyProfileId::SpineSentry, 0) => intent_def(
            "Spine Rack",
            "Deal 4 damage twice. Apply Bleed 1.",
            4,
            2,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            1,
        ),
        (EnemyProfileId::SpineSentry, 1) => intent_def(
            "Target Lock",
            "Deal 7 damage. Apply Momentum -1.",
            7,
            1,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            -1,
            0,
        ),
        (EnemyProfileId::SpineSentry, _) => intent_def(
            "Quill Plating",
            "Gain 9 Shield.",
            0,
            0,
            9,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ),
        (EnemyProfileId::PentaCore, 0) => intent_def(
            "Target Prism",
            "Deal 7 damage. Apply Momentum -1.",
            7,
            1,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            -1,
            0,
        ),
        (EnemyProfileId::PentaCore, 1) => intent_def(
            "Penta Bulwark",
            "Gain 10 Shield. Next hit applies Bleed 2.",
            0,
            0,
            10,
            2,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ),
        (EnemyProfileId::PentaCore, _) => intent_def(
            "Split Lattice",
            "Deal 4 damage three times.",
            4,
            3,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ),
        (EnemyProfileId::VoltMantis, 0) => {
            intent_def("Arc Cut", "Deal 8 damage.", 8, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0)
        }
        (EnemyProfileId::VoltMantis, 1) => intent_def(
            "Arc Lash",
            "Deal 4 damage twice.",
            4,
            2,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ),
        (EnemyProfileId::VoltMantis, _) => intent_def(
            "Surge Shell",
            "Gain 7 Shield. Gain Momentum +1.",
            0,
            0,
            7,
            0,
            0,
            0,
            1,
            0,
            0,
            0,
            0,
        ),
        (EnemyProfileId::ShardWeaver, 0) => intent_def(
            "Glass Cut",
            "Deal 6 damage. Apply Momentum -1.",
            6,
            1,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            -1,
            0,
        ),
        (EnemyProfileId::ShardWeaver, 1) => intent_def(
            "Mirror Volley",
            "Deal 3 damage twice. Gain 4 Shield.",
            3,
            2,
            4,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ),
        (EnemyProfileId::ShardWeaver, _) => intent_def(
            "Refocus",
            "Gain 8 Shield. Apply Rhythm -1.",
            0,
            0,
            8,
            0,
            0,
            0,
            0,
            0,
            -1,
            0,
            0,
        ),
        (EnemyProfileId::PrismArray, 0) => intent_def(
            "Prism Bite",
            "Deal 7 damage. Apply Momentum -1.",
            7,
            1,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            -1,
            0,
        ),
        (EnemyProfileId::PrismArray, 1) => intent_def(
            "Ion Salvo",
            "Deal 5 damage twice.",
            5,
            2,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ),
        (EnemyProfileId::PrismArray, _) => intent_def(
            "Prism Guard",
            "Gain 10 Shield.",
            0,
            0,
            10,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ),
        (EnemyProfileId::GlassBishop, 0) => intent_def(
            "Shatter Beam",
            "Deal 8 damage. Apply Momentum -1.",
            8,
            1,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            -1,
            0,
        ),
        (EnemyProfileId::GlassBishop, 1) => intent_def(
            "Split Halo",
            "Deal 5 damage twice. Gain 4 Shield.",
            5,
            2,
            4,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ),
        (EnemyProfileId::GlassBishop, _) => intent_def(
            "Faceted Ward",
            "Gain 10 Shield. Apply Bleed 1.",
            0,
            0,
            10,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            1,
        ),
        (EnemyProfileId::HexarchCore, 0) => intent_def(
            "Hex Shell",
            "Gain 12 Shield. Apply Focus -2.",
            0,
            0,
            12,
            0,
            0,
            0,
            0,
            -2,
            0,
            0,
            0,
        ),
        (EnemyProfileId::HexarchCore, 1) => intent_def(
            "Hex Breaker",
            "Deal 15 damage.",
            15,
            1,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ),
        (EnemyProfileId::HexarchCore, _) => intent_def(
            "Hex Volley",
            "Deal 5 damage three times.",
            5,
            3,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ),
        (EnemyProfileId::NullRaider, 0) => intent_def(
            "Null Shot",
            "Deal 10 damage.",
            10,
            1,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ),
        (EnemyProfileId::NullRaider, 1) => intent_def(
            "Chain Burst",
            "Deal 5 damage twice.",
            5,
            2,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ),
        (EnemyProfileId::NullRaider, _) => intent_def(
            "Null Guard",
            "Gain 9 Shield.",
            0,
            0,
            9,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ),
        (EnemyProfileId::RiftStalker, 0) => intent_def(
            "Rift Claw",
            "Deal 9 damage. Apply Bleed 1.",
            9,
            1,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            1,
        ),
        (EnemyProfileId::RiftStalker, 1) => intent_def(
            "Rend Salvo",
            "Deal 5 damage twice.",
            5,
            2,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ),
        (EnemyProfileId::RiftStalker, _) => intent_def(
            "Lock Anchor",
            "Gain 10 Shield. Apply Focus -1.",
            0,
            0,
            10,
            0,
            0,
            0,
            0,
            -1,
            0,
            0,
            0,
        ),
        (EnemyProfileId::SiegeSpider, 0) => intent_def(
            "Bulwark Hammer",
            "Deal 11 damage.",
            11,
            1,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ),
        (EnemyProfileId::SiegeSpider, 1) => intent_def(
            "Lock Volley",
            "Deal 6 damage twice.",
            6,
            2,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ),
        (EnemyProfileId::SiegeSpider, _) => intent_def(
            "Bulwark Seal",
            "Gain 12 Shield. Apply Focus -1.",
            0,
            0,
            12,
            0,
            0,
            0,
            0,
            -1,
            0,
            0,
            0,
        ),
        (EnemyProfileId::RiftBastion, 0) => intent_def(
            "Grav Hammer",
            "Deal 12 damage.",
            12,
            1,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ),
        (EnemyProfileId::RiftBastion, 1) => intent_def(
            "Collapse Grid",
            "Deal 6 damage twice. Apply Momentum -1.",
            6,
            2,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            -1,
            0,
        ),
        (EnemyProfileId::RiftBastion, _) => intent_def(
            "Anchor Field",
            "Gain 12 Shield. Next hit applies Bleed 3.",
            0,
            0,
            12,
            3,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ),
        (EnemyProfileId::HeptarchCore, 0) => intent_def(
            "Singularity Shell",
            "Gain 16 Shield. Next hit applies Bleed 3.",
            0,
            0,
            16,
            3,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ),
        (EnemyProfileId::HeptarchCore, 1) => intent_def(
            "Array Collapse",
            "Deal 8 damage twice. Apply Momentum -1.",
            8,
            2,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            -1,
            0,
        ),
        (EnemyProfileId::HeptarchCore, _) => intent_def(
            "Crown Breaker",
            "Deal 20 damage.",
            20,
            1,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_PRIMARY_SEED: u64 = 0x0BAD_5EED;
    const TEST_ALT_SEED: u64 = 0xDEAD_BEEF;
    const TEST_BOSS_REWARD_SEED: u64 = 0xBAAD_F00D;

    #[test]
    fn event_order_is_deterministic_and_non_repeating_within_each_level() {
        for level in 1..=3 {
            let a = ordered_events_for_level(TEST_PRIMARY_SEED, level);
            let b = ordered_events_for_level(TEST_PRIMARY_SEED, level);

            assert_eq!(a, b);
            assert_ne!(a[0], a[1]);
        }
    }

    #[test]
    fn event_pools_are_fixed_per_level() {
        assert_eq!(
            event_pool_for_level(1),
            [EventId::SalvageCache, EventId::RelayTerminal]
        );
        assert_eq!(
            event_pool_for_level(2),
            [EventId::ClinicPod, EventId::ExchangeConsole]
        );
        assert_eq!(
            event_pool_for_level(3),
            [EventId::PrototypeRack, EventId::CoolingVault]
        );
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
    fn new_event_choices_have_expected_effects() {
        assert_eq!(
            event_choice_effect(EventId::RelayTerminal, 0, 1),
            Some(EventChoiceEffect::AddCard(CardId::CoverPulse))
        );
        assert_eq!(
            event_choice_effect(EventId::RelayTerminal, 1, 3),
            Some(EventChoiceEffect::LoseHpAddCard {
                lose_hp: 4,
                card: CardId::TacticalBurst,
            })
        );
        assert_eq!(
            event_choice_effect(EventId::ExchangeConsole, 0, 2),
            Some(EventChoiceEffect::GainCredits(22))
        );
        assert_eq!(
            event_choice_effect(EventId::ExchangeConsole, 1, 1),
            Some(EventChoiceEffect::LoseHpGainCredits {
                lose_hp: 5,
                gain_credits: 36,
            })
        );
        assert_eq!(
            event_choice_effect(EventId::CoolingVault, 0, 3),
            Some(EventChoiceEffect::Heal(12))
        );
        assert_eq!(
            event_choice_effect(EventId::CoolingVault, 1, 2),
            Some(EventChoiceEffect::LoseHpGainMaxHp {
                lose_hp: 6,
                gain_max_hp: 5,
            })
        );
    }

    #[test]
    fn new_events_are_localized() {
        assert_eq!(
            localized_event_title(EventId::RelayTerminal, Language::Spanish),
            "Terminal de Relé"
        );
        assert_eq!(
            localized_event_title(EventId::ExchangeConsole, Language::Spanish),
            "Consola de Intercambio"
        );
        assert_eq!(
            localized_event_title(EventId::CoolingVault, Language::Spanish),
            "Bóveda Criogénica"
        );
        assert_eq!(
            localized_event_flavor(EventId::ExchangeConsole, Language::Spanish),
            "Una consola de intercambio aún te deja cambiar piezas útiles por créditos."
        );
        assert_eq!(
            localized_event_choice_title(EventId::RelayTerminal, 0, Language::Spanish),
            Some("Copiar la rutina de escudo")
        );
        assert_eq!(
            localized_event_choice_title(EventId::ExchangeConsole, 1, Language::Spanish),
            Some("Vender la reserva de refrigerante")
        );
        assert_eq!(
            localized_event_choice_title(EventId::CoolingVault, 1, Language::Spanish),
            Some("Aguantar la cámara helada")
        );
        assert_eq!(
            localized_event_choice_body(EventId::RelayTerminal, 1, 2, Language::Spanish),
            Some("Pierde 4 HP. Añade Impulso Táctico a tu mazo.".to_string())
        );
        assert_eq!(
            localized_event_choice_body(EventId::ExchangeConsole, 0, 2, Language::English),
            Some("Gain 22 Credits.".to_string())
        );
    }

    #[test]
    fn starter_module_choices_always_include_all_three_modules() {
        let choices = starter_module_choices();
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
        let a = starter_module_choices();
        let b = starter_module_choices();

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
    fn combat_rewards_include_the_expanded_staples() {
        let pool = reward_pool(RewardTier::Combat, 1);

        assert!(pool.contains(&CardId::PinpointJab));
        assert!(pool.contains(&CardId::SignalTap));
        assert!(pool.contains(&CardId::BurstArray));
        assert!(pool.contains(&CardId::CoverPulse));
        assert!(pool.contains(&CardId::RiftDart));
        assert!(pool.contains(&CardId::MarkPulse));
        assert!(pool.contains(&CardId::BraceCircuit));
        assert!(pool.contains(&CardId::FaultShot));
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
        assert!(pool.contains(&CardId::TerminalLoop));
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
        assert_eq!(upgraded_card(CardId::RiftDart), Some(CardId::RiftDartPlus));
        assert_eq!(
            upgraded_card(CardId::TerminalLoop),
            Some(CardId::TerminalLoopPlus)
        );
        assert_eq!(upgraded_card(CardId::FlareSlashPlus), None);
        assert_eq!(upgraded_card(CardId::ZeroPointPlus), None);
        assert_eq!(upgraded_card(CardId::OverwatchGridPlus), None);
        assert_eq!(upgraded_card(CardId::TerminalLoopPlus), None);
    }

    #[test]
    fn all_base_cards_are_unique_and_exclude_upgraded_variants() {
        let cards = all_base_cards();
        let mut unique_cards = cards.to_vec();
        unique_cards.sort_by_key(|card| *card as u8);
        unique_cards.dedup();

        assert_eq!(cards.len(), 64);
        assert_eq!(unique_cards.len(), cards.len());
        assert!(cards.iter().all(|card| upgraded_card(*card).is_some()));
        assert!(
            cards
                .iter()
                .filter_map(|card| upgraded_card(*card))
                .all(|card| !cards.contains(&card))
        );
    }

    #[test]
    fn expanded_cards_expose_expected_defs() {
        let rift = card_def(CardId::RiftDart);
        let mark = card_def(CardId::MarkPulse);
        let brace = card_def(CardId::BraceCircuit);
        let fault = card_def(CardId::FaultShot);
        let sever = card_def(CardId::SeverArc);
        let lockbreaker = card_def(CardId::Lockbreaker);
        let counter = card_def(CardId::CounterLattice);
        let terminal = card_def(CardId::TerminalLoop);

        assert_eq!(rift.name, "Rift Dart");
        assert_eq!(rift.cost, 1);
        assert_eq!(rift.target, CardTarget::Enemy);
        assert_eq!(mark.cost, 0);
        assert_eq!(
            mark.description,
            "Apply Momentum -1. If target has Bleed, gain 4 Shield."
        );
        assert_eq!(brace.target, CardTarget::SelfOnly);
        assert_eq!(
            brace.description,
            "Gain 6 Shield. If you already have Shield, draw 1."
        );
        assert_eq!(
            fault.description,
            "Deal 5 damage. If target Focus < 0, gain Focus +1."
        );
        assert_eq!(sever.cost, 2);
        assert_eq!(sever.target, CardTarget::Enemy);
        assert_eq!(
            lockbreaker.description,
            "Deal 6 damage. If target Focus < 0, apply Focus -1 and gain 6 Shield."
        );
        assert_eq!(counter.target, CardTarget::Enemy);
        assert_eq!(
            counter.description,
            "Deal 6 damage. If target Focus < 0, gain 1 Energy."
        );
        assert_eq!(terminal.cost, 2);
        assert_eq!(terminal.target, CardTarget::Enemy);
    }

    #[test]
    fn expanded_cards_ship_with_english_and_spanish_localization() {
        assert_eq!(
            localized_card_name(CardId::RiftDart, Language::English),
            "Rift Dart"
        );
        assert_eq!(
            localized_card_name(CardId::RiftDart, Language::Spanish),
            "Dardo de Brecha"
        );
        assert_eq!(
            localized_card_name(CardId::CounterLattice, Language::Spanish),
            "Trama de Respuesta"
        );
        assert_eq!(
            localized_card_def(CardId::BraceCircuit, Language::Spanish).description,
            "Gana 6 de Escudo. Si ya tienes Escudo, roba 1."
        );
        assert_eq!(
            localized_card_def(CardId::CounterLatticePlus, Language::Spanish).description,
            "Inflige 8 de daño. Si el objetivo tiene Enfoque < 0, gana 1 de Energía."
        );
        assert_eq!(
            localized_card_def(CardId::TerminalLoopPlus, Language::English).description,
            "Deal 15 damage. If target has Bleed, draw 1. If target Momentum < 0, gain Focus +2."
        );
    }

    #[test]
    fn early_enemy_intents_include_the_new_status_effects() {
        let scout_brace = enemy_intent(EnemyProfileId::ScoutDrone, 2);
        let rampart_clamp = enemy_intent(EnemyProfileId::RampartDrone, 1);
        let shard_refocus = enemy_intent(EnemyProfileId::ShardWeaver, 2);

        assert_eq!(scout_brace.gain_block, 4);
        assert_eq!(scout_brace.self_focus, 1);
        assert_eq!(scout_brace.summary, "Gain 4 Shield. Gain Focus +1.");

        assert_eq!(rampart_clamp.damage, 5);
        assert_eq!(rampart_clamp.target_focus, -1);
        assert_eq!(rampart_clamp.target_rhythm, 0);
        assert_eq!(rampart_clamp.summary, "Deal 5 damage. Apply Focus -1.");

        assert_eq!(shard_refocus.gain_block, 8);
        assert_eq!(shard_refocus.target_rhythm, -1);
        assert_eq!(shard_refocus.summary, "Gain 8 Shield. Apply Rhythm -1.");
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
        assert_eq!(penta_open.target_momentum, -1);
        assert_eq!(penta_mid.name, "Penta Bulwark");
        assert_eq!(penta_mid.gain_block, 10);
        assert_eq!(penta_mid.prime_bleed, 2);
        assert_eq!(penta_close.name, "Split Lattice");
        assert_eq!(penta_close.damage, 4);
        assert_eq!(penta_close.hits, 3);

        assert_eq!(hex_open.name, "Hex Shell");
        assert_eq!(hex_open.gain_block, 12);
        assert_eq!(hex_open.target_focus, -2);
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
        assert_eq!(hept_mid.target_momentum, -1);
        assert_eq!(hept_close.name, "Crown Breaker");
        assert_eq!(hept_close.damage, 20);
    }

    #[test]
    fn enemy_sprite_layer_codes_are_unique() {
        let profiles = [
            EnemyProfileId::ScoutDrone,
            EnemyProfileId::NeedlerDrone,
            EnemyProfileId::RampartDrone,
            EnemyProfileId::SpineSentry,
            EnemyProfileId::PentaCore,
            EnemyProfileId::VoltMantis,
            EnemyProfileId::ShardWeaver,
            EnemyProfileId::PrismArray,
            EnemyProfileId::GlassBishop,
            EnemyProfileId::HexarchCore,
            EnemyProfileId::NullRaider,
            EnemyProfileId::RiftStalker,
            EnemyProfileId::SiegeSpider,
            EnemyProfileId::RiftBastion,
            EnemyProfileId::HeptarchCore,
        ];
        let mut codes = Vec::new();

        for profile in profiles {
            let sprite = enemy_sprite_def(profile);
            codes.extend(sprite.layers.iter().map(|layer| layer.code));
        }

        let total = codes.len();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), total);
    }

    #[test]
    fn enemy_profiles_map_to_expected_levels() {
        assert_eq!(enemy_profile_level(EnemyProfileId::ScoutDrone), 1);
        assert_eq!(enemy_profile_level(EnemyProfileId::PentaCore), 1);
        assert_eq!(enemy_profile_level(EnemyProfileId::VoltMantis), 2);
        assert_eq!(enemy_profile_level(EnemyProfileId::HexarchCore), 2);
        assert_eq!(enemy_profile_level(EnemyProfileId::NullRaider), 3);
        assert_eq!(enemy_profile_level(EnemyProfileId::HeptarchCore), 3);
    }

    #[test]
    fn enemy_sprites_are_multilayer_packed_and_resolvable() {
        let profiles = [
            EnemyProfileId::ScoutDrone,
            EnemyProfileId::NeedlerDrone,
            EnemyProfileId::RampartDrone,
            EnemyProfileId::SpineSentry,
            EnemyProfileId::PentaCore,
            EnemyProfileId::VoltMantis,
            EnemyProfileId::ShardWeaver,
            EnemyProfileId::PrismArray,
            EnemyProfileId::GlassBishop,
            EnemyProfileId::HexarchCore,
            EnemyProfileId::NullRaider,
            EnemyProfileId::RiftStalker,
            EnemyProfileId::SiegeSpider,
            EnemyProfileId::RiftBastion,
            EnemyProfileId::HeptarchCore,
        ];

        for profile in profiles {
            let sprite = enemy_sprite_def(profile);
            assert!(sprite.layers.len() >= 2);

            for layer in sprite.layers {
                let expected_len = (layer.width as usize * layer.height as usize).div_ceil(8);
                assert_eq!(layer.bits.len(), expected_len);
                assert!(layer.bits.iter().any(|&byte| byte != 0));
                assert_eq!(enemy_sprite_layer_def_by_code(layer.code), Some(*layer));
            }
        }
    }
    #[test]
    fn enemy_sprites_fill_at_least_one_canvas_axis() {
        let profiles = [
            EnemyProfileId::ScoutDrone,
            EnemyProfileId::NeedlerDrone,
            EnemyProfileId::RampartDrone,
            EnemyProfileId::SpineSentry,
            EnemyProfileId::PentaCore,
            EnemyProfileId::VoltMantis,
            EnemyProfileId::ShardWeaver,
            EnemyProfileId::PrismArray,
            EnemyProfileId::GlassBishop,
            EnemyProfileId::HexarchCore,
            EnemyProfileId::NullRaider,
            EnemyProfileId::RiftStalker,
            EnemyProfileId::SiegeSpider,
            EnemyProfileId::RiftBastion,
            EnemyProfileId::HeptarchCore,
        ];

        for profile in profiles {
            let sprite = enemy_sprite_def(profile);
            let bounds = enemy_sprite_bounds(profile);
            let mut min_x = sprite.width as i32;
            let mut max_x = -1i32;
            let mut min_y = sprite.height as i32;
            let mut max_y = -1i32;

            for y in 0..sprite.height as usize {
                for x in 0..sprite.width as usize {
                    let active = sprite
                        .layers
                        .iter()
                        .any(|layer| enemy_sprite_layer_has_pixel(*layer, x, y));
                    if !active {
                        continue;
                    }
                    min_x = min_x.min(x as i32);
                    max_x = max_x.max(x as i32);
                    min_y = min_y.min(y as i32);
                    max_y = max_y.max(y as i32);
                }
            }

            assert!(max_x >= min_x && max_y >= min_y);
            assert_eq!(bounds.left, min_x as u8);
            assert_eq!(bounds.top, min_y as u8);
            assert_eq!(bounds.width, (max_x - min_x + 1) as u8);
            assert_eq!(bounds.height, (max_y - min_y + 1) as u8);
            assert!(
                bounds.width == sprite.width || bounds.height == sprite.height,
                "{profile:?} should fill width or height after source normalization"
            );
        }
    }

    #[test]
    fn enemy_sprite_codes_share_their_profile_union_bounds() {
        let profiles = [
            EnemyProfileId::ScoutDrone,
            EnemyProfileId::NeedlerDrone,
            EnemyProfileId::RampartDrone,
            EnemyProfileId::SpineSentry,
            EnemyProfileId::PentaCore,
            EnemyProfileId::VoltMantis,
            EnemyProfileId::ShardWeaver,
            EnemyProfileId::PrismArray,
            EnemyProfileId::GlassBishop,
            EnemyProfileId::HexarchCore,
            EnemyProfileId::NullRaider,
            EnemyProfileId::RiftStalker,
            EnemyProfileId::SiegeSpider,
            EnemyProfileId::RiftBastion,
            EnemyProfileId::HeptarchCore,
        ];

        for profile in profiles {
            let expected = enemy_sprite_bounds(profile);
            for layer in enemy_sprite_def(profile).layers {
                assert_eq!(enemy_sprite_bounds_by_code(layer.code), Some(expected));
            }
        }
    }
}
