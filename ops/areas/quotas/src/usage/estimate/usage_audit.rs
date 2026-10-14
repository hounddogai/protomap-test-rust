use super::*;

use acme_kit::text;

/// Where a usage audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageAuditStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl UsageAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for UsageAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One usage audit, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageAudit {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub minimum: f64,
    pub maximum: f64,
    pub status: UsageAuditStatus,
}

impl UsageAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageAuditStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageAuditStatus) -> bool {
        match self.status {
            UsageAuditStatus::Requested => {
                return next == UsageAuditStatus::Granted || next == UsageAuditStatus::Expired;
            }
            UsageAuditStatus::Granted => {
                match next {
                    UsageAuditStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageAuditStatus::Revoked => {
                match next {
                    UsageAuditStatus::Expired => {
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

    pub fn move_to(&mut self, next: UsageAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "usage audit", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.maximum - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsageAudit {
        UsageAudit {
            code: self.code.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            minimum: self.minimum,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a usage audit in status requested.
pub fn new_usage_audit(code: &str, label: &str) -> UsageAudit {
    UsageAudit {
        code: code.to_string(),
        label: label.to_string(),
        currency: "".to_string(),
        minimum: 0.0,
        maximum: 0.0,
        status: UsageAuditStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_usage_audit("code-727", "label-699");
        assert_eq!(item.status, UsageAuditStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_usage_audit("code-727", "label-699");
        if item.move_to(UsageAuditStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsageAuditStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_usage_audit("code-727", "label-699");
        item.status = UsageAuditStatus::Expired;
        assert!(
            item.move_to(UsageAuditStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
