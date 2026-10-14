use super::*;

use acme_kit::text;

/// Where a rule digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleDigestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RuleDigestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RuleDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rule digest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleDigest {
    pub code: String,
    pub name: String,
    pub amount: f64,
    pub principal: String,
    pub revision: i64,
    pub burst: i64,
    pub status: RuleDigestStatus,
}

impl RuleDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleDigestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleDigestStatus) -> bool {
        match self.status {
            RuleDigestStatus::Proposed => {
                match next {
                    RuleDigestStatus::Approved | RuleDigestStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleDigestStatus::Approved => {
                match next {
                    RuleDigestStatus::Rejected | RuleDigestStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleDigestStatus::Rejected => {
                return next == RuleDigestStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RuleDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.amount + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleDigest {
        RuleDigest {
            code: self.code.clone(),
            name: self.name.clone(),
            amount: self.amount,
            principal: self.principal.clone(),
            revision: self.revision,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rule digest in status proposed.
pub fn new_rule_digest(code: &str, name: &str) -> RuleDigest {
    RuleDigest {
        code: code.to_string(),
        name: name.to_string(),
        amount: 0.0,
        principal: "".to_string(),
        revision: 0,
        burst: 0,
        status: RuleDigestStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_rule_digest("code-397", "name-528");
        assert_eq!(item.status, RuleDigestStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_rule_digest("code-397", "name-528");
        if item.move_to(RuleDigestStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RuleDigestStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_rule_digest("code-397", "name-528");
        item.status = RuleDigestStatus::Withdrawn;
        assert!(
            item.move_to(RuleDigestStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
