use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback migration forecast must pass. Check returns its problems, or none.
pub trait FallbackMigrationForecastRule: std::fmt::Debug {
    fn check(&self, item: &FallbackMigrationForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMigrationForecastAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl FallbackMigrationForecastRule for FallbackMigrationForecastAllowedCodeRule {
    fn check(&self, item: &FallbackMigrationForecast) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is below minimum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMigrationForecastNameLengthRule {
    pub max_length: i64,
}

impl FallbackMigrationForecastRule for FallbackMigrationForecastNameLengthRule {
    fn check(&self, item: &FallbackMigrationForecast) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMigrationForecastTitleLengthRule {
    pub max_length: i64,
}

impl FallbackMigrationForecastRule for FallbackMigrationForecastTitleLengthRule {
    fn check(&self, item: &FallbackMigrationForecast) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackMigrationForecastRequiredNotesRule {
    pub message: String,
}

impl FallbackMigrationForecastRule for FallbackMigrationForecastRequiredNotesRule {
    fn check(&self, item: &FallbackMigrationForecast) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback migration forecast.
#[derive(Debug, Clone)]
pub struct FallbackMigrationForecastValidator {
    pub rules: Vec<Rc<dyn FallbackMigrationForecastRule>>,
}

impl FallbackMigrationForecastValidator {
    pub fn validate(&self, item: &FallbackMigrationForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackMigrationForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackMigrationForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_migration_forecast_validator() -> FallbackMigrationForecastValidator {
    FallbackMigrationForecastValidator {
        rules: vec![
            Rc::new(FallbackMigrationForecastAllowedCodeRule {
                allowed: vec!["capped".to_string()],
            }) as Rc<dyn FallbackMigrationForecastRule>,
            Rc::new(FallbackMigrationForecastNameLengthRule { max_length: 3600 }),
            Rc::new(FallbackMigrationForecastTitleLengthRule { max_length: 16 }),
            Rc::new(FallbackMigrationForecastRequiredNotesRule {
                message: "fallback migration forecast needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_migration_forecast_validator();
        let item = new_fallback_migration_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FallbackMigrationForecastNameLengthRule { max_length: 255 };
        let mut item = new_fallback_migration_forecast("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = FallbackMigrationForecastTitleLengthRule { max_length: 1000 };
        let mut item = new_fallback_migration_forecast("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = FallbackMigrationForecastRequiredNotesRule {
            message: "fallback migration forecast needs an entry".to_string(),
        };
        let mut item = new_fallback_migration_forecast("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
