use super::*;

use acme_kit::text;

/// Where a rule audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleAuditStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RuleAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for RuleAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rule audit, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleAudit {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub sequence: i64,
    pub minimum: f64,
    pub retries: i64,
    pub title: String,
    pub weight: f64,
    pub ratio: f64,
    pub status: RuleAuditStatus,
}

impl RuleAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleAuditStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleAuditStatus) -> bool {
        match self.status {
            RuleAuditStatus::Draft => {
                return next == RuleAuditStatus::Suspended || next == RuleAuditStatus::Closed;
            }
            RuleAuditStatus::Pending => {
                match next {
                    RuleAuditStatus::Active | RuleAuditStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleAuditStatus::Active => {
                return next == RuleAuditStatus::Suspended || next == RuleAuditStatus::Closed;
            }
            RuleAuditStatus::Suspended => {
                return next == RuleAuditStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RuleAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rule audit", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.weight + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleAudit {
        RuleAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            sequence: self.sequence,
            minimum: self.minimum,
            retries: self.retries,
            title: self.title.clone(),
            weight: self.weight,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rule audit in status draft.
pub fn new_rule_audit(code: &str, name: &str) -> RuleAudit {
    RuleAudit {
        code: code.to_string(),
        name: name.to_string(),
        fingerprint: "".to_string(),
        sequence: 0,
        minimum: 0.0,
        retries: 0,
        title: "".to_string(),
        weight: 0.0,
        ratio: 0.0,
        status: RuleAuditStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_rule_audit("code-580", "name-853");
        assert_eq!(item.status, RuleAuditStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_rule_audit("code-580", "name-853");
        if item.move_to(RuleAuditStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RuleAuditStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_rule_audit("code-580", "name-853");
        item.status = RuleAuditStatus::Closed;
        assert!(item.move_to(RuleAuditStatus::Draft).is_err(), "expected move to fail");
    }
}
