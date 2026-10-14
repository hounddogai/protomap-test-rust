use super::*;

use acme_kit::text;

/// Where a alert override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertOverrideStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AlertOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AlertOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One alert override, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertOverride {
    pub code: String,
    pub name: String,
    pub revision: i64,
    pub fingerprint: String,
    pub audience: String,
    pub floor_value: f64,
    pub ends_at: i64,
    pub rate: f64,
    pub version: i64,
    pub notes: String,
    pub unit: String,
    pub status: AlertOverrideStatus,
}

impl AlertOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertOverrideStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertOverrideStatus) -> bool {
        match self.status {
            AlertOverrideStatus::Proposed => {
                return next == AlertOverrideStatus::Rejected;
            }
            AlertOverrideStatus::Approved => {
                return next == AlertOverrideStatus::Rejected || next == AlertOverrideStatus::Withdrawn;
            }
            AlertOverrideStatus::Rejected => {
                match next {
                    AlertOverrideStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: AlertOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.floor_value + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertOverride {
        AlertOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            revision: self.revision,
            fingerprint: self.fingerprint.clone(),
            audience: self.audience.clone(),
            floor_value: self.floor_value,
            ends_at: self.ends_at,
            rate: self.rate,
            version: self.version,
            notes: self.notes.clone(),
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a alert override in status proposed.
pub fn new_alert_override(code: &str, name: &str) -> AlertOverride {
    AlertOverride {
        code: code.to_string(),
        name: name.to_string(),
        revision: 0,
        fingerprint: "".to_string(),
        audience: "".to_string(),
        floor_value: 0.0,
        ends_at: 0,
        rate: 0.0,
        version: 0,
        notes: "".to_string(),
        unit: "".to_string(),
        status: AlertOverrideStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_alert_override("code-740", "name-338");
        assert_eq!(item.status, AlertOverrideStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_alert_override("code-740", "name-338");
        if item.move_to(AlertOverrideStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AlertOverrideStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_alert_override("code-740", "name-338");
        item.status = AlertOverrideStatus::Withdrawn;
        assert!(
            item.move_to(AlertOverrideStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
