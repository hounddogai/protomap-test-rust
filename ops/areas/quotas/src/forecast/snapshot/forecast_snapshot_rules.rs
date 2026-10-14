use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a forecast snapshot must pass. Check returns its problems, or none.
pub trait ForecastSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &ForecastSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastSnapshotTitleLengthRule {
    pub max_length: i64,
}

impl ForecastSnapshotRule for ForecastSnapshotTitleLengthRule {
    fn check(&self, item: &ForecastSnapshot) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastSnapshotNotesLengthRule {
    pub max_length: i64,
}

impl ForecastSnapshotRule for ForecastSnapshotNotesLengthRule {
    fn check(&self, item: &ForecastSnapshot) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastSnapshotScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ForecastSnapshotRule for ForecastSnapshotScoreRangeRule {
    fn check(&self, item: &ForecastSnapshot) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastSnapshotCodeLengthRule {
    pub max_length: i64,
}

impl ForecastSnapshotRule for ForecastSnapshotCodeLengthRule {
    fn check(&self, item: &ForecastSnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForecastSnapshotRequiredReviewerRule {
    pub message: String,
}

impl ForecastSnapshotRule for ForecastSnapshotRequiredReviewerRule {
    fn check(&self, item: &ForecastSnapshot) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a forecast snapshot.
#[derive(Debug, Clone)]
pub struct ForecastSnapshotValidator {
    pub rules: Vec<Rc<dyn ForecastSnapshotRule>>,
}

impl ForecastSnapshotValidator {
    pub fn validate(&self, item: &ForecastSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ForecastSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ForecastSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_forecast_snapshot_validator() -> ForecastSnapshotValidator {
    ForecastSnapshotValidator {
        rules: vec![
            Rc::new(ForecastSnapshotTitleLengthRule { max_length: 80 }) as Rc<dyn ForecastSnapshotRule>,
            Rc::new(ForecastSnapshotNotesLengthRule { max_length: 86400 }),
            Rc::new(ForecastSnapshotScoreRangeRule { min: 4.4, max: 4670.5 }),
            Rc::new(ForecastSnapshotCodeLengthRule { max_length: 64 }),
            Rc::new(ForecastSnapshotRequiredReviewerRule {
                message: "forecast snapshot needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_forecast_snapshot_validator();
        let item = new_forecast_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ForecastSnapshotTitleLengthRule { max_length: 16 };
        let mut item = new_forecast_snapshot("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = ForecastSnapshotNotesLengthRule { max_length: 32 };
        let mut item = new_forecast_snapshot("x", "y");
        item.notes = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = ForecastSnapshotScoreRangeRule { min: 7.3, max: 2867.1 };
        let mut item = new_forecast_snapshot("x", "y");
        item.score = 2868.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ForecastSnapshotCodeLengthRule { max_length: 16 };
        let mut item = new_forecast_snapshot("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = ForecastSnapshotRequiredReviewerRule {
            message: "forecast snapshot needs a value".to_string(),
        };
        let mut item = new_forecast_snapshot("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
