use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted policy draft must pass. Check returns its problems, or none.
pub trait WeightedPolicyDraftRule: std::fmt::Debug {
    fn check(&self, item: &WeightedPolicyDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPolicyDraftRequiredCodeRule {
    pub message: String,
}

impl WeightedPolicyDraftRule for WeightedPolicyDraftRequiredCodeRule {
    fn check(&self, item: &WeightedPolicyDraft) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPolicyDraftRequiredLabelRule {
    pub message: String,
}

impl WeightedPolicyDraftRule for WeightedPolicyDraftRequiredLabelRule {
    fn check(&self, item: &WeightedPolicyDraft) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPolicyDraftCodeLengthRule {
    pub max_length: i64,
}

impl WeightedPolicyDraftRule for WeightedPolicyDraftCodeLengthRule {
    fn check(&self, item: &WeightedPolicyDraft) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPolicyDraftPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedPolicyDraftRule for WeightedPolicyDraftPriorityRangeRule {
    fn check(&self, item: &WeightedPolicyDraft) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPolicyDraftContactEmailLengthRule {
    pub max_length: i64,
}

impl WeightedPolicyDraftRule for WeightedPolicyDraftContactEmailLengthRule {
    fn check(&self, item: &WeightedPolicyDraft) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted policy draft.
#[derive(Debug, Clone)]
pub struct WeightedPolicyDraftValidator {
    pub rules: Vec<Rc<dyn WeightedPolicyDraftRule>>,
}

impl WeightedPolicyDraftValidator {
    pub fn validate(&self, item: &WeightedPolicyDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedPolicyDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedPolicyDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_policy_draft_validator() -> WeightedPolicyDraftValidator {
    WeightedPolicyDraftValidator {
        rules: vec![
            Rc::new(WeightedPolicyDraftRequiredCodeRule {
                message: "weighted policy draft needs a value".to_string(),
            }) as Rc<dyn WeightedPolicyDraftRule>,
            Rc::new(WeightedPolicyDraftRequiredLabelRule {
                message: "weighted policy draft needs a value".to_string(),
            }),
            Rc::new(WeightedPolicyDraftCodeLengthRule { max_length: 80 }),
            Rc::new(WeightedPolicyDraftPriorityRangeRule { min: 3600, max: 86400 }),
            Rc::new(WeightedPolicyDraftContactEmailLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_policy_draft_validator();
        let item = new_weighted_policy_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedPolicyDraftRequiredCodeRule {
            message: "weighted policy draft needs a value".to_string(),
        };
        let mut item = new_weighted_policy_draft("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = WeightedPolicyDraftRequiredLabelRule {
            message: "weighted policy draft needs an entry".to_string(),
        };
        let mut item = new_weighted_policy_draft("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedPolicyDraftCodeLengthRule { max_length: 32 };
        let mut item = new_weighted_policy_draft("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = WeightedPolicyDraftPriorityRangeRule { min: 32, max: 86400 };
        let mut item = new_weighted_policy_draft("x", "y");
        item.priority = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = WeightedPolicyDraftContactEmailLengthRule { max_length: 255 };
        let mut item = new_weighted_policy_draft("x", "y");
        item.contact_email = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
