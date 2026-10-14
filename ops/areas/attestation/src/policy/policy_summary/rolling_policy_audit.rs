use super::*;

use acme_kit::text;

/// Where a rolling policy audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingPolicyAuditStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RollingPolicyAuditStatus {
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

impl std::fmt::Display for RollingPolicyAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling policy audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPolicyAudit {
    pub code: String,
    pub title: String,
    pub archived: bool,
    pub ends_at: i64,
    pub margin: f64,
    pub phone: String,
    pub reviewer: String,
    pub expires_at: i64,
    pub status: RollingPolicyAuditStatus,
}

impl RollingPolicyAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingPolicyAuditStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingPolicyAuditStatus) -> bool {
        match self.status {
            RollingPolicyAuditStatus::Planned => {
                match next {
                    RollingPolicyAuditStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingPolicyAuditStatus::Scheduled => {
                match next {
                    RollingPolicyAuditStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingPolicyAuditStatus::InProgress => {
                match next {
                    RollingPolicyAuditStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingPolicyAuditStatus::Completed => {
                match next {
                    RollingPolicyAuditStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RollingPolicyAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling policy audit {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.expires_at as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingPolicyAudit {
        RollingPolicyAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            archived: self.archived,
            ends_at: self.ends_at,
            margin: self.margin,
            phone: self.phone.clone(),
            reviewer: self.reviewer.clone(),
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rolling policy audit in status planned.
pub fn new_rolling_policy_audit(code: &str, title: &str) -> RollingPolicyAudit {
    RollingPolicyAudit {
        code: code.to_string(),
        title: title.to_string(),
        archived: false,
        ends_at: 0,
        margin: 0.0,
        phone: "".to_string(),
        reviewer: "".to_string(),
        expires_at: 0,
        status: RollingPolicyAuditStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_rolling_policy_audit("code-651", "title-459");
        assert_eq!(item.status, RollingPolicyAuditStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_rolling_policy_audit("code-651", "title-459");
        if item.move_to(RollingPolicyAuditStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingPolicyAuditStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_rolling_policy_audit("code-651", "title-459");
        item.status = RollingPolicyAuditStatus::Cancelled;
        assert!(
            item.move_to(RollingPolicyAuditStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
