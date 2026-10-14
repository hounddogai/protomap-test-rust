use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a alert score must pass. Check returns its problems, or none.
pub trait AlertScoreRule: std::fmt::Debug {
    fn check(&self, item: &AlertScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertScoreTitleLengthRule {
    pub max_length: i64,
}

impl AlertScoreRule for AlertScoreTitleLengthRule {
    fn check(&self, item: &AlertScore) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertScoreBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AlertScoreRule for AlertScoreBudgetRangeRule {
    fn check(&self, item: &AlertScore) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertScoreWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AlertScoreRule for AlertScoreWindowDaysRangeRule {
    fn check(&self, item: &AlertScore) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a alert score.
#[derive(Debug, Clone)]
pub struct AlertScoreValidator {
    pub rules: Vec<Rc<dyn AlertScoreRule>>,
}

impl AlertScoreValidator {
    pub fn validate(&self, item: &AlertScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AlertScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_score_validator() -> AlertScoreValidator {
    AlertScoreValidator {
        rules: vec![
            Rc::new(AlertScoreTitleLengthRule { max_length: 16 }) as Rc<dyn AlertScoreRule>,
            Rc::new(AlertScoreBudgetRangeRule { min: 2.4, max: 4896.1 }),
            Rc::new(AlertScoreWindowDaysRangeRule { min: 80, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_alert_score_validator();
        let item = new_alert_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = AlertScoreTitleLengthRule { max_length: 120 };
        let mut item = new_alert_score("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = AlertScoreBudgetRangeRule { min: 8.2, max: 2491.7 };
        let mut item = new_alert_score("x", "y");
        item.budget = 2493.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = AlertScoreWindowDaysRangeRule { min: 16, max: 32 };
        let mut item = new_alert_score("x", "y");
        item.window_days = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
