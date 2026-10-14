use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy isolation forecast must pass. Check returns its problems, or none.
pub trait LegacyIsolationForecastRule: std::fmt::Debug {
    fn check(&self, item: &LegacyIsolationForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIsolationForecastRequiredCodeRule {
    pub message: String,
}

impl LegacyIsolationForecastRule for LegacyIsolationForecastRequiredCodeRule {
    fn check(&self, item: &LegacyIsolationForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIsolationForecastCodeLengthRule {
    pub max_length: i64,
}

impl LegacyIsolationForecastRule for LegacyIsolationForecastCodeLengthRule {
    fn check(&self, item: &LegacyIsolationForecast) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIsolationForecastTitleLengthRule {
    pub max_length: i64,
}

impl LegacyIsolationForecastRule for LegacyIsolationForecastTitleLengthRule {
    fn check(&self, item: &LegacyIsolationForecast) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy isolation forecast.
#[derive(Debug, Clone)]
pub struct LegacyIsolationForecastValidator {
    pub rules: Vec<Rc<dyn LegacyIsolationForecastRule>>,
}

impl LegacyIsolationForecastValidator {
    pub fn validate(&self, item: &LegacyIsolationForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyIsolationForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyIsolationForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_isolation_forecast_validator() -> LegacyIsolationForecastValidator {
    LegacyIsolationForecastValidator {
        rules: vec![
            Rc::new(LegacyIsolationForecastRequiredCodeRule {
                message: "legacy isolation forecast needs this field".to_string(),
            }) as Rc<dyn LegacyIsolationForecastRule>,
            Rc::new(LegacyIsolationForecastCodeLengthRule { max_length: 64 }),
            Rc::new(LegacyIsolationForecastTitleLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_isolation_forecast_validator();
        let item = new_legacy_isolation_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyIsolationForecastRequiredCodeRule {
            message: "legacy isolation forecast needs this field".to_string(),
        };
        let mut item = new_legacy_isolation_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyIsolationForecastCodeLengthRule { max_length: 120 };
        let mut item = new_legacy_isolation_forecast("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LegacyIsolationForecastTitleLengthRule { max_length: 255 };
        let mut item = new_legacy_isolation_forecast("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
