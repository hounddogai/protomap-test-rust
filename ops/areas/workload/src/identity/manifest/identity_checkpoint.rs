use super::*;

use acme_kit::text;

/// Where a identity checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityCheckpointStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl IdentityCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for IdentityCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a identity checkpoint and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityCheckpoint {
    pub code: String,
    pub name: String,
    pub rate: f64,
    pub grace_days: i64,
    pub description: String,
    pub principal: String,
    pub ceiling: f64,
    pub status: IdentityCheckpointStatus,
}

impl IdentityCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityCheckpointStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityCheckpointStatus) -> bool {
        match self.status {
            IdentityCheckpointStatus::Created => {
                return next == IdentityCheckpointStatus::Validated;
            }
            IdentityCheckpointStatus::Validated => {
                match next {
                    IdentityCheckpointStatus::Published | IdentityCheckpointStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityCheckpointStatus::Published => {
                return next == IdentityCheckpointStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IdentityCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.rate - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IdentityCheckpoint {
        IdentityCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            rate: self.rate,
            grace_days: self.grace_days,
            description: self.description.clone(),
            principal: self.principal.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a identity checkpoint in status created.
pub fn new_identity_checkpoint(code: &str, name: &str) -> IdentityCheckpoint {
    IdentityCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        rate: 0.0,
        grace_days: 0,
        description: "".to_string(),
        principal: "".to_string(),
        ceiling: 0.0,
        status: IdentityCheckpointStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_identity_checkpoint("code-134", "name-214");
        assert_eq!(item.status, IdentityCheckpointStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_identity_checkpoint("code-134", "name-214");
        if item.move_to(IdentityCheckpointStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentityCheckpointStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_identity_checkpoint("code-134", "name-214");
        item.status = IdentityCheckpointStatus::Retired;
        assert!(
            item.move_to(IdentityCheckpointStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
