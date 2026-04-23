use crate::content::ModuleId;
use crate::dungeon::DungeonRun;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PostVictoryModuleEffects {
    pub(crate) nanoforge_healed: i32,
    pub(crate) salvage_applied: bool,
    pub(crate) recovery_healed: i32,
}

pub(crate) fn combat_seed_for_dungeon(dungeon: &DungeonRun) -> u64 {
    let node_contribution =
        (dungeon.current_node.unwrap_or_default() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15_u64);
    dungeon.seed.wrapping_add(node_contribution)
}

pub(crate) fn apply_post_victory_modules(dungeon: &mut DungeonRun) -> PostVictoryModuleEffects {
    let nanoforge_healed = if dungeon.has_module(ModuleId::Nanoforge) {
        dungeon.recover_hp(2)
    } else {
        0
    };
    let salvage_applied = dungeon.has_module(ModuleId::SalvageLedger);
    if salvage_applied {
        dungeon.credits = dungeon.credits.saturating_add(4);
    }
    let recovery_healed = if dungeon.has_module(ModuleId::RecoveryMatrix) {
        dungeon.recover_hp(5)
    } else {
        0
    };

    PostVictoryModuleEffects {
        nanoforge_healed,
        salvage_applied,
        recovery_healed,
    }
}

#[cfg(test)]
mod tests {
    use super::{PostVictoryModuleEffects, apply_post_victory_modules, combat_seed_for_dungeon};
    use crate::combat::{CombatState, EncounterEnemySetup, EncounterSetup};
    use crate::content::{EnemyProfileId, ModuleId};
    use crate::dungeon::DungeonRun;

    const TEST_SEED: u64 = 0x51A7_C0DE;

    #[test]
    fn combat_seed_uses_current_node_index() {
        let mut dungeon = DungeonRun::new(TEST_SEED);
        dungeon.current_node = Some(7);

        assert_eq!(
            combat_seed_for_dungeon(&dungeon),
            TEST_SEED.wrapping_add(7_u64.wrapping_mul(0x9E37_79B9_7F4A_7C15_u64))
        );
    }

    #[test]
    fn room_seed_matches_current_room_formula_for_selected_node() {
        let mut dungeon = DungeonRun::new(TEST_SEED);
        dungeon.current_level = 2;
        dungeon.current_node = Some(3);

        assert_eq!(
            dungeon.room_seed_for(3),
            dungeon.current_room_seed().unwrap()
        );
    }

    #[test]
    fn start_of_combat_modules_apply_shared_effects_in_order() {
        let setup = EncounterSetup {
            player_hp: 24,
            player_max_hp: 24,
            player_max_energy: 3,
            enemies: vec![EncounterEnemySetup {
                hp: 12,
                max_hp: 12,
                block: 0,
                profile: EnemyProfileId::ScoutDrone,
                intent_index: 0,
                on_hit_bleed: 0,
            }],
        };
        let (mut combat, _) = CombatState::new_with_setup(TEST_SEED, setup);

        let applied = combat.apply_start_of_combat_modules(&[
            ModuleId::TargetingRelay,
            ModuleId::AegisDrive,
            ModuleId::OverclockCore,
            ModuleId::SuppressionField,
        ]);

        assert_eq!(
            applied,
            vec![
                ModuleId::TargetingRelay,
                ModuleId::AegisDrive,
                ModuleId::OverclockCore,
                ModuleId::SuppressionField,
            ]
        );
        assert_eq!(combat.player.fighter.block, 5);
        assert_eq!(combat.player.fighter.statuses.focus, 1);
        assert_eq!(combat.player.max_energy, 4);
        assert_eq!(combat.player.energy, 4);
        assert_eq!(combat.enemies[0].fighter.statuses.focus, -1);
    }

    #[test]
    fn post_victory_modules_apply_recovery_and_credits() {
        let mut dungeon = DungeonRun::new(TEST_SEED);
        dungeon.modules = vec![
            ModuleId::Nanoforge,
            ModuleId::SalvageLedger,
            ModuleId::RecoveryMatrix,
        ];
        dungeon.player_hp = 20;
        dungeon.player_max_hp = 32;
        dungeon.credits = 9;

        let effects = apply_post_victory_modules(&mut dungeon);

        assert_eq!(
            effects,
            PostVictoryModuleEffects {
                nanoforge_healed: 2,
                salvage_applied: true,
                recovery_healed: 5,
            }
        );
        assert_eq!(dungeon.player_hp, 27);
        assert_eq!(dungeon.credits, 13);
    }
}
