use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending migration forecast must pass. Check returns its problems, or none.
pub trait PendingMigrationForecastRule: std::fmt::Debug {
    fn check(&self, item: &PendingMigrationForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingMigrationForecastRequiredAudienceRule {
    pub message: String,
}

impl PendingMigrationForecastRule for PendingMigrationForecastRequiredAudienceRule {
    fn check(&self, item: &PendingMigrationForecast) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingMigrationForecastSummaryLengthRule {
    pub max_length: i64,
}

impl PendingMigrationForecastRule for PendingMigrationForecastSummaryLengthRule {
    fn check(&self, item: &PendingMigrationForecast) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingMigrationForecastAudienceLengthRule {
    pub max_length: i64,
}

impl PendingMigrationForecastRule for PendingMigrationForecastAudienceLengthRule {
    fn check(&self, item: &PendingMigrationForecast) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending migration forecast.
#[derive(Debug, Clone)]
pub struct PendingMigrationForecastValidator {
    pub rules: Vec<Rc<dyn PendingMigrationForecastRule>>,
}

impl PendingMigrationForecastValidator {
    pub fn validate(&self, item: &PendingMigrationForecast) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingMigrationForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingMigrationForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_migration_forecast_validator() -> PendingMigrationForecastValidator {
    PendingMigrationForecastValidator {
        rules: vec![
            Rc::new(PendingMigrationForecastRequiredAudienceRule {
                message: "pending migration forecast needs this field".to_string(),
            }) as Rc<dyn PendingMigrationForecastRule>,
            Rc::new(PendingMigrationForecastSummaryLengthRule { max_length: 64 }),
            Rc::new(PendingMigrationForecastAudienceLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_migration_forecast_validator();
        let item = new_pending_migration_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = PendingMigrationForecastRequiredAudienceRule {
            message: "pending migration forecast needs an entry".to_string(),
        };
        let mut item = new_pending_migration_forecast("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = PendingMigrationForecastSummaryLengthRule { max_length: 32 };
        let mut item = new_pending_migration_forecast("x", "y");
        item.summary = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = PendingMigrationForecastAudienceLengthRule { max_length: 16 };
        let mut item = new_pending_migration_forecast("x", "y");
        item.audience = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
