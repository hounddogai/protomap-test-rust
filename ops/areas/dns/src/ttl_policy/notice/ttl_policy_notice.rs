use super::*;

use acme_kit::text;

/// Where a ttl policy notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyNoticeStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TtlPolicyNoticeStatus {
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

impl std::fmt::Display for TtlPolicyNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A ttl policy notice. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyNotice {
    pub code: String,
    pub name: String,
    pub segment: String,
    pub key_id: String,
    pub limit: i64,
    pub status: TtlPolicyNoticeStatus,
}

impl TtlPolicyNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyNoticeStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyNoticeStatus) -> bool {
        match self.status {
            TtlPolicyNoticeStatus::Open => {
                return next == TtlPolicyNoticeStatus::Reopened || next == TtlPolicyNoticeStatus::Archived;
            }
            TtlPolicyNoticeStatus::InReview => {
                return next == TtlPolicyNoticeStatus::Reopened || next == TtlPolicyNoticeStatus::Archived;
            }
            TtlPolicyNoticeStatus::Resolved => {
                return next == TtlPolicyNoticeStatus::Archived;
            }
            TtlPolicyNoticeStatus::Reopened => {
                match next {
                    TtlPolicyNoticeStatus::Archived => {
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

    pub fn move_to(&mut self, next: TtlPolicyNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.limit as f64
    }

    pub fn copy(&self) -> TtlPolicyNotice {
        TtlPolicyNotice {
            code: self.code.clone(),
            name: self.name.clone(),
            segment: self.segment.clone(),
            key_id: self.key_id.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a ttl policy notice in status open.
pub fn new_ttl_policy_notice(code: &str, name: &str) -> TtlPolicyNotice {
    TtlPolicyNotice {
        code: code.to_string(),
        name: name.to_string(),
        segment: "".to_string(),
        key_id: "".to_string(),
        limit: 0,
        status: TtlPolicyNoticeStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_ttl_policy_notice("code-417", "name-344");
        assert_eq!(item.status, TtlPolicyNoticeStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_ttl_policy_notice("code-417", "name-344");
        if item.move_to(TtlPolicyNoticeStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyNoticeStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_ttl_policy_notice("code-417", "name-344");
        item.status = TtlPolicyNoticeStatus::Archived;
        assert!(
            item.move_to(TtlPolicyNoticeStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
