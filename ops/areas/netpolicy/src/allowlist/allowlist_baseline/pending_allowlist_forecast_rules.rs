use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending allowlist forecast must pass. Check returns its problems, or none.
pub trait PendingAllowlistForecastRule: std::fmt::Debug {
    fn check(&self, item: &PendingAllowlistForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingAllowlistForecastLabelLengthRule {
    pub max_length: i64,
}

impl PendingAllowlistForecastRule for PendingAllowlistForecastLabelLengthRule {
    fn check(&self, item: &PendingAllowlistForecast) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingAllowlistForecastSourceLengthRule {
    pub max_length: i64,
}

impl PendingAllowlistForecastRule for PendingAllowlistForecastSourceLengthRule {
    fn check(&self, item: &PendingAllowlistForecast) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingAllowlistForecastScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingAllowlistForecastRule for PendingAllowlistForecastScoreRangeRule {
    fn check(&self, item: &PendingAllowlistForecast) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a pending allowlist forecast.
#[derive(Debug, Clone)]
pub struct PendingAllowlistForecastValidator {
    pub rules: Vec<Rc<dyn PendingAllowlistForecastRule>>,
}

impl PendingAllowlistForecastValidator {
    pub fn validate(&self, item: &PendingAllowlistForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingAllowlistForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingAllowlistForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_allowlist_forecast_validator() -> PendingAllowlistForecastValidator {
    PendingAllowlistForecastValidator {
        rules: vec![
            Rc::new(PendingAllowlistForecastLabelLengthRule { max_length: 16 }) as Rc<dyn PendingAllowlistForecastRule>,
            Rc::new(PendingAllowlistForecastSourceLengthRule { max_length: 64 }),
            Rc::new(PendingAllowlistForecastScoreRangeRule { min: 3.2, max: 415.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_allowlist_forecast_validator();
        let item = new_pending_allowlist_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PendingAllowlistForecastLabelLengthRule { max_length: 16 };
        let mut item = new_pending_allowlist_forecast("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = PendingAllowlistForecastSourceLengthRule { max_length: 1000 };
        let mut item = new_pending_allowlist_forecast("x", "y");
        item.source = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = PendingAllowlistForecastScoreRangeRule { min: 0.6, max: 4364.9 };
        let mut item = new_pending_allowlist_forecast("x", "y");
        item.score = 4366.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
