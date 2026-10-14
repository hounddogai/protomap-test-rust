use super::*;

use acme_kit::text;

/// Where a weighted policy rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedPolicyRuleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl WeightedPolicyRuleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for WeightedPolicyRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted policy rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPolicyRule {
    pub code: String,
    pub name: String,
    pub cost: f64,
    pub revision: i64,
    pub cipher: String,
    pub active: bool,
    pub segment: String,
    pub status: WeightedPolicyRuleStatus,
}

impl WeightedPolicyRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedPolicyRuleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedPolicyRuleStatus) -> bool {
        match self.status {
            WeightedPolicyRuleStatus::Requested => {
                match next {
                    WeightedPolicyRuleStatus::Granted | WeightedPolicyRuleStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedPolicyRuleStatus::Granted => {
                return next == WeightedPolicyRuleStatus::Expired;
            }
            WeightedPolicyRuleStatus::Revoked => {
                match next {
                    WeightedPolicyRuleStatus::Expired => {
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

    pub fn move_to(&mut self, next: WeightedPolicyRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted policy rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.revision as f64) * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedPolicyRule {
        WeightedPolicyRule {
            code: self.code.clone(),
            name: self.name.clone(),
            cost: self.cost,
            revision: self.revision,
            cipher: self.cipher.clone(),
            active: self.active,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a weighted policy rule in status requested.
pub fn new_weighted_policy_rule(code: &str, name: &str) -> WeightedPolicyRule {
    WeightedPolicyRule {
        code: code.to_string(),
        name: name.to_string(),
        cost: 0.0,
        revision: 0,
        cipher: "".to_string(),
        active: false,
        segment: "".to_string(),
        status: WeightedPolicyRuleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_weighted_policy_rule("code-820", "name-153");
        assert_eq!(item.status, WeightedPolicyRuleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_weighted_policy_rule("code-820", "name-153");
        if item.move_to(WeightedPolicyRuleStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedPolicyRuleStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_weighted_policy_rule("code-820", "name-153");
        item.status = WeightedPolicyRuleStatus::Expired;
        assert!(
            item.move_to(WeightedPolicyRuleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
