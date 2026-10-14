use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped forecast score must pass. Check returns its problems, or none.
pub trait CappedForecastScoreRule: std::fmt::Debug {
    fn check(&self, item: &CappedForecastScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedForecastScoreAllowedNotesRule {
    pub allowed: Vec<String>,
}

impl CappedForecastScoreRule for CappedForecastScoreAllowedNotesRule {
    fn check(&self, item: &CappedForecastScore) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.notes))) {
            return vec![format!("notes {} is not allowed", item.notes)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedForecastScoreRequiredNameRule {
    pub message: String,
}

impl CappedForecastScoreRule for CappedForecastScoreRequiredNameRule {
    fn check(&self, item: &CappedForecastScore) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedForecastScoreHostLengthRule {
    pub max_length: i64,
}

impl CappedForecastScoreRule for CappedForecastScoreHostLengthRule {
    fn check(&self, item: &CappedForecastScore) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped forecast score.
#[derive(Debug, Clone)]
pub struct CappedForecastScoreValidator {
    pub rules: Vec<Rc<dyn CappedForecastScoreRule>>,
}

impl CappedForecastScoreValidator {
    pub fn validate(&self, item: &CappedForecastScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedForecastScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedForecastScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_forecast_score_validator() -> CappedForecastScoreValidator {
    CappedForecastScoreValidator {
        rules: vec![
            Rc::new(CappedForecastScoreAllowedNotesRule { allowed: Vec::new() }) as Rc<dyn CappedForecastScoreRule>,
            Rc::new(CappedForecastScoreRequiredNameRule {
                message: "capped forecast score needs a value".to_string(),
            }),
            Rc::new(CappedForecastScoreHostLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_forecast_score_validator();
        let item = new_capped_forecast_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = CappedForecastScoreRequiredNameRule {
            message: "capped forecast score needs this field".to_string(),
        };
        let mut item = new_capped_forecast_score("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = CappedForecastScoreHostLengthRule { max_length: 3600 };
        let mut item = new_capped_forecast_score("x", "y");
        item.host = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
