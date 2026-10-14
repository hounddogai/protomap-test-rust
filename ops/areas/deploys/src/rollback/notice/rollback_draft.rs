use super::*;

use acme_kit::text;

/// Where a rollback draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackDraftStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RollbackDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RollbackDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollback draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackDraft {
    pub code: String,
    pub name: String,
    pub updated_at: i64,
    pub currency: String,
    pub cluster: String,
    pub attempts: i64,
    pub region: String,
    pub timezone: String,
    pub status: RollbackDraftStatus,
}

impl RollbackDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackDraftStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackDraftStatus) -> bool {
        match self.status {
            RollbackDraftStatus::Proposed => {
                match next {
                    RollbackDraftStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackDraftStatus::Approved => {
                return next == RollbackDraftStatus::Rejected || next == RollbackDraftStatus::Withdrawn;
            }
            RollbackDraftStatus::Rejected => {
                match next {
                    RollbackDraftStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RollbackDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.updated_at as f64 + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollbackDraft {
        RollbackDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            updated_at: self.updated_at,
            currency: self.currency.clone(),
            cluster: self.cluster.clone(),
            attempts: self.attempts,
            region: self.region.clone(),
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a rollback draft in status proposed.
pub fn new_rollback_draft(code: &str, name: &str) -> RollbackDraft {
    RollbackDraft {
        code: code.to_string(),
        name: name.to_string(),
        updated_at: 0,
        currency: "".to_string(),
        cluster: "".to_string(),
        attempts: 0,
        region: "".to_string(),
        timezone: "".to_string(),
        status: RollbackDraftStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_rollback_draft("code-992", "name-744");
        assert_eq!(item.status, RollbackDraftStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_rollback_draft("code-992", "name-744");
        if item.move_to(RollbackDraftStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackDraftStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_rollback_draft("code-992", "name-744");
        item.status = RollbackDraftStatus::Withdrawn;
        assert!(
            item.move_to(RollbackDraftStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
