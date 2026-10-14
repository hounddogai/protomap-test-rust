use super::*;

use acme_kit::text;

/// Where a delegation slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationSlotStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DelegationSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DelegationSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A delegation slot as the delegation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationSlot {
    pub code: String,
    pub name: String,
    pub notes: String,
    pub cipher: String,
    pub ratio: f64,
    pub maximum: f64,
    pub segment: String,
    pub duration_minutes: i64,
    pub host: String,
    pub rate: f64,
    pub margin: f64,
    pub status: DelegationSlotStatus,
}

impl DelegationSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationSlotStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationSlotStatus) -> bool {
        match self.status {
            DelegationSlotStatus::Requested => {
                return next == DelegationSlotStatus::Granted;
            }
            DelegationSlotStatus::Granted => {
                match next {
                    DelegationSlotStatus::Revoked | DelegationSlotStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationSlotStatus::Revoked => {
                match next {
                    DelegationSlotStatus::Expired => {
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

    pub fn move_to(&mut self, next: DelegationSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "delegation slot", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.ratio - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationSlot {
        DelegationSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            notes: self.notes.clone(),
            cipher: self.cipher.clone(),
            ratio: self.ratio,
            maximum: self.maximum,
            segment: self.segment.clone(),
            duration_minutes: self.duration_minutes,
            host: self.host.clone(),
            rate: self.rate,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a delegation slot in status requested.
pub fn new_delegation_slot(code: &str, name: &str) -> DelegationSlot {
    DelegationSlot {
        code: code.to_string(),
        name: name.to_string(),
        notes: "".to_string(),
        cipher: "".to_string(),
        ratio: 0.0,
        maximum: 0.0,
        segment: "".to_string(),
        duration_minutes: 0,
        host: "".to_string(),
        rate: 0.0,
        margin: 0.0,
        status: DelegationSlotStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_delegation_slot("code-174", "name-158");
        assert_eq!(item.status, DelegationSlotStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_delegation_slot("code-174", "name-158");
        if item.move_to(DelegationSlotStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DelegationSlotStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_delegation_slot("code-174", "name-158");
        item.status = DelegationSlotStatus::Expired;
        assert!(
            item.move_to(DelegationSlotStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
