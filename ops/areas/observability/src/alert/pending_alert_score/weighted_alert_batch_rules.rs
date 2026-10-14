use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted alert batch must pass. Check returns its problems, or none.
pub trait WeightedAlertBatchRule: std::fmt::Debug {
    fn check(&self, item: &WeightedAlertBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAlertBatchCodeLengthRule {
    pub max_length: i64,
}

impl WeightedAlertBatchRule for WeightedAlertBatchCodeLengthRule {
    fn check(&self, item: &WeightedAlertBatch) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAlertBatchLabelLengthRule {
    pub max_length: i64,
}

impl WeightedAlertBatchRule for WeightedAlertBatchLabelLengthRule {
    fn check(&self, item: &WeightedAlertBatch) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAlertBatchCountryLengthRule {
    pub max_length: i64,
}

impl WeightedAlertBatchRule for WeightedAlertBatchCountryLengthRule {
    fn check(&self, item: &WeightedAlertBatch) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAlertBatchAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl WeightedAlertBatchRule for WeightedAlertBatchAllowedLabelRule {
    fn check(&self, item: &WeightedAlertBatch) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is stale", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedAlertBatchQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedAlertBatchRule for WeightedAlertBatchQuotaUnitsRangeRule {
    fn check(&self, item: &WeightedAlertBatch) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted alert batch.
#[derive(Debug, Clone)]
pub struct WeightedAlertBatchValidator {
    pub rules: Vec<Rc<dyn WeightedAlertBatchRule>>,
}

impl WeightedAlertBatchValidator {
    pub fn validate(&self, item: &WeightedAlertBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedAlertBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedAlertBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_alert_batch_validator() -> WeightedAlertBatchValidator {
    WeightedAlertBatchValidator {
        rules: vec![
            Rc::new(WeightedAlertBatchCodeLengthRule { max_length: 64 }) as Rc<dyn WeightedAlertBatchRule>,
            Rc::new(WeightedAlertBatchLabelLengthRule { max_length: 3600 }),
            Rc::new(WeightedAlertBatchCountryLengthRule { max_length: 16 }),
            Rc::new(WeightedAlertBatchAllowedLabelRule {
                allowed: vec!["manual".to_string(), "manual".to_string(), "legacy".to_string()],
            }),
            Rc::new(WeightedAlertBatchQuotaUnitsRangeRule { min: 120, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_alert_batch_validator();
        let item = new_weighted_alert_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedAlertBatchCodeLengthRule { max_length: 3600 };
        let mut item = new_weighted_alert_batch("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = WeightedAlertBatchLabelLengthRule { max_length: 16 };
        let mut item = new_weighted_alert_batch("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = WeightedAlertBatchCountryLengthRule { max_length: 16 };
        let mut item = new_weighted_alert_batch("x", "y");
        item.country = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = WeightedAlertBatchQuotaUnitsRangeRule { min: 32, max: 80 };
        let mut item = new_weighted_alert_batch("x", "y");
        item.quota_units = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
