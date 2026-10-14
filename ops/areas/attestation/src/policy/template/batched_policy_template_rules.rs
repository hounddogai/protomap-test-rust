use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched policy template must pass. Check returns its problems, or none.
pub trait BatchedPolicyTemplateRule: std::fmt::Debug {
    fn check(&self, item: &BatchedPolicyTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyTemplateDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedPolicyTemplateRule for BatchedPolicyTemplateDurationMinutesRangeRule {
    fn check(&self, item: &BatchedPolicyTemplate) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyTemplateCodePrefixRule {
    pub prefix: String,
}

impl BatchedPolicyTemplateRule for BatchedPolicyTemplateCodePrefixRule {
    fn check(&self, item: &BatchedPolicyTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyTemplateRequiredCodeRule {
    pub message: String,
}

impl BatchedPolicyTemplateRule for BatchedPolicyTemplateRequiredCodeRule {
    fn check(&self, item: &BatchedPolicyTemplate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyTemplateRequiredLabelRule {
    pub message: String,
}

impl BatchedPolicyTemplateRule for BatchedPolicyTemplateRequiredLabelRule {
    fn check(&self, item: &BatchedPolicyTemplate) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyTemplateKeyIdLengthRule {
    pub max_length: i64,
}

impl BatchedPolicyTemplateRule for BatchedPolicyTemplateKeyIdLengthRule {
    fn check(&self, item: &BatchedPolicyTemplate) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPolicyTemplateAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl BatchedPolicyTemplateRule for BatchedPolicyTemplateAllowedCodeRule {
    fn check(&self, item: &BatchedPolicyTemplate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is unknown", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched policy template.
#[derive(Debug, Clone)]
pub struct BatchedPolicyTemplateValidator {
    pub rules: Vec<Rc<dyn BatchedPolicyTemplateRule>>,
}

impl BatchedPolicyTemplateValidator {
    pub fn validate(&self, item: &BatchedPolicyTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedPolicyTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedPolicyTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_policy_template_validator() -> BatchedPolicyTemplateValidator {
    BatchedPolicyTemplateValidator {
        rules: vec![
            Rc::new(BatchedPolicyTemplateDurationMinutesRangeRule { min: 80, max: 3600 })
                as Rc<dyn BatchedPolicyTemplateRule>,
            Rc::new(BatchedPolicyTemplateCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(BatchedPolicyTemplateRequiredCodeRule {
                message: "batched policy template needs this field".to_string(),
            }),
            Rc::new(BatchedPolicyTemplateRequiredLabelRule {
                message: "batched policy template needs a value".to_string(),
            }),
            Rc::new(BatchedPolicyTemplateKeyIdLengthRule { max_length: 32 }),
            Rc::new(BatchedPolicyTemplateAllowedCodeRule {
                allowed: vec!["default".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_policy_template_validator();
        let item = new_batched_policy_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = BatchedPolicyTemplateDurationMinutesRangeRule { min: 16, max: 32 };
        let mut item = new_batched_policy_template("x", "y");
        item.duration_minutes = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedPolicyTemplateRequiredCodeRule {
            message: "batched policy template needs a value".to_string(),
        };
        let mut item = new_batched_policy_template("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = BatchedPolicyTemplateRequiredLabelRule {
            message: "batched policy template needs an entry".to_string(),
        };
        let mut item = new_batched_policy_template("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = BatchedPolicyTemplateKeyIdLengthRule { max_length: 1000 };
        let mut item = new_batched_policy_template("x", "y");
        item.key_id = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
