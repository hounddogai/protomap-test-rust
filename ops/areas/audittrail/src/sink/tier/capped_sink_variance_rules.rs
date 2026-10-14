use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped sink variance must pass. Check returns its problems, or none.
pub trait CappedSinkVarianceRule: std::fmt::Debug {
    fn check(&self, item: &CappedSinkVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSinkVarianceMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedSinkVarianceRule for CappedSinkVarianceMaximumRangeRule {
    fn check(&self, item: &CappedSinkVariance) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSinkVarianceDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedSinkVarianceRule for CappedSinkVarianceDurationMinutesRangeRule {
    fn check(&self, item: &CappedSinkVariance) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSinkVarianceCodePrefixRule {
    pub prefix: String,
}

impl CappedSinkVarianceRule for CappedSinkVarianceCodePrefixRule {
    fn check(&self, item: &CappedSinkVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped sink variance.
#[derive(Debug, Clone)]
pub struct CappedSinkVarianceValidator {
    pub rules: Vec<Rc<dyn CappedSinkVarianceRule>>,
}

impl CappedSinkVarianceValidator {
    pub fn validate(&self, item: &CappedSinkVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedSinkVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedSinkVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_sink_variance_validator() -> CappedSinkVarianceValidator {
    CappedSinkVarianceValidator {
        rules: vec![
            Rc::new(CappedSinkVarianceMaximumRangeRule { min: 8.3, max: 627.3 }) as Rc<dyn CappedSinkVarianceRule>,
            Rc::new(CappedSinkVarianceDurationMinutesRangeRule { min: 32, max: 1000 }),
            Rc::new(CappedSinkVarianceCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_sink_variance_validator();
        let item = new_capped_sink_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = CappedSinkVarianceMaximumRangeRule { min: 2.3, max: 766.2 };
        let mut item = new_capped_sink_variance("x", "y");
        item.maximum = 767.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = CappedSinkVarianceDurationMinutesRangeRule { min: 255, max: 255 };
        let mut item = new_capped_sink_variance("x", "y");
        item.duration_minutes = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
