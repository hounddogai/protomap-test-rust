use super::*;

use acme_kit::text;

/// Where a rollback threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackThresholdStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RollbackThresholdStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RollbackThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rollback threshold and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackThreshold {
    pub code: String,
    pub label: String,
    pub pinned: bool,
    pub channel: String,
    pub currency: String,
    pub name: String,
    pub approved: bool,
    pub ends_at: i64,
    pub status: RollbackThresholdStatus,
}

impl RollbackThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackThresholdStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackThresholdStatus) -> bool {
        match self.status {
            RollbackThresholdStatus::Created => {
                match next {
                    RollbackThresholdStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackThresholdStatus::Validated => {
                return next == RollbackThresholdStatus::Retired;
            }
            RollbackThresholdStatus::Published => {
                match next {
                    RollbackThresholdStatus::Retired => {
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

    pub fn move_to(&mut self, next: RollbackThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.ends_at as f64
    }

    pub fn copy(&self) -> RollbackThreshold {
        RollbackThreshold {
            code: self.code.clone(),
            label: self.label.clone(),
            pinned: self.pinned,
            channel: self.channel.clone(),
            currency: self.currency.clone(),
            name: self.name.clone(),
            approved: self.approved,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a rollback threshold in status created.
pub fn new_rollback_threshold(code: &str, label: &str) -> RollbackThreshold {
    RollbackThreshold {
        code: code.to_string(),
        label: label.to_string(),
        pinned: false,
        channel: "".to_string(),
        currency: "".to_string(),
        name: "".to_string(),
        approved: false,
        ends_at: 0,
        status: RollbackThresholdStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_rollback_threshold("code-87", "label-98");
        assert_eq!(item.status, RollbackThresholdStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_rollback_threshold("code-87", "label-98");
        if item.move_to(RollbackThresholdStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackThresholdStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_rollback_threshold("code-87", "label-98");
        item.status = RollbackThresholdStatus::Retired;
        assert!(
            item.move_to(RollbackThresholdStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
