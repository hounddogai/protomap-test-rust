use super::*;

use acme_kit::text;

/// Where a regional allowlist slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalAllowlistSlotStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RegionalAllowlistSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RegionalAllowlistSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional allowlist slot as the allowlist team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalAllowlistSlot {
    pub code: String,
    pub name: String,
    pub created_at: i64,
    pub starts_at: i64,
    pub cost: f64,
    pub region: String,
    pub status: RegionalAllowlistSlotStatus,
}

impl RegionalAllowlistSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalAllowlistSlotStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalAllowlistSlotStatus) -> bool {
        match self.status {
            RegionalAllowlistSlotStatus::Requested => {
                return next == RegionalAllowlistSlotStatus::Granted || next == RegionalAllowlistSlotStatus::Expired;
            }
            RegionalAllowlistSlotStatus::Granted => {
                match next {
                    RegionalAllowlistSlotStatus::Revoked | RegionalAllowlistSlotStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalAllowlistSlotStatus::Revoked => {
                match next {
                    RegionalAllowlistSlotStatus::Expired => {
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

    pub fn move_to(&mut self, next: RegionalAllowlistSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional allowlist slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.starts_at as f64 + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalAllowlistSlot {
        RegionalAllowlistSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            created_at: self.created_at,
            starts_at: self.starts_at,
            cost: self.cost,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a regional allowlist slot in status requested.
pub fn new_regional_allowlist_slot(code: &str, name: &str) -> RegionalAllowlistSlot {
    RegionalAllowlistSlot {
        code: code.to_string(),
        name: name.to_string(),
        created_at: 0,
        starts_at: 0,
        cost: 0.0,
        region: "".to_string(),
        status: RegionalAllowlistSlotStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_regional_allowlist_slot("code-681", "name-584");
        assert_eq!(item.status, RegionalAllowlistSlotStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_regional_allowlist_slot("code-681", "name-584");
        if item.move_to(RegionalAllowlistSlotStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalAllowlistSlotStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_regional_allowlist_slot("code-681", "name-584");
        item.status = RegionalAllowlistSlotStatus::Expired;
        assert!(
            item.move_to(RegionalAllowlistSlotStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
