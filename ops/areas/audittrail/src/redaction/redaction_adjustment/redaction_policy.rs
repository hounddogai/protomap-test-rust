use super::*;

use acme_kit::text;

/// Where a redaction policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionPolicyStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RedactionPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RedactionPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a redaction policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionPolicy {
    pub code: String,
    pub title: String,
    pub host: String,
    pub weight: f64,
    pub unit: String,
    pub namespace: String,
    pub status: RedactionPolicyStatus,
}

impl RedactionPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionPolicyStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionPolicyStatus) -> bool {
        match self.status {
            RedactionPolicyStatus::Requested => {
                match next {
                    RedactionPolicyStatus::Granted | RedactionPolicyStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionPolicyStatus::Granted => {
                match next {
                    RedactionPolicyStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionPolicyStatus::Revoked => {
                return next == RedactionPolicyStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RedactionPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.weight
    }

    pub fn copy(&self) -> RedactionPolicy {
        RedactionPolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            host: self.host.clone(),
            weight: self.weight,
            unit: self.unit.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a redaction policy in status requested.
pub fn new_redaction_policy(code: &str, title: &str) -> RedactionPolicy {
    RedactionPolicy {
        code: code.to_string(),
        title: title.to_string(),
        host: "".to_string(),
        weight: 0.0,
        unit: "".to_string(),
        namespace: "".to_string(),
        status: RedactionPolicyStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_redaction_policy("code-903", "title-97");
        assert_eq!(item.status, RedactionPolicyStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_redaction_policy("code-903", "title-97");
        if item.move_to(RedactionPolicyStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionPolicyStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_redaction_policy("code-903", "title-97");
        item.status = RedactionPolicyStatus::Expired;
        assert!(
            item.move_to(RedactionPolicyStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
