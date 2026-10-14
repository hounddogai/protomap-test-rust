use super::*;

use acme_kit::text;

/// Where a rolling alert slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingAlertSlotStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RollingAlertSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for RollingAlertSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling alert slot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingAlertSlot {
    pub code: String,
    pub name: String,
    pub country: String,
    pub host: String,
    pub region: String,
    pub checksum: String,
    pub retries: i64,
    pub key_id: String,
    pub maximum: f64,
    pub margin: f64,
    pub status: RollingAlertSlotStatus,
}

impl RollingAlertSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingAlertSlotStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingAlertSlotStatus) -> bool {
        match self.status {
            RollingAlertSlotStatus::Draft => {
                return next == RollingAlertSlotStatus::Suspended;
            }
            RollingAlertSlotStatus::Pending => {
                return next == RollingAlertSlotStatus::Closed;
            }
            RollingAlertSlotStatus::Active => {
                match next {
                    RollingAlertSlotStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingAlertSlotStatus::Suspended => {
                return next == RollingAlertSlotStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingAlertSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling alert slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rolling alert slot", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.maximum + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingAlertSlot {
        RollingAlertSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            host: self.host.clone(),
            region: self.region.clone(),
            checksum: self.checksum.clone(),
            retries: self.retries,
            key_id: self.key_id.clone(),
            maximum: self.maximum,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a rolling alert slot in status draft.
pub fn new_rolling_alert_slot(code: &str, name: &str) -> RollingAlertSlot {
    RollingAlertSlot {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        host: "".to_string(),
        region: "".to_string(),
        checksum: "".to_string(),
        retries: 0,
        key_id: "".to_string(),
        maximum: 0.0,
        margin: 0.0,
        status: RollingAlertSlotStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_rolling_alert_slot("code-123", "name-108");
        assert_eq!(item.status, RollingAlertSlotStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_rolling_alert_slot("code-123", "name-108");
        if item.move_to(RollingAlertSlotStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingAlertSlotStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_rolling_alert_slot("code-123", "name-108");
        item.status = RollingAlertSlotStatus::Closed;
        assert!(
            item.move_to(RollingAlertSlotStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
