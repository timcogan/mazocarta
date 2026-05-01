#![cfg_attr(target_arch = "wasm32", allow(dead_code))]

use crate::combat::{Actor, CombatAction, CombatEvent, CombatOutcome, CombatState};
use crate::content::{
    CardArchetype, CardId, CardTarget, EventChoiceEffect, EventId, ModuleId, RewardTier,
    boss_module_choices, card_def, event_choice_effect, shop_offers, starter_module_choices,
    upgraded_card,
};
use crate::dungeon::{DungeonNode, DungeonRun, RoomKind};
use crate::party::PartyRunState;

const SHOP_PURCHASE_THRESHOLD: i32 = 30;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RestChoice {
    Heal,
    Upgrade(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShopChoice {
    Buy(usize),
    Leave,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RewardChoice {
    Pick(usize),
    Skip,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CombatChoice {
    PlayCard {
        hand_index: usize,
        target_enemy: Option<usize>,
    },
    EndTurn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ActionCandidate {
    action: CombatAction,
    hand_index: Option<usize>,
    enemy_index: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ActionAnalysis {
    candidate: ActionCandidate,
    score: i32,
    state_score: i32,
    damage: i32,
    block_gain: i32,
    threat_reduction: i32,
    draw_count: i32,
    created_count: i32,
    energy_gain: i32,
    enemy_kills: i32,
    uncovered_after: i32,
    zero_cost: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CardAddPolicy {
    ZeroCostOnly,
    DefensiveFallback,
    Skip,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ScoredChoice<T> {
    score: i32,
    key: usize,
    value: T,
    zero_cost: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CombatSnapshot {
    player_hp: i32,
    player_block: i32,
    player_energy: u8,
    player_bleed: u8,
    player_focus: i8,
    player_rhythm: i8,
    player_momentum: i8,
    enemy_total_hp: i32,
    enemy_total_block: i32,
    enemy_alive_count: usize,
    enemy_bleed_sum: u8,
    enemy_focus_sum: i32,
    enemy_rhythm_sum: i32,
    enemy_momentum_sum: i32,
}

pub(crate) fn pick_starter_module(dungeon: &DungeonRun) -> Option<ModuleId> {
    let choices = starter_module_choices();
    choices
        .iter()
        .find(|&&module| module == ModuleId::Nanoforge)
        .copied()
        .or_else(|| choose_best_module(dungeon, &choices))
}

pub(crate) fn choose_best_module(dungeon: &DungeonRun, options: &[ModuleId]) -> Option<ModuleId> {
    options
        .iter()
        .copied()
        .map(|module| (module_score(dungeon, module), module))
        .max_by(|(score_a, module_a), (score_b, module_b)| {
            score_a
                .cmp(score_b)
                .then((*module_b as u8).cmp(&(*module_a as u8)))
        })
        .map(|(_, module)| module)
}

pub(crate) fn pick_map_node(dungeon: &DungeonRun) -> Option<usize> {
    dungeon
        .available_nodes
        .iter()
        .copied()
        .filter_map(|node_id| {
            dungeon
                .node(node_id)
                .map(|node| (map_node_score(dungeon, node), node_id))
        })
        .max_by(|(score_a, id_a), (score_b, id_b)| score_a.cmp(score_b).then(id_b.cmp(id_a)))
        .map(|(_, node_id)| node_id)
}

pub(crate) fn pick_party_map_node(party_run: &PartyRunState) -> Option<usize> {
    let lead_dungeon = party_run.active_dungeon(0)?;
    lead_dungeon
        .available_nodes
        .iter()
        .copied()
        .filter_map(|node_id| {
            let node = lead_dungeon.node(node_id)?;
            let mut total_score = 0i32;
            let mut any_alive = false;
            for slot in 0..party_run.party_size() {
                let Some(hero) = party_run.hero(slot) else {
                    continue;
                };
                if hero.player_hp <= 0 {
                    continue;
                }
                let Some(dungeon) = party_run.active_dungeon(slot) else {
                    continue;
                };
                total_score += map_node_score(&dungeon, node);
                any_alive = true;
            }
            if any_alive {
                Some((total_score, node_id))
            } else {
                None
            }
        })
        .max_by(|(score_a, id_a), (score_b, id_b)| score_a.cmp(score_b).then(id_b.cmp(id_a)))
        .map(|(_, node_id)| node_id)
}

pub(crate) fn pick_rest_choice(dungeon: &DungeonRun) -> Option<RestChoice> {
    if should_heal_at_rest(dungeon) {
        Some(RestChoice::Heal)
    } else {
        pick_rest_upgrade(dungeon).map(RestChoice::Upgrade)
    }
}

pub(crate) fn pick_shop_choice(
    dungeon: &DungeonRun,
    offers: &[crate::content::ShopOffer],
) -> ShopChoice {
    pick_shop_offer(dungeon, offers)
        .map(ShopChoice::Buy)
        .unwrap_or(ShopChoice::Leave)
}

pub(crate) fn pick_reward_choice(
    dungeon: &DungeonRun,
    options: &[CardId],
    tier: RewardTier,
) -> RewardChoice {
    let (policy, choices) = scored_reward_choices(dungeon, options, tier);
    best_scored_choice_index(&choices, policy, reward_pick_threshold(dungeon, tier) - 16)
        .map(RewardChoice::Pick)
        .unwrap_or(RewardChoice::Skip)
}

pub(crate) fn pick_event_choice(dungeon: &DungeonRun, event: EventId) -> Option<usize> {
    best_event_choice(dungeon, event).map(|(_, choice_index)| choice_index)
}

pub(crate) fn choose_combat_action(combat: &CombatState) -> CombatChoice {
    let analyses = collect_action_analyses(combat);
    let end_turn = analyses
        .iter()
        .copied()
        .find(|analysis| matches!(analysis.candidate.action, CombatAction::EndTurn))
        .unwrap_or(ActionAnalysis {
            candidate: ActionCandidate {
                action: CombatAction::EndTurn,
                hand_index: None,
                enemy_index: None,
            },
            score: i32::MIN,
            state_score: i32::MIN,
            damage: 0,
            block_gain: 0,
            threat_reduction: 0,
            draw_count: 0,
            created_count: 0,
            energy_gain: 0,
            enemy_kills: 0,
            uncovered_after: 0,
            zero_cost: false,
        });
    let mut non_end = analyses
        .into_iter()
        .filter(|analysis| !matches!(analysis.candidate.action, CombatAction::EndTurn))
        .collect::<Vec<_>>();
    if non_end.is_empty() {
        return CombatChoice::EndTurn;
    }
    non_end.sort_by(|analysis_a, analysis_b| compare_action_analyses(*analysis_b, *analysis_a));

    let choice = if let Some(analysis) = non_end
        .iter()
        .copied()
        .find(|analysis| action_is_victory(*analysis))
    {
        analysis.candidate
    } else {
        let uncovered_before = (expected_enemy_threat(combat) - combat.player.fighter.block).max(0);
        if uncovered_before > 0 {
            if let Some(analysis) = non_end
                .iter()
                .copied()
                .filter(|analysis| action_is_defensive(*analysis, uncovered_before))
                .max_by(|analysis_a, analysis_b| {
                    compare_defensive_actions(*analysis_a, *analysis_b)
                })
            {
                analysis.candidate
            } else if let Some(analysis) = non_end
                .iter()
                .copied()
                .find(|analysis| action_is_useful_zero_cost(*analysis, end_turn.score))
            {
                analysis.candidate
            } else if let Some(analysis) = non_end
                .iter()
                .copied()
                .find(|analysis| action_is_setup(*analysis, end_turn.score))
            {
                analysis.candidate
            } else if let Some(analysis) = non_end
                .iter()
                .copied()
                .find(|analysis| action_is_positive_trade(*analysis, end_turn.score))
            {
                analysis.candidate
            } else {
                end_turn.candidate
            }
        } else if let Some(analysis) = non_end
            .iter()
            .copied()
            .find(|analysis| action_is_useful_zero_cost(*analysis, end_turn.score))
        {
            analysis.candidate
        } else if let Some(analysis) = non_end
            .iter()
            .copied()
            .find(|analysis| action_is_setup(*analysis, end_turn.score))
        {
            analysis.candidate
        } else if let Some(analysis) = non_end
            .iter()
            .copied()
            .find(|analysis| action_is_positive_trade(*analysis, end_turn.score))
        {
            analysis.candidate
        } else {
            end_turn.candidate
        }
    };

    match choice.action {
        CombatAction::PlayCard { hand_index, target } => CombatChoice::PlayCard {
            hand_index,
            target_enemy: match target {
                Some(Actor::Enemy(index)) => Some(index),
                _ => None,
            },
        },
        CombatAction::EndTurn => CombatChoice::EndTurn,
    }
}

fn module_score(dungeon: &DungeonRun, module: ModuleId) -> i32 {
    let hp = hp_percent(dungeon);
    let missing_hp = dungeon.player_max_hp - dungeon.player_hp;
    let offensive_cards = dungeon
        .deck
        .iter()
        .filter(|&&card| !matches!(card_def(card).target, CardTarget::SelfOnly))
        .count() as i32;
    let defensive_cards = dungeon.deck.len() as i32 - offensive_cards;
    let expensive_cards = dungeon
        .deck
        .iter()
        .filter(|&&card| card_def(card).cost >= 2)
        .count() as i32;
    let zero_cost_cards = dungeon
        .deck
        .iter()
        .filter(|&&card| card_def(card).cost == 0)
        .count() as i32;
    let draw_cards = dungeon
        .deck
        .iter()
        .filter(|&&card| card_draw_value(card) > 0)
        .count() as i32;
    let disruption_cards = dungeon
        .deck
        .iter()
        .filter(|&&card| card_disruption_value(card) > 0)
        .count() as i32;
    let momentum_cards = dungeon
        .deck
        .iter()
        .filter(|&&card| card_momentum_value(card) > 0)
        .count() as i32;
    let burst_cards = dungeon
        .deck
        .iter()
        .filter(|&&card| card_burst_value(card) >= 10)
        .count() as i32;
    let boss_cards = dungeon
        .deck
        .iter()
        .filter(|&&card| matches!(card_def(card).reward_tier, Some(RewardTier::Boss)))
        .count() as i32;
    let deck_power = deck_power_score(dungeon);
    let current_level = dungeon.current_level() as i32;

    match module {
        ModuleId::AegisDrive => 136 + missing_hp * 3 + defensive_cards * 2 + (70 - hp).max(0),
        ModuleId::TargetingRelay => {
            178 + offensive_cards * 4 + draw_cards * 6 + burst_cards * 5 + boss_cards * 4
        }
        ModuleId::Nanoforge => 86 + missing_hp * 5 + (52 - hp).max(0) * 3,
        ModuleId::CapacitorBank => {
            182 + momentum_cards * 12
                + draw_cards * 6
                + expensive_cards * 5
                + zero_cost_cards * 3
                + deck_power.max(0)
        }
        ModuleId::PrismScope => {
            174 + current_level * 12 + disruption_cards * 7 + offensive_cards * 2
        }
        ModuleId::SalvageLedger => {
            56 + current_level * 8 + ((14_i32 - dungeon.credits as i32).max(0))
        }
        ModuleId::OverclockCore => {
            210 + draw_cards * 8
                + expensive_cards * 9
                + zero_cost_cards * 4
                + boss_cards * 5
                + deck_power.max(0)
                - (72 - hp).max(0) * 2
        }
        ModuleId::SuppressionField => {
            214 + current_level * 18
                + disruption_cards * 8
                + offensive_cards * 3
                + (84 - hp).max(0) * 2
        }
        ModuleId::RecoveryMatrix => 144 + missing_hp * 10 + (78 - hp).max(0) * 3,
    }
}

fn map_node_score(dungeon: &DungeonRun, node: &DungeonNode) -> i32 {
    let hp = hp_percent(dungeon);
    let missing_hp = dungeon.player_max_hp - dungeon.player_hp;
    let deck_power = deck_power_score(dungeon);
    let mut score = match node.kind {
        RoomKind::Boss => 1000,
        RoomKind::Rest => 690,
        RoomKind::Event => 620,
        RoomKind::Shop => 500,
        RoomKind::Combat => 360,
        RoomKind::Elite => 300,
        RoomKind::Start => 0,
    };

    match node.kind {
        RoomKind::Elite => {
            score += (deck_power - 22) * 8;
            if dungeon.current_level() == 1 && hp > 92 && missing_hp <= 5 && deck_power > 18 {
                score += 120;
            }
            if dungeon.current_level() >= 2 && hp > 95 && deck_power > 26 {
                score += 90;
            }
            if hp < 88 {
                score -= 220;
            }
            if hp < 76 {
                score -= 260;
            }
            if hp < 64 {
                score -= 300;
            }
        }
        RoomKind::Rest => {
            if hp < 72 || missing_hp >= 8 {
                score += 260;
            }
            if hp < 55 || missing_hp >= 14 {
                score += 180;
            }
            if pick_rest_upgrade(dungeon).is_some() {
                score += 80;
            }
        }
        RoomKind::Shop => {
            let offers = shop_offers(dungeon.room_seed_for(node.id), dungeon.current_level());
            let (policy, choices) = scored_shop_choices(dungeon, &offers);
            let has_good_purchase =
                best_scored_choice_index(&choices, policy, SHOP_PURCHASE_THRESHOLD - 16).is_some();
            if has_good_purchase {
                score += 140;
            } else {
                score -= 200;
            }
            if dungeon.credits < 24 {
                score -= 60;
            }
        }
        RoomKind::Event => {
            if let Some(event) = dungeon.event_for_node(node.id) {
                score += pick_event_choice_score(dungeon, event) / 3;
            }
        }
        RoomKind::Combat => {
            if hp < 55 {
                score -= 80;
            }
            if hp < 40 {
                score -= 110;
            }
        }
        RoomKind::Boss | RoomKind::Start => {}
    }

    score
}

fn should_heal_at_rest(dungeon: &DungeonRun) -> bool {
    let hp = hp_percent(dungeon);
    let Some(deck_index) = pick_rest_upgrade(dungeon) else {
        return true;
    };
    let Some(base_card) = dungeon.deck.get(deck_index).copied() else {
        return true;
    };
    let Some(upgraded) = upgraded_card(base_card) else {
        return true;
    };
    let upgrade_gain =
        card_value(upgraded, &dungeon.deck, hp) - card_value(base_card, &dungeon.deck, hp);
    let zero_cost_upgrade = is_zero_cost_card(base_card);
    let weak_upgrade_threshold = if zero_cost_upgrade { 9 } else { 16 };
    let worthless_upgrade_threshold = if zero_cost_upgrade { 0 } else { 4 };

    hp < 72
        || dungeon.rest_heal_amount() >= DungeonRun::rest_heal_cap()
        || (hp < if zero_cost_upgrade { 74 } else { 82 } && upgrade_gain < weak_upgrade_threshold)
        || upgrade_gain <= worthless_upgrade_threshold
}

fn pick_rest_upgrade(dungeon: &DungeonRun) -> Option<usize> {
    let hp = hp_percent(dungeon);
    let upgrades = dungeon
        .upgradable_card_indices()
        .into_iter()
        .filter_map(|deck_index| {
            let base_card = *dungeon.deck.get(deck_index)?;
            let upgraded = upgraded_card(base_card)?;
            let gain =
                card_value(upgraded, &dungeon.deck, hp) - card_value(base_card, &dungeon.deck, hp);
            Some((base_card, gain, deck_index))
        })
        .collect::<Vec<_>>();

    upgrades
        .iter()
        .copied()
        .filter(|(card, _, _)| is_zero_cost_card(*card))
        .max_by(|(_, gain_a, index_a), (_, gain_b, index_b)| {
            gain_a.cmp(gain_b).then(index_b.cmp(index_a))
        })
        .or_else(|| {
            upgrades
                .iter()
                .copied()
                .filter(|(_, gain, _)| *gain > 0)
                .max_by(|(_, gain_a, index_a), (_, gain_b, index_b)| {
                    gain_a.cmp(gain_b).then(index_b.cmp(index_a))
                })
        })
        .map(|(_, _, deck_index)| deck_index)
}

fn pick_shop_offer(dungeon: &DungeonRun, offers: &[crate::content::ShopOffer]) -> Option<usize> {
    let (policy, choices) = scored_shop_choices(dungeon, offers);
    best_scored_choice_index(&choices, policy, SHOP_PURCHASE_THRESHOLD - 16)
}

fn pick_event_choice_score(dungeon: &DungeonRun, event: EventId) -> i32 {
    best_event_choice(dungeon, event)
        .map(|(score, _)| score)
        .unwrap_or_default()
}

fn best_event_choice(dungeon: &DungeonRun, event: EventId) -> Option<(i32, usize)> {
    let choices = collect_event_choices(event, dungeon.current_level());
    let policy = card_add_policy_for_event_choices(&choices);

    choices
        .into_iter()
        .map(|(effect, choice_index)| (score_event_choice(dungeon, effect, policy), choice_index))
        .max_by(|(score_a, index_a), (score_b, index_b)| {
            score_a.cmp(score_b).then(index_b.cmp(index_a))
        })
}

fn score_event_choice(
    dungeon: &DungeonRun,
    effect: EventChoiceEffect,
    policy: CardAddPolicy,
) -> i32 {
    let hp = hp_percent(dungeon);
    let hp_cost_weight = 12 + (100 - hp) / 4;
    match effect {
        EventChoiceEffect::GainCredits(credits) => credits as i32 / 2,
        EventChoiceEffect::LoseHpGainCredits {
            lose_hp,
            gain_credits,
        } => gain_credits as i32 / 2 - actual_hp_loss(dungeon, lose_hp) * hp_cost_weight,
        EventChoiceEffect::Heal(amount) => actual_heal(dungeon, amount) * (10 + (100 - hp) / 6),
        EventChoiceEffect::LoseHpGainMaxHp {
            lose_hp,
            gain_max_hp,
        } => gain_max_hp * 18 - actual_hp_loss(dungeon, lose_hp) * hp_cost_weight,
        EventChoiceEffect::AddCard(card) => score_event_choice_card(dungeon, card, policy, 0),
        EventChoiceEffect::LoseHpAddCard { lose_hp, card } => score_event_choice_card(
            dungeon,
            card,
            policy,
            actual_hp_loss(dungeon, lose_hp) * hp_cost_weight,
        ),
    }
}

fn score_event_choice_card(
    dungeon: &DungeonRun,
    card: CardId,
    policy: CardAddPolicy,
    hp_cost: i32,
) -> i32 {
    if actor_should_add_card(card, policy) {
        card_pick_score(dungeon, card, card_def(card).reward_tier) - hp_cost
    } else {
        -500 - hp_cost
    }
}

fn collect_action_analyses(combat: &CombatState) -> Vec<ActionAnalysis> {
    let playable_exists = (0..combat.hand_len()).any(|index| combat.can_play_card(index));
    legal_actions(combat)
        .into_iter()
        .map(|candidate| analyze_action(combat, candidate, playable_exists))
        .collect()
}

fn action_is_victory(analysis: ActionAnalysis) -> bool {
    analysis.state_score >= 1_000_000 || analysis.score >= 100_000
}

fn action_is_defensive(analysis: ActionAnalysis, uncovered_before: i32) -> bool {
    analysis.uncovered_after < uncovered_before
        || analysis.block_gain > 0
        || analysis.threat_reduction > 0
        || analysis.enemy_kills > 0
}

fn action_is_useful_zero_cost(analysis: ActionAnalysis, end_turn_score: i32) -> bool {
    analysis.zero_cost
        && (analysis.score >= end_turn_score - 12
            || analysis.damage > 0
            || analysis.block_gain > 0
            || analysis.threat_reduction > 0
            || analysis.draw_count > 0
            || analysis.created_count > 0
            || analysis.energy_gain > 0
            || analysis.enemy_kills > 0)
}

fn action_is_setup(analysis: ActionAnalysis, end_turn_score: i32) -> bool {
    (analysis.energy_gain > 0 || analysis.draw_count > 0 || analysis.created_count > 0)
        && analysis.score >= end_turn_score + 4
}

fn action_is_positive_trade(analysis: ActionAnalysis, end_turn_score: i32) -> bool {
    analysis.score >= end_turn_score + 6
        || analysis.enemy_kills > 0
        || analysis.damage > 0
        || analysis.block_gain > 0
}

fn compare_defensive_actions(left: ActionAnalysis, right: ActionAnalysis) -> std::cmp::Ordering {
    defensive_priority(left)
        .cmp(&defensive_priority(right))
        .then(compare_action_analyses(left, right))
}

fn defensive_priority(analysis: ActionAnalysis) -> (i32, i32, i32, i32, i32, i32, i32) {
    (
        i32::from(analysis.uncovered_after == 0),
        -analysis.uncovered_after,
        analysis.enemy_kills,
        analysis.threat_reduction,
        analysis.block_gain,
        analysis.energy_gain + analysis.draw_count * 2 + analysis.created_count,
        analysis.score,
    )
}

fn evaluate_combat_state(combat: &CombatState) -> i32 {
    match combat.outcome() {
        Some(CombatOutcome::Victory) => return 1_000_000 + combat.player.fighter.hp * 100,
        Some(CombatOutcome::Defeat) => return -1_000_000,
        None => {}
    }

    let snapshot = CombatSnapshot::from(combat);
    let threat = expected_enemy_threat(combat);
    let mut score = 0;

    score += snapshot.player_hp * 70;
    score += snapshot.player_block.min(threat).max(0) * 18;
    score += snapshot.player_energy as i32 * 26;
    score += combat.player.max_energy as i32 * 10;
    score += combat.hand_len() as i32 * 10;
    score += snapshot.player_focus as i32 * 17;
    score += snapshot.player_rhythm as i32 * 11;
    score += snapshot.player_momentum as i32 * 21;
    score -= snapshot.player_bleed as i32 * 26;

    score -= snapshot.enemy_total_hp * 30;
    score -= snapshot.enemy_total_block * 10;
    score -= snapshot.enemy_alive_count as i32 * 90;
    score += snapshot.enemy_bleed_sum as i32 * 18;
    score -= snapshot.enemy_focus_sum * 12;
    score -= snapshot.enemy_rhythm_sum * 6;
    score -= snapshot.enemy_momentum_sum * 10;
    score -= threat * 18;

    if snapshot.player_hp <= threat {
        score -= 400 + (threat - snapshot.player_hp).max(0) * 60;
    }

    score
}

fn legal_actions(combat: &CombatState) -> Vec<ActionCandidate> {
    let mut actions = Vec::new();
    for hand_index in 0..combat.hand_len() {
        if !combat.can_play_card(hand_index) {
            continue;
        }
        if combat.card_targets_all_enemies(hand_index) {
            actions.push(ActionCandidate {
                action: CombatAction::PlayCard {
                    hand_index,
                    target: None,
                },
                hand_index: Some(hand_index),
                enemy_index: None,
            });
        } else if combat.card_requires_enemy(hand_index) {
            for enemy_index in 0..combat.enemy_count() {
                if !combat.enemy_is_alive(enemy_index) {
                    continue;
                }
                actions.push(ActionCandidate {
                    action: CombatAction::PlayCard {
                        hand_index,
                        target: Some(Actor::Enemy(enemy_index)),
                    },
                    hand_index: Some(hand_index),
                    enemy_index: Some(enemy_index),
                });
            }
        } else {
            actions.push(ActionCandidate {
                action: CombatAction::PlayCard {
                    hand_index,
                    target: Some(Actor::Player),
                },
                hand_index: Some(hand_index),
                enemy_index: None,
            });
        }
    }

    actions.push(ActionCandidate {
        action: CombatAction::EndTurn,
        hand_index: None,
        enemy_index: None,
    });
    actions
}

fn analyze_action(
    combat: &CombatState,
    candidate: ActionCandidate,
    playable_exists: bool,
) -> ActionAnalysis {
    let mut simulated = combat.clone();
    let threat_before = expected_enemy_threat(combat);
    let before = CombatSnapshot::from(combat);
    let events = simulated.dispatch(candidate.action);
    let after = CombatSnapshot::from(&simulated);
    let threat_after = expected_enemy_threat(&simulated);
    let mut score = 0;
    let mut draw_count = 0;
    let mut created_count = 0;
    let cost = candidate
        .hand_index
        .and_then(|hand_index| combat.hand_card(hand_index))
        .map(|card| card_def(card).cost)
        .unwrap_or(u8::MAX);
    let zero_cost = cost == 0;

    for event in &events {
        score += match event {
            CombatEvent::CombatWon => 100_000,
            CombatEvent::CombatLost => -100_000,
            CombatEvent::ActorDefeated {
                actor: Actor::Enemy(_),
            } => 120,
            CombatEvent::ActorDefeated {
                actor: Actor::Player,
            } => -1_000,
            CombatEvent::DamageDealt {
                source: Actor::Player,
                target: Actor::Enemy(_),
                amount,
            } => amount * 6,
            CombatEvent::DamageDealt {
                target: Actor::Player,
                amount,
                ..
            } => -(amount * 10),
            CombatEvent::CardCreated { .. } => {
                created_count += 1;
                10
            }
            CombatEvent::CardDrawn { .. } if !matches!(candidate.action, CombatAction::EndTurn) => {
                draw_count += 1;
                6
            }
            CombatEvent::CardBurned { .. } => -6,
            CombatEvent::BlockGained {
                actor: Actor::Player,
                amount,
            } => (*amount).min(threat_before.max(1)).max(0) * 5,
            CombatEvent::InvalidAction { .. }
            | CombatEvent::NotEnoughEnergy { .. }
            | CombatEvent::RequirementNotMet { .. } => -1_000,
            _ => 0,
        };
    }

    let damage = before.enemy_total_hp - after.enemy_total_hp;
    let block_gain = (after.player_block - before.player_block).max(0);
    let threat_reduction = (threat_before - threat_after).max(0);
    let enemy_kills = before.enemy_alive_count as i32 - after.enemy_alive_count as i32;
    let energy_gain = (after.player_energy as i32 - before.player_energy as i32).max(0);

    score += damage * 9;
    score += (before.enemy_total_block - after.enemy_total_block) * 4;
    score += enemy_kills * 48;
    score -= (before.player_hp - after.player_hp) * 15;
    score += (after.player_focus as i32 - before.player_focus as i32) * 6;
    score += (after.player_rhythm as i32 - before.player_rhythm as i32) * 4;
    score += (after.player_momentum as i32 - before.player_momentum as i32) * 12;
    score += (before.enemy_focus_sum - after.enemy_focus_sum) * 5;
    score += (before.enemy_rhythm_sum - after.enemy_rhythm_sum) * 3;
    score += (before.enemy_momentum_sum - after.enemy_momentum_sum) * 6;
    score += (after.enemy_bleed_sum as i32 - before.enemy_bleed_sum as i32) * 7;
    score -= (after.player_bleed as i32 - before.player_bleed as i32) * 10;

    let uncovered_before = (threat_before - before.player_block).max(0);
    let uncovered_after = (threat_after - after.player_block).max(0);
    if uncovered_before > 0 {
        score += (uncovered_before - uncovered_after) * 32;
        if uncovered_after == 0 {
            score += 24;
        }
        if uncovered_after >= uncovered_before && !matches!(candidate.action, CombatAction::EndTurn)
        {
            score -= 52 + uncovered_before * 4;
        }
    } else if threat_before == 0 || before.player_block >= threat_before {
        score += damage * 3;
    }

    if !matches!(candidate.action, CombatAction::EndTurn) {
        let before_covered = before.player_block.min(threat_before);
        let after_covered = after.player_block.min(threat_after.max(threat_before));
        if after_covered > before_covered {
            score += (after_covered - before_covered) * 8;
        }
        score += energy_gain * 18;
        score += draw_count * 10;
        score += created_count * 8;
        if zero_cost {
            score += 18;
            if damage > 0
                || block_gain > 0
                || draw_count > 0
                || created_count > 0
                || energy_gain > 0
                || threat_reduction > 0
            {
                score += 10;
            }
        }
        if after.player_hp <= threat_after {
            score -= 180;
        }
    } else if playable_exists {
        score -= 24;
        let zero_cost_playable = playable_zero_cost_count(combat) as i32;
        if zero_cost_playable > 0 {
            score -= 80 + zero_cost_playable * 10;
        }
    }

    let state_score = evaluate_combat_state(&simulated);

    ActionAnalysis {
        candidate,
        score,
        state_score,
        damage,
        block_gain,
        threat_reduction,
        draw_count,
        created_count,
        energy_gain,
        enemy_kills,
        uncovered_after,
        zero_cost,
    }
}

fn playable_zero_cost_count(combat: &CombatState) -> usize {
    (0..combat.hand_len())
        .filter(|&index| {
            combat.can_play_card(index)
                && combat
                    .hand_card(index)
                    .map(|card| card_def(card).cost == 0)
                    .unwrap_or(false)
        })
        .count()
}

pub(crate) fn expected_enemy_threat(combat: &CombatState) -> i32 {
    combat.expected_enemy_threat()
}

fn is_zero_cost_card(card: CardId) -> bool {
    card_def(card).cost == 0
}

fn is_defensive_card(card: CardId) -> bool {
    card_block_value(card) > 0 || matches!(card_def(card).target, CardTarget::SelfOnly)
}

fn collect_event_choices(event: EventId, level: usize) -> Vec<(EventChoiceEffect, usize)> {
    (0..4)
        .filter_map(|choice_index| {
            let effect = event_choice_effect(event, choice_index, level)?;
            Some((effect, choice_index))
        })
        .collect()
}

fn scored_shop_choices(
    dungeon: &DungeonRun,
    offers: &[crate::content::ShopOffer],
) -> (CardAddPolicy, Vec<ScoredChoice<usize>>) {
    let policy = card_add_policy_for_cards(
        offers
            .iter()
            .filter(|offer| dungeon.can_afford_shop_price(offer.price))
            .map(|offer| offer.card),
    );
    let choices = offers
        .iter()
        .enumerate()
        .filter(|(_, offer)| {
            dungeon.can_afford_shop_price(offer.price) && actor_should_add_card(offer.card, policy)
        })
        .map(|(index, offer)| ScoredChoice {
            score: card_pick_score(dungeon, offer.card, card_def(offer.card).reward_tier),
            key: index,
            value: index,
            zero_cost: is_zero_cost_card(offer.card),
        })
        .collect();
    (policy, choices)
}

fn scored_reward_choices(
    dungeon: &DungeonRun,
    options: &[CardId],
    tier: RewardTier,
) -> (CardAddPolicy, Vec<ScoredChoice<usize>>) {
    let policy = card_add_policy_for_cards(options.iter().copied());
    let choices = options
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, card)| actor_should_add_card(*card, policy))
        .map(|(index, card)| ScoredChoice {
            score: card_pick_score(dungeon, card, Some(tier)),
            key: card as usize,
            value: index,
            zero_cost: is_zero_cost_card(card),
        })
        .collect();
    (policy, choices)
}

fn best_scored_choice_index<T: Copy>(
    choices: &[ScoredChoice<T>],
    policy: CardAddPolicy,
    zero_cost_threshold: i32,
) -> Option<T> {
    match policy {
        CardAddPolicy::ZeroCostOnly => choices
            .iter()
            .copied()
            .filter(|choice| choice.zero_cost && choice.score >= zero_cost_threshold)
            .max_by(compare_scored_choice)
            .map(|choice| choice.value),
        CardAddPolicy::DefensiveFallback => choices
            .iter()
            .copied()
            .filter(|choice| choice.score >= zero_cost_threshold)
            .max_by(compare_scored_choice)
            .map(|choice| choice.value),
        CardAddPolicy::Skip => None,
    }
}

fn compare_scored_choice<T>(left: &ScoredChoice<T>, right: &ScoredChoice<T>) -> std::cmp::Ordering {
    left.score.cmp(&right.score).then(right.key.cmp(&left.key))
}

fn card_add_policy_for_event_choices(choices: &[(EventChoiceEffect, usize)]) -> CardAddPolicy {
    card_add_policy_for_cards(
        choices
            .iter()
            .filter_map(|(effect, _)| event_choice_card(*effect)),
    )
}

fn card_add_policy_for_cards<I>(cards: I) -> CardAddPolicy
where
    I: IntoIterator<Item = CardId>,
{
    let mut has_defensive = false;
    for card in cards {
        if is_zero_cost_card(card) {
            return CardAddPolicy::ZeroCostOnly;
        }
        has_defensive |= is_defensive_card(card);
    }
    if has_defensive {
        CardAddPolicy::DefensiveFallback
    } else {
        CardAddPolicy::Skip
    }
}

fn event_choice_card(effect: EventChoiceEffect) -> Option<CardId> {
    match effect {
        EventChoiceEffect::AddCard(card) | EventChoiceEffect::LoseHpAddCard { card, .. } => {
            Some(card)
        }
        _ => None,
    }
}

fn actor_should_add_card(card: CardId, policy: CardAddPolicy) -> bool {
    match policy {
        CardAddPolicy::ZeroCostOnly => is_zero_cost_card(card),
        CardAddPolicy::DefensiveFallback => is_defensive_card(card),
        CardAddPolicy::Skip => false,
    }
}

fn card_draw_value(card: CardId) -> i32 {
    match card {
        CardId::Slipstream
        | CardId::SlipstreamPlus
        | CardId::QuickStrike
        | CardId::QuickStrikePlus
        | CardId::SignalTap
        | CardId::SignalTapPlus
        | CardId::CoverPulse
        | CardId::CoverPulsePlus
        | CardId::BreachSignal
        | CardId::BreachSignalPlus
        | CardId::RiftDart
        | CardId::RiftDartPlus
        | CardId::PulseConverterPlus
        | CardId::FortressMatrix
        | CardId::FortressMatrixPlus
        | CardId::StormVaultPlus
        | CardId::SparkSmith
        | CardId::AssemblyLine
        | CardId::ToolCache
        | CardId::ImprovisedArsenal
        | CardId::ForgeStorm
        | CardId::HardReset
        | CardId::EmberBurst
        | CardId::AshenVector
        | CardId::LastProtocol => 1,
        CardId::TacticalBurst
        | CardId::TacticalBurstPlus
        | CardId::AnchorLoop
        | CardId::AnchorLoopPlus
        | CardId::OverwatchGrid
        | CardId::OverwatchGridPlus
        | CardId::PrimeRoutine
        | CardId::PrimeRoutinePlus
        | CardId::SparkSmithPlus
        | CardId::AssemblyLinePlus
        | CardId::ToolCachePlus
        | CardId::ImprovisedArsenalPlus
        | CardId::ForgeStormPlus
        | CardId::HardResetPlus
        | CardId::EmberBurstPlus
        | CardId::AshenVectorPlus
        | CardId::LastProtocolPlus => 2,
        _ => 0,
    }
}

fn card_disruption_value(card: CardId) -> i32 {
    match card {
        CardId::SignalTap
        | CardId::SignalTapPlus
        | CardId::PressurePoint
        | CardId::PressurePointPlus
        | CardId::BarrierField
        | CardId::BarrierFieldPlus
        | CardId::VectorLock
        | CardId::VectorLockPlus
        | CardId::BreachSignal
        | CardId::BreachSignalPlus
        | CardId::MarkPulse
        | CardId::MarkPulsePlus
        | CardId::PulseConverter
        | CardId::PulseConverterPlus
        | CardId::CollapsePattern
        | CardId::CollapsePatternPlus
        | CardId::SuppressionNet
        | CardId::SuppressionNetPlus
        | CardId::Cauterize
        | CardId::CauterizePlus
        | CardId::Tracer => 1,
        CardId::ZeroPoint
        | CardId::ZeroPointPlus
        | CardId::DimmingWave
        | CardId::DimmingWavePlus
        | CardId::Lockbreaker
        | CardId::LockbreakerPlus => 2,
        _ => 0,
    }
}

fn card_momentum_value(card: CardId) -> i32 {
    match card {
        CardId::ArcSpark | CardId::CapacitiveShell | CardId::PrimeRoutine => 2,
        CardId::ArcSparkPlus | CardId::CapacitiveShellPlus | CardId::PrimeRoutinePlus => 3,
        CardId::Stockpile => 3,
        CardId::StockpilePlus => 4,
        _ => 0,
    }
}

fn card_rhythm_source_value(card: CardId) -> i32 {
    match card {
        CardId::Slipstream | CardId::SlipstreamPlus | CardId::Reinforce => 1,
        CardId::ReinforcePlus => 2,
        _ => 0,
    }
}

fn card_generation_value(card: CardId) -> i32 {
    match card {
        CardId::SparkSmith
        | CardId::SparkSmithPlus
        | CardId::PatchBay
        | CardId::PatchBayPlus
        | CardId::TracerWeave
        | CardId::TracerWeavePlus
        | CardId::NeedleNest
        | CardId::NeedleNestPlus => 1,
        CardId::AssemblyLine
        | CardId::AssemblyLinePlus
        | CardId::ToolCache
        | CardId::ToolCachePlus => 2,
        CardId::ImprovisedArsenal
        | CardId::ImprovisedArsenalPlus
        | CardId::ForgeStorm
        | CardId::ForgeStormPlus => 4,
        _ => 0,
    }
}

fn card_burst_value(card: CardId) -> i32 {
    match card {
        CardId::RazorRush | CardId::RazorRushPlus => 10,
        CardId::HardReset | CardId::HardResetPlus => 8,
        CardId::EmergencyPlating | CardId::EmergencyPlatingPlus => 10,
        CardId::EmberBurst | CardId::EmberBurstPlus => 9,
        CardId::FracturePulse | CardId::FracturePulsePlus => 9,
        CardId::VectorLock | CardId::VectorLockPlus => 10,
        CardId::ZeroPoint | CardId::ZeroPointPlus => 11,
        CardId::PrimeRoutine | CardId::PrimeRoutinePlus => 10,
        CardId::ReservoirGuard | CardId::ReservoirGuardPlus => 9,
        CardId::VoltaicDrive | CardId::VoltaicDrivePlus => 10,
        CardId::ChainBarrage | CardId::ChainBarragePlus => 12,
        CardId::OverwatchGrid | CardId::OverwatchGridPlus => 10,
        CardId::SuppressionNet | CardId::SuppressionNetPlus => 11,
        CardId::LastProtocol | CardId::LastProtocolPlus => 12,
        CardId::PurgeArray | CardId::PurgeArrayPlus => 12,
        CardId::NovaCollapse | CardId::NovaCollapsePlus => 12,
        _ => 0,
    }
}

fn card_block_value(card: CardId) -> i32 {
    match card {
        CardId::GuardStep => 8,
        CardId::GuardStepPlus => 12,
        CardId::Slipstream => 3,
        CardId::SlipstreamPlus => 6,
        CardId::SignalTapPlus => 5,
        CardId::Reinforce => 13,
        CardId::ReinforcePlus => 18,
        CardId::CoverPulse => 10,
        CardId::CoverPulsePlus => 13,
        CardId::BarrierField => 16,
        CardId::BarrierFieldPlus => 21,
        CardId::VectorLock => 8,
        CardId::VectorLockPlus => 10,
        CardId::AnchorLoop => 22,
        CardId::AnchorLoopPlus => 27,
        CardId::FortressMatrix => 26,
        CardId::FortressMatrixPlus => 32,
        CardId::OverwatchGrid => 29,
        CardId::OverwatchGridPlus => 35,
        CardId::MarkPulse => 6,
        CardId::MarkPulsePlus => 10,
        CardId::BraceCircuit => 10,
        CardId::BraceCircuitPlus => 13,
        CardId::Lockbreaker => 10,
        CardId::LockbreakerPlus => 13,
        CardId::CapacitiveShell => 8,
        CardId::CapacitiveShellPlus => 13,
        CardId::ReservoirGuard => 16,
        CardId::ReservoirGuardPlus => 21,
        CardId::PatchBay => 10,
        CardId::PatchBayPlus => 13,
        CardId::ShrapnelVeil => 6,
        CardId::ShrapnelVeilPlus => 10,
        CardId::SuppressionNet => 13,
        CardId::SuppressionNetPlus => 18,
        CardId::EmergencyPlating => 19,
        CardId::EmergencyPlatingPlus => 26,
        CardId::Patch => 8,
        _ => 0,
    }
}

fn card_value(card: CardId, deck: &[CardId], hp_percent: i32) -> i32 {
    let def = card_def(card);
    let duplicates = deck.iter().filter(|&&owned| owned == card).count() as i32;
    let same_archetype = deck
        .iter()
        .filter(|&&owned| card_def(owned).archetype == def.archetype)
        .count();
    let zero_cost_owned = deck
        .iter()
        .filter(|&&owned| is_zero_cost_card(owned))
        .count() as i32;
    let draw_value = card_draw_value(card);
    let disruption_value = card_disruption_value(card);
    let momentum_value = card_momentum_value(card);
    let rhythm_sources = deck
        .iter()
        .filter(|&&owned| card_rhythm_source_value(owned) > 0)
        .count() as i32;
    let generation_value = card_generation_value(card);
    let burst_value = card_burst_value(card);
    let block_value = card_block_value(card);
    let zero_cost = is_zero_cost_card(card);

    let mut value = 6;
    value += match def.cost {
        0 => 16,
        1 => 6,
        2 => 3,
        3 => 0,
        4 => -4,
        _ => -8,
    };
    value += match def.target {
        CardTarget::AllEnemies => 10,
        CardTarget::SelfOnly => {
            if hp_percent < 70 {
                7
            } else {
                3
            }
        }
        CardTarget::Enemy => 4,
    };
    value += match def.reward_tier {
        Some(RewardTier::Combat) => 0,
        Some(RewardTier::Elite) => 6,
        Some(RewardTier::Boss) => 12,
        None => 0,
    };
    value += match def.archetype {
        CardArchetype::Bulwark => {
            if hp_percent < 70 {
                8
            } else {
                3
            }
        }
        CardArchetype::Pressure => {
            if hp_percent > 45 {
                6
            } else {
                2
            }
        }
        CardArchetype::Sweep => 7,
        CardArchetype::Burst => 6,
        CardArchetype::Momentum => 5,
        CardArchetype::Fabricate => 2,
    };
    value += match same_archetype {
        0 => 5,
        1..=2 => 2,
        3..=4 => -1,
        5..=7 => -4,
        _ => -8,
    };
    if def.traits.piercing {
        value += 5;
    }
    if def.traits.temporary {
        value -= 14;
    }
    if zero_cost {
        value += 16;
        value += (10 - zero_cost_owned).max(0);
        if upgraded_card(card).is_some() {
            value += 8;
        }
    }
    value += draw_value * 5;
    value += disruption_value * 6;
    value += momentum_value * 7;
    value += generation_value * 3;
    value += burst_value;
    if block_value > 0 && hp_percent < 65 {
        value += block_value / 2;
    }
    if matches!(def.target, CardTarget::AllEnemies) {
        value += 6;
    }
    if zero_cost
        && (draw_value > 0 || disruption_value > 0 || momentum_value > 0 || burst_value > 0)
    {
        value += 18;
    }
    if matches!(card, CardId::PulseConverter | CardId::PulseConverterPlus) {
        value += if rhythm_sources >= 2 { 10 } else { -12 };
    }
    if matches!(
        card,
        CardId::SignalTap
            | CardId::MarkPulse
            | CardId::PressurePoint
            | CardId::SparkSmith
            | CardId::NeedleNest
            | CardId::TracerWeave
    ) {
        value -= 4;
    }
    value -= duplicates * if zero_cost { 2 } else { 6 };

    value
}

fn card_pick_score(dungeon: &DungeonRun, card: CardId, tier: Option<RewardTier>) -> i32 {
    let hp = hp_percent(dungeon);
    let raw = card_value(card, &dungeon.deck, hp);
    let target = target_deck_size(dungeon, tier);
    let bloat_penalty =
        (dungeon.deck.len() as i32 - target).max(0) * if is_zero_cost_card(card) { 2 } else { 5 };
    raw - bloat_penalty + if is_zero_cost_card(card) { 14 } else { 0 }
}

fn target_deck_size(dungeon: &DungeonRun, tier: Option<RewardTier>) -> i32 {
    let base = match dungeon.current_level() {
        1 => 13,
        2 => 16,
        _ => 19,
    };
    match tier {
        Some(RewardTier::Boss) => base + 2,
        Some(RewardTier::Elite) => base + 1,
        _ => base,
    }
}

fn reward_pick_threshold(dungeon: &DungeonRun, tier: RewardTier) -> i32 {
    let base = match tier {
        RewardTier::Combat => 24,
        RewardTier::Elite => 20,
        RewardTier::Boss => 16,
    };
    base + (dungeon.deck.len() as i32 - target_deck_size(dungeon, Some(tier))).max(0) * 3
}

fn deck_power_score(dungeon: &DungeonRun) -> i32 {
    if dungeon.deck.is_empty() {
        return 0;
    }
    let hp = hp_percent(dungeon);
    dungeon
        .deck
        .iter()
        .copied()
        .map(|card| card_value(card, &dungeon.deck, hp))
        .sum::<i32>()
        / dungeon.deck.len() as i32
}

fn compare_action_candidates(left: ActionCandidate, right: ActionCandidate) -> std::cmp::Ordering {
    left.hand_index
        .unwrap_or(usize::MAX)
        .cmp(&right.hand_index.unwrap_or(usize::MAX))
        .then(
            left.enemy_index
                .unwrap_or(usize::MAX)
                .cmp(&right.enemy_index.unwrap_or(usize::MAX)),
        )
}

fn compare_action_analyses(left: ActionAnalysis, right: ActionAnalysis) -> std::cmp::Ordering {
    left.score
        .cmp(&right.score)
        .then(left.state_score.cmp(&right.state_score))
        .then(compare_action_candidates(right.candidate, left.candidate))
}

fn actual_heal(dungeon: &DungeonRun, amount: i32) -> i32 {
    (dungeon.player_max_hp - dungeon.player_hp).clamp(0, amount)
}

fn actual_hp_loss(dungeon: &DungeonRun, amount: i32) -> i32 {
    (dungeon.player_hp - 1).max(0).min(amount)
}

fn hp_percent(dungeon: &DungeonRun) -> i32 {
    if dungeon.player_max_hp <= 0 {
        0
    } else {
        (dungeon.player_hp.max(0) * 100) / dungeon.player_max_hp
    }
}

impl From<&CombatState> for CombatSnapshot {
    fn from(combat: &CombatState) -> Self {
        let mut enemy_total_hp = 0;
        let mut enemy_total_block = 0;
        let mut enemy_alive_count = 0usize;
        let mut enemy_bleed_sum = 0u8;
        let mut enemy_focus_sum = 0i32;
        let mut enemy_rhythm_sum = 0i32;
        let mut enemy_momentum_sum = 0i32;

        for enemy in &combat.enemies {
            enemy_total_hp += enemy.fighter.hp.max(0);
            enemy_total_block += enemy.fighter.block.max(0);
            if enemy.fighter.hp > 0 {
                enemy_alive_count += 1;
            }
            enemy_bleed_sum = enemy_bleed_sum.saturating_add(enemy.fighter.statuses.bleed);
            enemy_focus_sum += enemy.fighter.statuses.focus as i32;
            enemy_rhythm_sum += enemy.fighter.statuses.rhythm as i32;
            enemy_momentum_sum += enemy.fighter.statuses.momentum as i32;
        }

        Self {
            player_hp: combat.player.fighter.hp,
            player_block: combat.player.fighter.block,
            player_energy: combat.player.energy,
            player_bleed: combat.player.fighter.statuses.bleed,
            player_focus: combat.player.fighter.statuses.focus,
            player_rhythm: combat.player.fighter.statuses.rhythm,
            player_momentum: combat.player.fighter.statuses.momentum,
            enemy_total_hp,
            enemy_total_block,
            enemy_alive_count,
            enemy_bleed_sum,
            enemy_focus_sum,
            enemy_rhythm_sum,
            enemy_momentum_sum,
        }
    }
}

pub(crate) fn choose_boss_module_reward(
    dungeon: &DungeonRun,
    boss_level: usize,
) -> Option<ModuleId> {
    let options = boss_module_choices(boss_level)
        .into_iter()
        .filter(|module| !dungeon.has_module(*module))
        .collect::<Vec<_>>();
    choose_best_module(dungeon, &options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn card_block_values_include_shield_granting_cards() {
        let cases = [
            (CardId::SignalTapPlus, 5),
            (CardId::VectorLock, 8),
            (CardId::VectorLockPlus, 10),
            (CardId::FortressMatrix, 26),
            (CardId::FortressMatrixPlus, 32),
            (CardId::MarkPulse, 6),
            (CardId::MarkPulsePlus, 10),
            (CardId::BraceCircuit, 10),
            (CardId::BraceCircuitPlus, 13),
            (CardId::Lockbreaker, 10),
            (CardId::LockbreakerPlus, 13),
            (CardId::ShrapnelVeil, 6),
            (CardId::ShrapnelVeilPlus, 10),
            (CardId::SuppressionNet, 13),
            (CardId::SuppressionNetPlus, 18),
        ];

        for (card, block) in cases {
            assert_eq!(card_block_value(card), block, "{card:?}");
            assert!(is_defensive_card(card), "{card:?}");
        }
    }

    #[test]
    fn defensive_fallback_respects_score_threshold() {
        let choices = [
            ScoredChoice {
                score: 13,
                key: 0,
                value: 0usize,
                zero_cost: false,
            },
            ScoredChoice {
                score: 9,
                key: 1,
                value: 1usize,
                zero_cost: false,
            },
        ];

        assert_eq!(
            best_scored_choice_index(&choices, CardAddPolicy::DefensiveFallback, 14),
            None
        );
        assert_eq!(
            best_scored_choice_index(&choices, CardAddPolicy::DefensiveFallback, 13),
            Some(0)
        );
    }
}
