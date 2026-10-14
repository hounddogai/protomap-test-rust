use super::*;

use acme_kit::text;

/// Where a rollback quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackQuotaStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RollbackQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RollbackQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollback quota. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackQuota {
    pub code: String,
    pub label: String,
    pub ceiling: f64,
    pub approved: bool,
    pub notes: String,
    pub external_ref: String,
    pub version: i64,
    pub status: RollbackQuotaStatus,
}

impl RollbackQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackQuotaStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackQuotaStatus) -> bool {
        match self.status {
            RollbackQuotaStatus::Requested => {
                match next {
                    RollbackQuotaStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackQuotaStatus::Granted => {
                match next {
                    RollbackQuotaStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackQuotaStatus::Revoked => {
                match next {
                    RollbackQuotaStatus::Expired => {
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

    pub fn move_to(&mut self, next: RollbackQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.ceiling + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollbackQuota {
        RollbackQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            ceiling: self.ceiling,
            approved: self.approved,
            notes: self.notes.clone(),
            external_ref: self.external_ref.clone(),
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a rollback quota in status requested.
pub fn new_rollback_quota(code: &str, label: &str) -> RollbackQuota {
    RollbackQuota {
        code: code.to_string(),
        label: label.to_string(),
        ceiling: 0.0,
        approved: false,
        notes: "".to_string(),
        external_ref: "".to_string(),
        version: 0,
        status: RollbackQuotaStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_rollback_quota("code-530", "label-972");
        assert_eq!(item.status, RollbackQuotaStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_rollback_quota("code-530", "label-972");
        if item.move_to(RollbackQuotaStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackQuotaStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_rollback_quota("code-530", "label-972");
        item.status = RollbackQuotaStatus::Expired;
        assert!(
            item.move_to(RollbackQuotaStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
