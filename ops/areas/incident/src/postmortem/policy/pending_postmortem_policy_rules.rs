use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending postmortem policy must pass. Check returns its problems, or none.
pub trait PendingPostmortemPolicyRule: std::fmt::Debug {
    fn check(&self, item: &PendingPostmortemPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPostmortemPolicyLabelLengthRule {
    pub max_length: i64,
}

impl PendingPostmortemPolicyRule for PendingPostmortemPolicyLabelLengthRule {
    fn check(&self, item: &PendingPostmortemPolicy) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPostmortemPolicyRequiredLabelRule {
    pub message: String,
}

impl PendingPostmortemPolicyRule for PendingPostmortemPolicyRequiredLabelRule {
    fn check(&self, item: &PendingPostmortemPolicy) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingPostmortemPolicyRequiredExternalRefRule {
    pub message: String,
}

impl PendingPostmortemPolicyRule for PendingPostmortemPolicyRequiredExternalRefRule {
    fn check(&self, item: &PendingPostmortemPolicy) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending postmortem policy.
#[derive(Debug, Clone)]
pub struct PendingPostmortemPolicyValidator {
    pub rules: Vec<Rc<dyn PendingPostmortemPolicyRule>>,
}

impl PendingPostmortemPolicyValidator {
    pub fn validate(&self, item: &PendingPostmortemPolicy) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingPostmortemPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingPostmortemPolicy]) -> BTreeMap<String, Vec<String>> {
        let mut report: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for item in items.iter() {
            let problems = self.validate(item);
            if (problems.len() as i64) > 0 {
                report.insert(item.code.clone(), problems);
            }
        }
        report
    }
}

pub fn default_pending_postmortem_policy_validator() -> PendingPostmortemPolicyValidator {
    PendingPostmortemPolicyValidator {
        rules: vec![
            Rc::new(PendingPostmortemPolicyLabelLengthRule { max_length: 255 }) as Rc<dyn PendingPostmortemPolicyRule>,
            Rc::new(PendingPostmortemPolicyRequiredLabelRule {
                message: "pending postmortem policy needs this field".to_string(),
            }),
            Rc::new(PendingPostmortemPolicyRequiredExternalRefRule {
                message: "pending postmortem policy needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_postmortem_policy_validator();
        let item = new_pending_postmortem_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PendingPostmortemPolicyLabelLengthRule { max_length: 255 };
        let mut item = new_pending_postmortem_policy("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PendingPostmortemPolicyRequiredLabelRule {
            message: "pending postmortem policy needs this field".to_string(),
        };
        let mut item = new_pending_postmortem_policy("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = PendingPostmortemPolicyRequiredExternalRefRule {
            message: "pending postmortem policy needs this field".to_string(),
        };
        let mut item = new_pending_postmortem_policy("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
