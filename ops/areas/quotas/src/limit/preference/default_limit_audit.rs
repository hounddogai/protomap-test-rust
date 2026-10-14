use super::*;

use acme_kit::text;

/// Where a default limit audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultLimitAuditStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DefaultLimitAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DefaultLimitAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default limit audit and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultLimitAudit {
    pub code: String,
    pub label: String,
    pub ttl_seconds: i64,
    pub category: String,
    pub floor_value: f64,
    pub sequence: i64,
    pub attempts: i64,
    pub status: DefaultLimitAuditStatus,
}

impl DefaultLimitAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultLimitAuditStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultLimitAuditStatus) -> bool {
        match self.status {
            DefaultLimitAuditStatus::Requested => {
                match next {
                    DefaultLimitAuditStatus::Granted | DefaultLimitAuditStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultLimitAuditStatus::Granted => {
                return next == DefaultLimitAuditStatus::Expired;
            }
            DefaultLimitAuditStatus::Revoked => {
                return next == DefaultLimitAuditStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultLimitAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default limit audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.floor_value + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultLimitAudit {
        DefaultLimitAudit {
            code: self.code.clone(),
            label: self.label.clone(),
            ttl_seconds: self.ttl_seconds,
            category: self.category.clone(),
            floor_value: self.floor_value,
            sequence: self.sequence,
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a default limit audit in status requested.
pub fn new_default_limit_audit(code: &str, label: &str) -> DefaultLimitAudit {
    DefaultLimitAudit {
        code: code.to_string(),
        label: label.to_string(),
        ttl_seconds: 0,
        category: "".to_string(),
        floor_value: 0.0,
        sequence: 0,
        attempts: 0,
        status: DefaultLimitAuditStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_default_limit_audit("code-759", "label-784");
        assert_eq!(item.status, DefaultLimitAuditStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_default_limit_audit("code-759", "label-784");
        if item.move_to(DefaultLimitAuditStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultLimitAuditStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_default_limit_audit("code-759", "label-784");
        item.status = DefaultLimitAuditStatus::Expired;
        assert!(
            item.move_to(DefaultLimitAuditStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
