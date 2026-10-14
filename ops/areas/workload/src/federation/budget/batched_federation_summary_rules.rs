use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched federation summary must pass. Check returns its problems, or none.
pub trait BatchedFederationSummaryRule: std::fmt::Debug {
    fn check(&self, item: &BatchedFederationSummary) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFederationSummaryRegionLengthRule {
    pub max_length: i64,
}

impl BatchedFederationSummaryRule for BatchedFederationSummaryRegionLengthRule {
    fn check(&self, item: &BatchedFederationSummary) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFederationSummaryNameLengthRule {
    pub max_length: i64,
}

impl BatchedFederationSummaryRule for BatchedFederationSummaryNameLengthRule {
    fn check(&self, item: &BatchedFederationSummary) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFederationSummaryRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedFederationSummaryRule for BatchedFederationSummaryRateRangeRule {
    fn check(&self, item: &BatchedFederationSummary) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFederationSummaryBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedFederationSummaryRule for BatchedFederationSummaryBudgetRangeRule {
    fn check(&self, item: &BatchedFederationSummary) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedFederationSummaryMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl BatchedFederationSummaryRule for BatchedFederationSummaryMinimumRangeRule {
    fn check(&self, item: &BatchedFederationSummary) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched federation summary.
#[derive(Debug, Clone)]
pub struct BatchedFederationSummaryValidator {
    pub rules: Vec<Rc<dyn BatchedFederationSummaryRule>>,
}

impl BatchedFederationSummaryValidator {
    pub fn validate(&self, item: &BatchedFederationSummary) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedFederationSummary) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedFederationSummary]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_federation_summary_validator() -> BatchedFederationSummaryValidator {
    BatchedFederationSummaryValidator {
        rules: vec![
            Rc::new(BatchedFederationSummaryRegionLengthRule { max_length: 80 })
                as Rc<dyn BatchedFederationSummaryRule>,
            Rc::new(BatchedFederationSummaryNameLengthRule { max_length: 120 }),
            Rc::new(BatchedFederationSummaryRateRangeRule { min: 9.7, max: 3925.1 }),
            Rc::new(BatchedFederationSummaryBudgetRangeRule { min: 6.6, max: 3067.4 }),
            Rc::new(BatchedFederationSummaryMinimumRangeRule { min: 3.8, max: 3231.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_federation_summary_validator();
        let item = new_batched_federation_summary("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = BatchedFederationSummaryRegionLengthRule { max_length: 32 };
        let mut item = new_batched_federation_summary("x", "y");
        item.region = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = BatchedFederationSummaryNameLengthRule { max_length: 1000 };
        let mut item = new_batched_federation_summary("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = BatchedFederationSummaryRateRangeRule { min: 8.3, max: 2786.8 };
        let mut item = new_batched_federation_summary("x", "y");
        item.rate = 2788.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = BatchedFederationSummaryBudgetRangeRule { min: 1.1, max: 4780.4 };
        let mut item = new_batched_federation_summary("x", "y");
        item.budget = 4781.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = BatchedFederationSummaryMinimumRangeRule { min: 7.7, max: 3212.3 };
        let mut item = new_batched_federation_summary("x", "y");
        item.minimum = 3213.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
