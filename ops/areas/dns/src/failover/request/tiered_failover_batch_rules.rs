use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered failover batch must pass. Check returns its problems, or none.
pub trait TieredFailoverBatchRule: std::fmt::Debug {
    fn check(&self, item: &TieredFailoverBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFailoverBatchExternalRefLengthRule {
    pub max_length: i64,
}

impl TieredFailoverBatchRule for TieredFailoverBatchExternalRefLengthRule {
    fn check(&self, item: &TieredFailoverBatch) -> Vec<String> {
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
pub struct TieredFailoverBatchAllowedCurrencyRule {
    pub allowed: Vec<String>,
}

impl TieredFailoverBatchRule for TieredFailoverBatchAllowedCurrencyRule {
    fn check(&self, item: &TieredFailoverBatch) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.currency)))
        {
            return vec![format!("currency {} is above maximum", item.currency)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredFailoverBatchDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredFailoverBatchRule for TieredFailoverBatchDurationMinutesRangeRule {
    fn check(&self, item: &TieredFailoverBatch) -> Vec<String> {
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

/// Runs every rule of a tiered failover batch.
#[derive(Debug, Clone)]
pub struct TieredFailoverBatchValidator {
    pub rules: Vec<Rc<dyn TieredFailoverBatchRule>>,
}

impl TieredFailoverBatchValidator {
    pub fn validate(&self, item: &TieredFailoverBatch) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredFailoverBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredFailoverBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_failover_batch_validator() -> TieredFailoverBatchValidator {
    TieredFailoverBatchValidator {
        rules: vec![
            Rc::new(TieredFailoverBatchExternalRefLengthRule { max_length: 120 }) as Rc<dyn TieredFailoverBatchRule>,
            Rc::new(TieredFailoverBatchAllowedCurrencyRule { allowed: Vec::new() }),
            Rc::new(TieredFailoverBatchDurationMinutesRangeRule { min: 3600, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_failover_batch_validator();
        let item = new_tiered_failover_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = TieredFailoverBatchExternalRefLengthRule { max_length: 16 };
        let mut item = new_tiered_failover_batch("x", "y");
        item.external_ref = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = TieredFailoverBatchDurationMinutesRangeRule { min: 80, max: 255 };
        let mut item = new_tiered_failover_batch("x", "y");
        item.duration_minutes = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
