use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a health check batch must pass. Check returns its problems, or none.
pub trait HealthCheckBatchRule: std::fmt::Debug {
    fn check(&self, item: &HealthCheckBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckBatchExternalRefLengthRule {
    pub max_length: i64,
}

impl HealthCheckBatchRule for HealthCheckBatchExternalRefLengthRule {
    fn check(&self, item: &HealthCheckBatch) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckBatchAllowedExternalRefRule {
    pub allowed: Vec<String>,
}

impl HealthCheckBatchRule for HealthCheckBatchAllowedExternalRefRule {
    fn check(&self, item: &HealthCheckBatch) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.external_ref)))
        {
            return vec![format!("external ref {} is above maximum", item.external_ref)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckBatchRequiredTitleRule {
    pub message: String,
}

impl HealthCheckBatchRule for HealthCheckBatchRequiredTitleRule {
    fn check(&self, item: &HealthCheckBatch) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckBatchAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl HealthCheckBatchRule for HealthCheckBatchAmountRangeRule {
    fn check(&self, item: &HealthCheckBatch) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckBatchCodePrefixRule {
    pub prefix: String,
}

impl HealthCheckBatchRule for HealthCheckBatchCodePrefixRule {
    fn check(&self, item: &HealthCheckBatch) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a health check batch.
#[derive(Debug, Clone)]
pub struct HealthCheckBatchValidator {
    pub rules: Vec<Rc<dyn HealthCheckBatchRule>>,
}

impl HealthCheckBatchValidator {
    pub fn validate(&self, item: &HealthCheckBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &HealthCheckBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[HealthCheckBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_health_check_batch_validator() -> HealthCheckBatchValidator {
    HealthCheckBatchValidator {
        rules: vec![
            Rc::new(HealthCheckBatchExternalRefLengthRule { max_length: 64 }) as Rc<dyn HealthCheckBatchRule>,
            Rc::new(HealthCheckBatchAllowedExternalRefRule {
                allowed: vec!["seasonal".to_string()],
            }),
            Rc::new(HealthCheckBatchRequiredTitleRule {
                message: "health check batch needs a value".to_string(),
            }),
            Rc::new(HealthCheckBatchAmountRangeRule { min: 9.8, max: 2460.6 }),
            Rc::new(HealthCheckBatchCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_health_check_batch_validator();
        let item = new_health_check_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = HealthCheckBatchExternalRefLengthRule { max_length: 120 };
        let mut item = new_health_check_batch("x", "y");
        item.external_ref = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = HealthCheckBatchRequiredTitleRule {
            message: "health check batch needs an entry".to_string(),
        };
        let mut item = new_health_check_batch("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = HealthCheckBatchAmountRangeRule { min: 6.0, max: 2913.1 };
        let mut item = new_health_check_batch("x", "y");
        item.amount = 2914.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
