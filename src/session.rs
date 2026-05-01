use serde::{Deserialize, Serialize};

pub(crate) const MAX_PARTY_SIZE: usize = 7;
const DEFAULT_PARTY_SIZE: usize = 1;
const DEFAULT_LOCAL_NAME: &str = "Player";
const DEFAULT_REMOTE_NAME: &str = "Guest";

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SlotClaimKind {
    Open,
    Local,
    Remote,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SlotLifeState {
    Alive,
    Dead,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct PartySlotSnapshot {
    pub(crate) slot: usize,
    pub(crate) name: String,
    pub(crate) claim: SlotClaimKind,
    pub(crate) connected: bool,
    pub(crate) life: SlotLifeState,
    pub(crate) ready: bool,
    pub(crate) in_run: bool,
    pub(crate) in_combat: bool,
    pub(crate) hp: i32,
    pub(crate) max_hp: i32,
    pub(crate) block: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct HeroRuntimeSummary {
    pub(crate) in_run: bool,
    pub(crate) in_combat: bool,
    pub(crate) hp: i32,
    pub(crate) max_hp: i32,
    pub(crate) block: i32,
    pub(crate) alive: bool,
}

impl Default for HeroRuntimeSummary {
    fn default() -> Self {
        Self {
            in_run: false,
            in_combat: false,
            hp: 0,
            max_hp: 0,
            block: 0,
            alive: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct PartySessionSnapshot {
    pub(crate) configured_party_size: usize,
    pub(crate) captain_slot: usize,
    pub(crate) local_slot: usize,
    pub(crate) screen: String,
    pub(crate) in_run: bool,
    pub(crate) slots: Vec<PartySlotSnapshot>,
}

impl Default for PartySessionSnapshot {
    fn default() -> Self {
        Self::new(DEFAULT_PARTY_SIZE)
    }
}

impl PartySessionSnapshot {
    pub(crate) fn new(configured_party_size: usize) -> Self {
        let mut slots = (0..MAX_PARTY_SIZE).map(blank_slot).collect::<Vec<_>>();
        slots[0].claim = SlotClaimKind::Local;
        slots[0].connected = true;
        slots[0].name = DEFAULT_LOCAL_NAME.to_string();
        Self {
            configured_party_size: configured_party_size.clamp(1, MAX_PARTY_SIZE),
            captain_slot: 0,
            local_slot: 0,
            screen: "boot".to_string(),
            in_run: false,
            slots,
        }
    }

    pub(crate) fn normalize(mut self) -> Self {
        self.configured_party_size = self.configured_party_size.clamp(1, MAX_PARTY_SIZE);
        self.captain_slot = self.captain_slot.min(self.configured_party_size - 1);
        self.local_slot = self.local_slot.min(self.configured_party_size - 1);
        if self.slots.len() < MAX_PARTY_SIZE {
            self.slots
                .extend((self.slots.len()..MAX_PARTY_SIZE).map(blank_slot));
        } else if self.slots.len() > MAX_PARTY_SIZE {
            self.slots.truncate(MAX_PARTY_SIZE);
        }
        for (index, slot) in self.slots.iter_mut().enumerate() {
            slot.slot = index;
            if index != self.local_slot && slot.claim == SlotClaimKind::Local {
                slot.claim = SlotClaimKind::Open;
                slot.connected = false;
            }
        }
        for index in self.configured_party_size..MAX_PARTY_SIZE {
            self.slots[index] = blank_slot(index);
        }
        let local_slot = &mut self.slots[self.local_slot];
        local_slot.claim = SlotClaimKind::Local;
        local_slot.connected = true;
        if local_slot.name.trim().is_empty() {
            local_slot.name = DEFAULT_LOCAL_NAME.to_string();
        }
        if self.screen.is_empty() {
            self.screen = "boot".to_string();
        }
        self
    }

    pub(crate) fn set_configured_party_size(&mut self, size: usize) {
        self.configured_party_size = size.clamp(1, MAX_PARTY_SIZE);
        self.captain_slot = self.captain_slot.min(self.configured_party_size - 1);
        self.local_slot = self.local_slot.min(self.configured_party_size - 1);
        for index in self.configured_party_size..MAX_PARTY_SIZE {
            self.slots[index] = blank_slot(index);
        }
        for (index, slot) in self
            .slots
            .iter_mut()
            .enumerate()
            .take(self.configured_party_size)
        {
            if index != self.local_slot && slot.claim == SlotClaimKind::Local {
                slot.claim = SlotClaimKind::Open;
                slot.connected = false;
            }
        }
        let local_slot = &mut self.slots[self.local_slot];
        local_slot.claim = SlotClaimKind::Local;
        local_slot.connected = true;
        if local_slot.name.trim().is_empty() {
            local_slot.name = DEFAULT_LOCAL_NAME.to_string();
        }
    }

    pub(crate) fn set_local_display_name(&mut self, name: &str) {
        self.slots[self.local_slot].name = if name.trim().is_empty() {
            DEFAULT_LOCAL_NAME.to_string()
        } else {
            name.to_string()
        };
    }

    #[cfg(test)]
    pub(crate) fn slot(&self, slot: usize) -> Option<&PartySlotSnapshot> {
        self.slots
            .get(slot)
            .filter(|_| slot < self.configured_party_size)
    }

    pub(crate) fn sync_remote_slot(
        &mut self,
        slot: usize,
        name: Option<&str>,
        connected: bool,
        ready: bool,
        runtime: HeroRuntimeSummary,
    ) -> bool {
        if slot >= self.configured_party_size || slot == self.local_slot {
            return false;
        }
        let slot_state = &mut self.slots[slot];
        let before = slot_state.clone();
        slot_state.claim = SlotClaimKind::Remote;
        slot_state.connected = connected;
        slot_state.ready = ready;
        if let Some(name) = name.map(str::trim).filter(|name| !name.is_empty()) {
            slot_state.name = name.to_string();
        } else if slot_state.name.trim().is_empty() {
            slot_state.name = format!("{DEFAULT_REMOTE_NAME} {}", slot + 1);
        }
        slot_state.in_run = runtime.in_run;
        slot_state.in_combat = runtime.in_combat;
        slot_state.hp = runtime.hp;
        slot_state.max_hp = runtime.max_hp;
        slot_state.block = runtime.block;
        slot_state.life = if runtime.alive {
            SlotLifeState::Alive
        } else {
            SlotLifeState::Dead
        };
        *slot_state != before
    }

    pub(crate) fn disconnect_remote_slot(&mut self, slot: usize) -> bool {
        let Some(slot_state) = self.slots.get_mut(slot) else {
            return false;
        };
        if slot >= self.configured_party_size || slot == self.local_slot {
            return false;
        }
        if slot_state.claim != SlotClaimKind::Remote {
            return false;
        }
        let before = slot_state.clone();
        slot_state.connected = false;
        slot_state.ready = false;
        *slot_state != before
    }

    pub(crate) fn clear_remote_slot(&mut self, slot: usize) -> bool {
        if slot >= self.configured_party_size || slot == self.local_slot {
            return false;
        }
        let cleared = blank_slot(slot);
        if self.slots[slot] == cleared {
            return false;
        }
        self.slots[slot] = cleared;
        true
    }

    pub(crate) fn update_local_runtime(&mut self, screen: &str, runtime: HeroRuntimeSummary) {
        self.screen.clear();
        self.screen.push_str(screen);
        self.in_run = runtime.in_run;
        let slot = &mut self.slots[self.local_slot];
        slot.in_run = runtime.in_run;
        slot.in_combat = runtime.in_combat;
        slot.hp = runtime.hp;
        slot.max_hp = runtime.max_hp;
        slot.block = runtime.block;
        slot.life = if runtime.alive {
            SlotLifeState::Alive
        } else {
            SlotLifeState::Dead
        };
    }
}

fn blank_slot(slot: usize) -> PartySlotSnapshot {
    PartySlotSnapshot {
        slot,
        name: String::new(),
        claim: SlotClaimKind::Open,
        connected: false,
        life: SlotLifeState::Alive,
        ready: false,
        in_run: false,
        in_combat: false,
        hp: 0,
        max_hp: 0,
        block: 0,
    }
}

pub(crate) fn serialize_party_session(snapshot: &PartySessionSnapshot) -> Result<String, String> {
    serde_json::to_string(snapshot).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_session_has_one_local_slot() {
        let session = PartySessionSnapshot::default();
        assert_eq!(session.configured_party_size, 1);
        assert_eq!(session.local_slot, 0);
        assert_eq!(session.captain_slot, 0);
        assert_eq!(session.slots.len(), MAX_PARTY_SIZE);
        assert_eq!(session.slots[0].claim, SlotClaimKind::Local);
        assert!(session.slots[0].connected);
    }

    #[test]
    fn configured_party_size_is_clamped_and_truncates_extra_slots() {
        let mut session = PartySessionSnapshot::new(4);
        session.slots[3].claim = SlotClaimKind::Remote;
        session.slots[3].connected = true;
        session.slots[3].name = "Mia".to_string();
        session.set_configured_party_size(MAX_PARTY_SIZE + 5);
        assert_eq!(session.configured_party_size, MAX_PARTY_SIZE);
        session.set_configured_party_size(1);
        assert_eq!(session.configured_party_size, 1);
        assert_eq!(session.slots[3].claim, SlotClaimKind::Open);
        assert!(session.slots[3].name.is_empty());
    }

    #[test]
    fn shrinking_party_size_preserves_a_connected_local_slot() {
        let mut session = PartySessionSnapshot::new(4);
        session.slots[0] = blank_slot(0);
        session.local_slot = 3;
        session.slots[3].claim = SlotClaimKind::Local;
        session.slots[3].connected = true;
        session.slots[3].name = "Captain".to_string();

        session.set_configured_party_size(1);

        assert_eq!(session.local_slot, 0);
        assert_eq!(session.slots[0].claim, SlotClaimKind::Local);
        assert!(session.slots[0].connected);
        assert_eq!(session.slots[0].name, DEFAULT_LOCAL_NAME);
    }

    #[test]
    fn local_runtime_overlay_updates_local_slot_only() {
        let mut session = PartySessionSnapshot::new(3);
        session.slots[1].claim = SlotClaimKind::Remote;
        session.slots[1].connected = true;
        session.slots[1].name = "Ava".to_string();
        session.update_local_runtime(
            "combat",
            HeroRuntimeSummary {
                in_run: true,
                in_combat: true,
                hp: 27,
                max_hp: 32,
                block: 5,
                alive: true,
            },
        );
        assert_eq!(session.screen, "combat");
        assert_eq!(session.slots[0].hp, 27);
        assert_eq!(session.slots[0].block, 5);
        assert_eq!(session.slots[1].name, "Ava");
    }

    #[test]
    fn sync_remote_slot_claims_and_updates_runtime() {
        let mut session = PartySessionSnapshot::new(3);

        assert!(session.sync_remote_slot(
            1,
            Some("Mia"),
            true,
            true,
            HeroRuntimeSummary {
                in_run: true,
                in_combat: true,
                hp: 21,
                max_hp: 34,
                block: 6,
                alive: true,
            },
        ));

        let slot = session.slot(1).unwrap();
        assert_eq!(slot.claim, SlotClaimKind::Remote);
        assert_eq!(slot.name, "Mia");
        assert!(slot.connected);
        assert!(slot.ready);
        assert_eq!(slot.hp, 21);
        assert_eq!(slot.max_hp, 34);
        assert_eq!(slot.block, 6);
    }

    #[test]
    fn disconnect_remote_slot_keeps_claim_but_marks_not_ready() {
        let mut session = PartySessionSnapshot::new(3);
        session.sync_remote_slot(1, Some("Mia"), true, true, HeroRuntimeSummary::default());

        assert!(session.disconnect_remote_slot(1));

        let slot = session.slot(1).unwrap();
        assert_eq!(slot.claim, SlotClaimKind::Remote);
        assert!(!slot.connected);
        assert!(!slot.ready);
    }
}
