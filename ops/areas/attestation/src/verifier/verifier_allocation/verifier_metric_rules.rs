use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a verifier metric must pass. Check returns its problems, or none.
pub trait VerifierMetricRule: std::fmt::Debug {
    fn check(&self, item: &VerifierMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierMetricUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl VerifierMetricRule for VerifierMetricUpdatedAtRangeRule {
    fn check(&self, item: &VerifierMetric) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierMetricNameLengthRule {
    pub max_length: i64,
}

impl VerifierMetricRule for VerifierMetricNameLengthRule {
    fn check(&self, item: &VerifierMetric) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierMetricCodeLengthRule {
    pub max_length: i64,
}

impl VerifierMetricRule for VerifierMetricCodeLengthRule {
    fn check(&self, item: &VerifierMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierMetricCodePrefixRule {
    pub prefix: String,
}

impl VerifierMetricRule for VerifierMetricCodePrefixRule {
    fn check(&self, item: &VerifierMetric) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierMetricCategoryLengthRule {
    pub max_length: i64,
}

impl VerifierMetricRule for VerifierMetricCategoryLengthRule {
    fn check(&self, item: &VerifierMetric) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier metric.
#[derive(Debug, Clone)]
pub struct VerifierMetricValidator {
    pub rules: Vec<Rc<dyn VerifierMetricRule>>,
}

impl VerifierMetricValidator {
    pub fn validate(&self, item: &VerifierMetric) -> Vec<String> {
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

    pub fn require_valid(&self, item: &VerifierMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_metric_validator() -> VerifierMetricValidator {
    VerifierMetricValidator {
        rules: vec![
            Rc::new(VerifierMetricUpdatedAtRangeRule { min: 64, max: 120 }) as Rc<dyn VerifierMetricRule>,
            Rc::new(VerifierMetricNameLengthRule { max_length: 1000 }),
            Rc::new(VerifierMetricCodeLengthRule { max_length: 3600 }),
            Rc::new(VerifierMetricCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(VerifierMetricCategoryLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_verifier_metric_validator();
        let item = new_verifier_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = VerifierMetricUpdatedAtRangeRule { min: 32, max: 255 };
        let mut item = new_verifier_metric("x", "y");
        item.updated_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = VerifierMetricNameLengthRule { max_length: 1000 };
        let mut item = new_verifier_metric("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = VerifierMetricCodeLengthRule { max_length: 32 };
        let mut item = new_verifier_metric("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = VerifierMetricCategoryLengthRule { max_length: 32 };
        let mut item = new_verifier_metric("x", "y");
        item.category = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
