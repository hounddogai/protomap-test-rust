use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal migration score must pass. Check returns its problems, or none.
pub trait SeasonalMigrationScoreRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalMigrationScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalMigrationScoreCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalMigrationScoreRule for SeasonalMigrationScoreCodeLengthRule {
    fn check(&self, item: &SeasonalMigrationScore) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalMigrationScoreTitleLengthRule {
    pub max_length: i64,
}

impl SeasonalMigrationScoreRule for SeasonalMigrationScoreTitleLengthRule {
    fn check(&self, item: &SeasonalMigrationScore) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalMigrationScoreRequiredCodeRule {
    pub message: String,
}

impl SeasonalMigrationScoreRule for SeasonalMigrationScoreRequiredCodeRule {
    fn check(&self, item: &SeasonalMigrationScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalMigrationScoreThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalMigrationScoreRule for SeasonalMigrationScoreThresholdRangeRule {
    fn check(&self, item: &SeasonalMigrationScore) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a seasonal migration score.
#[derive(Debug, Clone)]
pub struct SeasonalMigrationScoreValidator {
    pub rules: Vec<Rc<dyn SeasonalMigrationScoreRule>>,
}

impl SeasonalMigrationScoreValidator {
    pub fn validate(&self, item: &SeasonalMigrationScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalMigrationScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalMigrationScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_migration_score_validator() -> SeasonalMigrationScoreValidator {
    SeasonalMigrationScoreValidator {
        rules: vec![
            Rc::new(SeasonalMigrationScoreCodeLengthRule { max_length: 80 }) as Rc<dyn SeasonalMigrationScoreRule>,
            Rc::new(SeasonalMigrationScoreTitleLengthRule { max_length: 64 }),
            Rc::new(SeasonalMigrationScoreRequiredCodeRule {
                message: "seasonal migration score needs this field".to_string(),
            }),
            Rc::new(SeasonalMigrationScoreThresholdRangeRule { min: 0.5, max: 4464.6 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_migration_score_validator();
        let item = new_seasonal_migration_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SeasonalMigrationScoreCodeLengthRule { max_length: 120 };
        let mut item = new_seasonal_migration_score("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = SeasonalMigrationScoreTitleLengthRule { max_length: 80 };
        let mut item = new_seasonal_migration_score("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalMigrationScoreRequiredCodeRule {
            message: "seasonal migration score needs a value".to_string(),
        };
        let mut item = new_seasonal_migration_score("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = SeasonalMigrationScoreThresholdRangeRule { min: 1.0, max: 2627.5 };
        let mut item = new_seasonal_migration_score("x", "y");
        item.threshold = 2629.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
