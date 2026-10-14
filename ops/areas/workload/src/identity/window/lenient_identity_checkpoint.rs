use super::*;

use acme_kit::text;

/// Where a lenient identity checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientIdentityCheckpointStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LenientIdentityCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for LenientIdentityCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient identity checkpoint, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientIdentityCheckpoint {
    pub code: String,
    pub title: String,
    pub tenant_id: String,
    pub attempts: i64,
    pub quota_units: i64,
    pub currency: String,
    pub region: String,
    pub expires_at: i64,
    pub created_at: i64,
    pub status: LenientIdentityCheckpointStatus,
}

impl LenientIdentityCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientIdentityCheckpointStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientIdentityCheckpointStatus) -> bool {
        match self.status {
            LenientIdentityCheckpointStatus::Planned => {
                return next == LenientIdentityCheckpointStatus::Scheduled;
            }
            LenientIdentityCheckpointStatus::Scheduled => {
                return next == LenientIdentityCheckpointStatus::InProgress;
            }
            LenientIdentityCheckpointStatus::InProgress => {
                match next {
                    LenientIdentityCheckpointStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientIdentityCheckpointStatus::Completed => {
                match next {
                    LenientIdentityCheckpointStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: LenientIdentityCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient identity checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.created_at as f64 - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientIdentityCheckpoint {
        LenientIdentityCheckpoint {
            code: self.code.clone(),
            title: self.title.clone(),
            tenant_id: self.tenant_id.clone(),
            attempts: self.attempts,
            quota_units: self.quota_units,
            currency: self.currency.clone(),
            region: self.region.clone(),
            expires_at: self.expires_at,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a lenient identity checkpoint in status planned.
pub fn new_lenient_identity_checkpoint(code: &str, title: &str) -> LenientIdentityCheckpoint {
    LenientIdentityCheckpoint {
        code: code.to_string(),
        title: title.to_string(),
        tenant_id: "".to_string(),
        attempts: 0,
        quota_units: 0,
        currency: "".to_string(),
        region: "".to_string(),
        expires_at: 0,
        created_at: 0,
        status: LenientIdentityCheckpointStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_lenient_identity_checkpoint("code-84", "title-298");
        assert_eq!(item.status, LenientIdentityCheckpointStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_lenient_identity_checkpoint("code-84", "title-298");
        if item.move_to(LenientIdentityCheckpointStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientIdentityCheckpointStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_lenient_identity_checkpoint("code-84", "title-298");
        item.status = LenientIdentityCheckpointStatus::Cancelled;
        assert!(
            item.move_to(LenientIdentityCheckpointStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
