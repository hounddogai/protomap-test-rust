use super::*;

use acme_kit::text;

/// Where a grant audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantAuditStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl GrantAuditStatus {
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

impl std::fmt::Display for GrantAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A grant audit as the grant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantAudit {
    pub code: String,
    pub title: String,
    pub quantity: i64,
    pub region: String,
    pub ttl_seconds: i64,
    pub retries: i64,
    pub timezone: String,
    pub status: GrantAuditStatus,
}

impl GrantAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantAuditStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantAuditStatus) -> bool {
        match self.status {
            GrantAuditStatus::Open => {
                match next {
                    GrantAuditStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantAuditStatus::InReview => {
                match next {
                    GrantAuditStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantAuditStatus::Resolved => {
                return next == GrantAuditStatus::Reopened || next == GrantAuditStatus::Archived;
            }
            GrantAuditStatus::Reopened => {
                match next {
                    GrantAuditStatus::Archived => {
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

    pub fn move_to(&mut self, next: GrantAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.ttl_seconds as f64 - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantAudit {
        GrantAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            region: self.region.clone(),
            ttl_seconds: self.ttl_seconds,
            retries: self.retries,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a grant audit in status open.
pub fn new_grant_audit(code: &str, title: &str) -> GrantAudit {
    GrantAudit {
        code: code.to_string(),
        title: title.to_string(),
        quantity: 0,
        region: "".to_string(),
        ttl_seconds: 0,
        retries: 0,
        timezone: "".to_string(),
        status: GrantAuditStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_grant_audit("code-268", "title-653");
        assert_eq!(item.status, GrantAuditStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_grant_audit("code-268", "title-653");
        if item.move_to(GrantAuditStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GrantAuditStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_grant_audit("code-268", "title-653");
        item.status = GrantAuditStatus::Archived;
        assert!(item.move_to(GrantAuditStatus::Open).is_err(), "expected move to fail");
    }
}
