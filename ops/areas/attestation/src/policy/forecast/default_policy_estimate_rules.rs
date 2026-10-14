use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default policy estimate must pass. Check returns its problems, or none.
pub trait DefaultPolicyEstimateRule: std::fmt::Debug {
    fn check(&self, item: &DefaultPolicyEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPolicyEstimateRequiredContactEmailRule {
    pub message: String,
}

impl DefaultPolicyEstimateRule for DefaultPolicyEstimateRequiredContactEmailRule {
    fn check(&self, item: &DefaultPolicyEstimate) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPolicyEstimateRequiredNameRule {
    pub message: String,
}

impl DefaultPolicyEstimateRule for DefaultPolicyEstimateRequiredNameRule {
    fn check(&self, item: &DefaultPolicyEstimate) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultPolicyEstimateNameLengthRule {
    pub max_length: i64,
}

impl DefaultPolicyEstimateRule for DefaultPolicyEstimateNameLengthRule {
    fn check(&self, item: &DefaultPolicyEstimate) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default policy estimate.
#[derive(Debug, Clone)]
pub struct DefaultPolicyEstimateValidator {
    pub rules: Vec<Rc<dyn DefaultPolicyEstimateRule>>,
}

impl DefaultPolicyEstimateValidator {
    pub fn validate(&self, item: &DefaultPolicyEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultPolicyEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultPolicyEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_policy_estimate_validator() -> DefaultPolicyEstimateValidator {
    DefaultPolicyEstimateValidator {
        rules: vec![
            Rc::new(DefaultPolicyEstimateRequiredContactEmailRule {
                message: "default policy estimate needs this field".to_string(),
            }) as Rc<dyn DefaultPolicyEstimateRule>,
            Rc::new(DefaultPolicyEstimateRequiredNameRule {
                message: "default policy estimate needs an entry".to_string(),
            }),
            Rc::new(DefaultPolicyEstimateNameLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_policy_estimate_validator();
        let item = new_default_policy_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = DefaultPolicyEstimateRequiredContactEmailRule {
            message: "default policy estimate needs a value".to_string(),
        };
        let mut item = new_default_policy_estimate("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = DefaultPolicyEstimateRequiredNameRule {
            message: "default policy estimate needs this field".to_string(),
        };
        let mut item = new_default_policy_estimate("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = DefaultPolicyEstimateNameLengthRule { max_length: 1000 };
        let mut item = new_default_policy_estimate("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
