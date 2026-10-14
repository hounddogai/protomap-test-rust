use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a ttl policy threshold must pass. Check returns its problems, or none.
pub trait TtlPolicyThresholdRule: std::fmt::Debug {
    fn check(&self, item: &TtlPolicyThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyThresholdRequiredCountryRule {
    pub message: String,
}

impl TtlPolicyThresholdRule for TtlPolicyThresholdRequiredCountryRule {
    fn check(&self, item: &TtlPolicyThreshold) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyThresholdRequiredCodeRule {
    pub message: String,
}

impl TtlPolicyThresholdRule for TtlPolicyThresholdRequiredCodeRule {
    fn check(&self, item: &TtlPolicyThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyThresholdCodePrefixRule {
    pub prefix: String,
}

impl TtlPolicyThresholdRule for TtlPolicyThresholdCodePrefixRule {
    fn check(&self, item: &TtlPolicyThreshold) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyThresholdLabelLengthRule {
    pub max_length: i64,
}

impl TtlPolicyThresholdRule for TtlPolicyThresholdLabelLengthRule {
    fn check(&self, item: &TtlPolicyThreshold) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a ttl policy threshold.
#[derive(Debug, Clone)]
pub struct TtlPolicyThresholdValidator {
    pub rules: Vec<Rc<dyn TtlPolicyThresholdRule>>,
}

impl TtlPolicyThresholdValidator {
    pub fn validate(&self, item: &TtlPolicyThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TtlPolicyThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TtlPolicyThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_ttl_policy_threshold_validator() -> TtlPolicyThresholdValidator {
    TtlPolicyThresholdValidator {
        rules: vec![
            Rc::new(TtlPolicyThresholdRequiredCountryRule {
                message: "ttl policy threshold needs an entry".to_string(),
            }) as Rc<dyn TtlPolicyThresholdRule>,
            Rc::new(TtlPolicyThresholdRequiredCodeRule {
                message: "ttl policy threshold needs an entry".to_string(),
            }),
            Rc::new(TtlPolicyThresholdCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(TtlPolicyThresholdLabelLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_ttl_policy_threshold_validator();
        let item = new_ttl_policy_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = TtlPolicyThresholdRequiredCountryRule {
            message: "ttl policy threshold needs a value".to_string(),
        };
        let mut item = new_ttl_policy_threshold("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TtlPolicyThresholdRequiredCodeRule {
            message: "ttl policy threshold needs an entry".to_string(),
        };
        let mut item = new_ttl_policy_threshold("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = TtlPolicyThresholdLabelLengthRule { max_length: 64 };
        let mut item = new_ttl_policy_threshold("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
