use super::*;

use acme_kit::text;

/// Where a rule profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleProfileStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RuleProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RuleProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rule profile. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleProfile {
    pub code: String,
    pub name: String,
    pub maximum: f64,
    pub attempts: i64,
    pub retries: i64,
    pub scope: String,
    pub summary: String,
    pub revision: i64,
    pub status: RuleProfileStatus,
}

impl RuleProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleProfileStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleProfileStatus) -> bool {
        match self.status {
            RuleProfileStatus::Requested => {
                match next {
                    RuleProfileStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleProfileStatus::Granted => {
                return next == RuleProfileStatus::Revoked;
            }
            RuleProfileStatus::Revoked => {
                match next {
                    RuleProfileStatus::Expired => {
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

    pub fn move_to(&mut self, next: RuleProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rule profile", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.maximum - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleProfile {
        RuleProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            maximum: self.maximum,
            attempts: self.attempts,
            retries: self.retries,
            scope: self.scope.clone(),
            summary: self.summary.clone(),
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a rule profile in status requested.
pub fn new_rule_profile(code: &str, name: &str) -> RuleProfile {
    RuleProfile {
        code: code.to_string(),
        name: name.to_string(),
        maximum: 0.0,
        attempts: 0,
        retries: 0,
        scope: "".to_string(),
        summary: "".to_string(),
        revision: 0,
        status: RuleProfileStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_rule_profile("code-997", "name-515");
        assert_eq!(item.status, RuleProfileStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_rule_profile("code-997", "name-515");
        if item.move_to(RuleProfileStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RuleProfileStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_rule_profile("code-997", "name-515");
        item.status = RuleProfileStatus::Expired;
        assert!(
            item.move_to(RuleProfileStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
