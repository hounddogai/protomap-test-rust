use super::*;

use acme_kit::text;

/// Where a rollback checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackCheckpointStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RollbackCheckpointStatus {
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

impl std::fmt::Display for RollbackCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rollback checkpoint, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackCheckpoint {
    pub code: String,
    pub name: String,
    pub quota_units: i64,
    pub margin: f64,
    pub starts_at: i64,
    pub currency: String,
    pub status: RollbackCheckpointStatus,
}

impl RollbackCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackCheckpointStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackCheckpointStatus) -> bool {
        match self.status {
            RollbackCheckpointStatus::Draft => {
                match next {
                    RollbackCheckpointStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackCheckpointStatus::Pending => {
                match next {
                    RollbackCheckpointStatus::Active | RollbackCheckpointStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackCheckpointStatus::Active => {
                return next == RollbackCheckpointStatus::Suspended;
            }
            RollbackCheckpointStatus::Suspended => {
                match next {
                    RollbackCheckpointStatus::Closed => {
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

    pub fn move_to(&mut self, next: RollbackCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.margin + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollbackCheckpoint {
        RollbackCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            quota_units: self.quota_units,
            margin: self.margin,
            starts_at: self.starts_at,
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rollback checkpoint in status draft.
pub fn new_rollback_checkpoint(code: &str, name: &str) -> RollbackCheckpoint {
    RollbackCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        quota_units: 0,
        margin: 0.0,
        starts_at: 0,
        currency: "".to_string(),
        status: RollbackCheckpointStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_rollback_checkpoint("code-796", "name-528");
        assert_eq!(item.status, RollbackCheckpointStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_rollback_checkpoint("code-796", "name-528");
        if item.move_to(RollbackCheckpointStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackCheckpointStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_rollback_checkpoint("code-796", "name-528");
        item.status = RollbackCheckpointStatus::Closed;
        assert!(
            item.move_to(RollbackCheckpointStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
