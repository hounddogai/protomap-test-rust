use super::*;

use acme_kit::text;

/// Where a region pin slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinSlotStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RegionPinSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RegionPinSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One region pin slot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinSlot {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub description: String,
    pub capacity: i64,
    pub namespace: String,
    pub unit: String,
    pub status: RegionPinSlotStatus,
}

impl RegionPinSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinSlotStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinSlotStatus) -> bool {
        match self.status {
            RegionPinSlotStatus::Requested => {
                match next {
                    RegionPinSlotStatus::Granted | RegionPinSlotStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinSlotStatus::Granted => {
                match next {
                    RegionPinSlotStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinSlotStatus::Revoked => {
                match next {
                    RegionPinSlotStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionPinSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.port as f64) * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinSlot {
        RegionPinSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            description: self.description.clone(),
            capacity: self.capacity,
            namespace: self.namespace.clone(),
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a region pin slot in status requested.
pub fn new_region_pin_slot(code: &str, name: &str) -> RegionPinSlot {
    RegionPinSlot {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        description: "".to_string(),
        capacity: 0,
        namespace: "".to_string(),
        unit: "".to_string(),
        status: RegionPinSlotStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_region_pin_slot("code-565", "name-588");
        assert_eq!(item.status, RegionPinSlotStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_region_pin_slot("code-565", "name-588");
        if item.move_to(RegionPinSlotStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionPinSlotStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_region_pin_slot("code-565", "name-588");
        item.status = RegionPinSlotStatus::Expired;
        assert!(
            item.move_to(RegionPinSlotStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
