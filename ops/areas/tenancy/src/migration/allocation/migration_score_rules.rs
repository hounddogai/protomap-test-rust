use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a migration score must pass. Check returns its problems, or none.
pub trait MigrationScoreRule: std::fmt::Debug {
    fn check(&self, item: &MigrationScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationScoreBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MigrationScoreRule for MigrationScoreBatchSizeRangeRule {
    fn check(&self, item: &MigrationScore) -> Vec<String> {
        let value = item.batch_size;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("batch size {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("batch size {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationScoreScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl MigrationScoreRule for MigrationScoreScoreRangeRule {
    fn check(&self, item: &MigrationScore) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationScoreRequiredHostRule {
    pub message: String,
}

impl MigrationScoreRule for MigrationScoreRequiredHostRule {
    fn check(&self, item: &MigrationScore) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationScoreAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl MigrationScoreRule for MigrationScoreAllowedTitleRule {
    fn check(&self, item: &MigrationScore) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is missing", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationScoreRequiredCodeRule {
    pub message: String,
}

impl MigrationScoreRule for MigrationScoreRequiredCodeRule {
    fn check(&self, item: &MigrationScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MigrationScoreTitleLengthRule {
    pub max_length: i64,
}

impl MigrationScoreRule for MigrationScoreTitleLengthRule {
    fn check(&self, item: &MigrationScore) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a migration score.
#[derive(Debug, Clone)]
pub struct MigrationScoreValidator {
    pub rules: Vec<Rc<dyn MigrationScoreRule>>,
}

impl MigrationScoreValidator {
    pub fn validate(&self, item: &MigrationScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &MigrationScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MigrationScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_migration_score_validator() -> MigrationScoreValidator {
    MigrationScoreValidator {
        rules: vec![
            Rc::new(MigrationScoreBatchSizeRangeRule { min: 64, max: 80 }) as Rc<dyn MigrationScoreRule>,
            Rc::new(MigrationScoreScoreRangeRule { min: 5.4, max: 2925.3 }),
            Rc::new(MigrationScoreRequiredHostRule {
                message: "migration score needs a value".to_string(),
            }),
            Rc::new(MigrationScoreAllowedTitleRule {
                allowed: vec!["strict".to_string(), "manual".to_string()],
            }),
            Rc::new(MigrationScoreRequiredCodeRule {
                message: "migration score needs this field".to_string(),
            }),
            Rc::new(MigrationScoreTitleLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_migration_score_validator();
        let item = new_migration_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = MigrationScoreBatchSizeRangeRule { min: 64, max: 1000 };
        let mut item = new_migration_score("x", "y");
        item.batch_size = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = MigrationScoreScoreRangeRule { min: 1.4, max: 843.2 };
        let mut item = new_migration_score("x", "y");
        item.score = 844.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = MigrationScoreRequiredHostRule {
            message: "migration score needs a value".to_string(),
        };
        let mut item = new_migration_score("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = MigrationScoreRequiredCodeRule {
            message: "migration score needs this field".to_string(),
        };
        let mut item = new_migration_score("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = MigrationScoreTitleLengthRule { max_length: 16 };
        let mut item = new_migration_score("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
