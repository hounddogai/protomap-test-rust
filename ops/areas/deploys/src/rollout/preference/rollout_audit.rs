use super::*;

use acme_kit::text;

/// Where a rollout audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutAuditStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RolloutAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RolloutAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollout audit as the rollout team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAudit {
    pub code: String,
    pub title: String,
    pub host: String,
    pub threshold: f64,
    pub unit: String,
    pub scope: String,
    pub retries: i64,
    pub created_at: i64,
    pub sequence: i64,
    pub cipher: String,
    pub status: RolloutAuditStatus,
}

impl RolloutAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutAuditStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutAuditStatus) -> bool {
        match self.status {
            RolloutAuditStatus::Requested => {
                match next {
                    RolloutAuditStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutAuditStatus::Granted => {
                match next {
                    RolloutAuditStatus::Revoked | RolloutAuditStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutAuditStatus::Revoked => {
                return next == RolloutAuditStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RolloutAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout audit {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.created_at as f64 + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutAudit {
        RolloutAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            host: self.host.clone(),
            threshold: self.threshold,
            unit: self.unit.clone(),
            scope: self.scope.clone(),
            retries: self.retries,
            created_at: self.created_at,
            sequence: self.sequence,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a rollout audit in status requested.
pub fn new_rollout_audit(code: &str, title: &str) -> RolloutAudit {
    RolloutAudit {
        code: code.to_string(),
        title: title.to_string(),
        host: "".to_string(),
        threshold: 0.0,
        unit: "".to_string(),
        scope: "".to_string(),
        retries: 0,
        created_at: 0,
        sequence: 0,
        cipher: "".to_string(),
        status: RolloutAuditStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_rollout_audit("code-207", "title-343");
        assert_eq!(item.status, RolloutAuditStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_rollout_audit("code-207", "title-343");
        if item.move_to(RolloutAuditStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutAuditStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_rollout_audit("code-207", "title-343");
        item.status = RolloutAuditStatus::Expired;
        assert!(
            item.move_to(RolloutAuditStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
