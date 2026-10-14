use super::*;

use acme_kit::text;

/// Where a federation checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationCheckpointStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FederationCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for FederationCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One federation checkpoint, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationCheckpoint {
    pub code: String,
    pub name: String,
    pub version: i64,
    pub currency: String,
    pub principal: String,
    pub tenant_id: String,
    pub status: FederationCheckpointStatus,
}

impl FederationCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationCheckpointStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationCheckpointStatus) -> bool {
        match self.status {
            FederationCheckpointStatus::Open => {
                return next == FederationCheckpointStatus::InReview;
            }
            FederationCheckpointStatus::InReview => {
                match next {
                    FederationCheckpointStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationCheckpointStatus::Resolved => {
                return next == FederationCheckpointStatus::Reopened;
            }
            FederationCheckpointStatus::Reopened => {
                match next {
                    FederationCheckpointStatus::Archived => {
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

    pub fn move_to(&mut self, next: FederationCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.version as f64
    }

    pub fn copy(&self) -> FederationCheckpoint {
        FederationCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            version: self.version,
            currency: self.currency.clone(),
            principal: self.principal.clone(),
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a federation checkpoint in status open.
pub fn new_federation_checkpoint(code: &str, name: &str) -> FederationCheckpoint {
    FederationCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        version: 0,
        currency: "".to_string(),
        principal: "".to_string(),
        tenant_id: "".to_string(),
        status: FederationCheckpointStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_federation_checkpoint("code-976", "name-89");
        assert_eq!(item.status, FederationCheckpointStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_federation_checkpoint("code-976", "name-89");
        if item.move_to(FederationCheckpointStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationCheckpointStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_federation_checkpoint("code-976", "name-89");
        item.status = FederationCheckpointStatus::Archived;
        assert!(
            item.move_to(FederationCheckpointStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
