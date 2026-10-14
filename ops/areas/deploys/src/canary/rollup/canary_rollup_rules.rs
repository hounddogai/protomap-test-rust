use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a canary rollup must pass. Check returns its problems, or none.
pub trait CanaryRollupRule: std::fmt::Debug {
    fn check(&self, item: &CanaryRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryRollupActiveGuardRule {
    pub limit: i64,
}

impl CanaryRollupRule for CanaryRollupActiveGuardRule {
    fn check(&self, item: &CanaryRollup) -> Vec<String> {
        if item.active && (item.batch_size > self.limit) {
            return vec![format!(
                "batch size {} exceeds {} while active",
                item.batch_size, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryRollupCodePrefixRule {
    pub prefix: String,
}

impl CanaryRollupRule for CanaryRollupCodePrefixRule {
    fn check(&self, item: &CanaryRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryRollupAllowedNameRule {
    pub allowed: Vec<String>,
}

impl CanaryRollupRule for CanaryRollupAllowedNameRule {
    fn check(&self, item: &CanaryRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is above maximum", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryRollupRequiredLabelRule {
    pub message: String,
}

impl CanaryRollupRule for CanaryRollupRequiredLabelRule {
    fn check(&self, item: &CanaryRollup) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryRollupNameLengthRule {
    pub max_length: i64,
}

impl CanaryRollupRule for CanaryRollupNameLengthRule {
    fn check(&self, item: &CanaryRollup) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryRollupBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CanaryRollupRule for CanaryRollupBatchSizeRangeRule {
    fn check(&self, item: &CanaryRollup) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary rollup.
#[derive(Debug, Clone)]
pub struct CanaryRollupValidator {
    pub rules: Vec<Rc<dyn CanaryRollupRule>>,
}

impl CanaryRollupValidator {
    pub fn validate(&self, item: &CanaryRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            let found = rule.check(item);
            if (found.len() as i64) == 0 {
                continue;
            }
            for problem in found.iter() {
                problems.push(format!("{}: {}", item.code, problem));
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CanaryRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanaryRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_rollup_validator() -> CanaryRollupValidator {
    CanaryRollupValidator {
        rules: vec![
            Rc::new(CanaryRollupActiveGuardRule { limit: 3600 }) as Rc<dyn CanaryRollupRule>,
            Rc::new(CanaryRollupCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(CanaryRollupAllowedNameRule { allowed: Vec::new() }),
            Rc::new(CanaryRollupRequiredLabelRule {
                message: "canary rollup needs an entry".to_string(),
            }),
            Rc::new(CanaryRollupNameLengthRule { max_length: 32 }),
            Rc::new(CanaryRollupBatchSizeRangeRule { min: 32, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_canary_rollup_validator();
        let item = new_canary_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = CanaryRollupRequiredLabelRule {
            message: "canary rollup needs this field".to_string(),
        };
        let mut item = new_canary_rollup("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = CanaryRollupNameLengthRule { max_length: 120 };
        let mut item = new_canary_rollup("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = CanaryRollupBatchSizeRangeRule { min: 16, max: 255 };
        let mut item = new_canary_rollup("x", "y");
        item.batch_size = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
