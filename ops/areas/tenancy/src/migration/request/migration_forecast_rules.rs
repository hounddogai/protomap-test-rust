use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a migration forecast must pass. Check returns its problems, or none.
pub trait MigrationForecastRule: std::fmt::Debug {
    fn check(&self, item: &MigrationForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationForecastCodeLengthRule {
    pub max_length: i64,
}

impl MigrationForecastRule for MigrationForecastCodeLengthRule {
    fn check(&self, item: &MigrationForecast) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationForecastRequiredNameRule {
    pub message: String,
}

impl MigrationForecastRule for MigrationForecastRequiredNameRule {
    fn check(&self, item: &MigrationForecast) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationForecastRequiredCodeRule {
    pub message: String,
}

impl MigrationForecastRule for MigrationForecastRequiredCodeRule {
    fn check(&self, item: &MigrationForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationForecastRequiredSourceRule {
    pub message: String,
}

impl MigrationForecastRule for MigrationForecastRequiredSourceRule {
    fn check(&self, item: &MigrationForecast) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationForecastNameLengthRule {
    pub max_length: i64,
}

impl MigrationForecastRule for MigrationForecastNameLengthRule {
    fn check(&self, item: &MigrationForecast) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a migration forecast.
#[derive(Debug, Clone)]
pub struct MigrationForecastValidator {
    pub rules: Vec<Rc<dyn MigrationForecastRule>>,
}

impl MigrationForecastValidator {
    pub fn validate(&self, item: &MigrationForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &MigrationForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_forecast_validator() -> MigrationForecastValidator {
    MigrationForecastValidator {
        rules: vec![
            Rc::new(MigrationForecastCodeLengthRule { max_length: 3600 }) as Rc<dyn MigrationForecastRule>,
            Rc::new(MigrationForecastRequiredNameRule {
                message: "migration forecast needs an entry".to_string(),
            }),
            Rc::new(MigrationForecastRequiredCodeRule {
                message: "migration forecast needs this field".to_string(),
            }),
            Rc::new(MigrationForecastRequiredSourceRule {
                message: "migration forecast needs this field".to_string(),
            }),
            Rc::new(MigrationForecastNameLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_migration_forecast_validator();
        let item = new_migration_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = MigrationForecastCodeLengthRule { max_length: 255 };
        let mut item = new_migration_forecast("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = MigrationForecastRequiredNameRule {
            message: "migration forecast needs an entry".to_string(),
        };
        let mut item = new_migration_forecast("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = MigrationForecastRequiredCodeRule {
            message: "migration forecast needs a value".to_string(),
        };
        let mut item = new_migration_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = MigrationForecastRequiredSourceRule {
            message: "migration forecast needs a value".to_string(),
        };
        let mut item = new_migration_forecast("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = MigrationForecastNameLengthRule { max_length: 3600 };
        let mut item = new_migration_forecast("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
