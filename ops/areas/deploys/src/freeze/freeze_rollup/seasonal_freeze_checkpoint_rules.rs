use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal freeze checkpoint must pass. Check returns its problems, or none.
pub trait SeasonalFreezeCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalFreezeCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeCheckpointCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalFreezeCheckpointRule for SeasonalFreezeCheckpointCodeLengthRule {
    fn check(&self, item: &SeasonalFreezeCheckpoint) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeCheckpointRequiredCodeRule {
    pub message: String,
}

impl SeasonalFreezeCheckpointRule for SeasonalFreezeCheckpointRequiredCodeRule {
    fn check(&self, item: &SeasonalFreezeCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeCheckpointLabelLengthRule {
    pub max_length: i64,
}

impl SeasonalFreezeCheckpointRule for SeasonalFreezeCheckpointLabelLengthRule {
    fn check(&self, item: &SeasonalFreezeCheckpoint) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeCheckpointVisibleGuardRule {
    pub limit: i64,
}

impl SeasonalFreezeCheckpointRule for SeasonalFreezeCheckpointVisibleGuardRule {
    fn check(&self, item: &SeasonalFreezeCheckpoint) -> Vec<String> {
        if item.visible && (item.expires_at > self.limit) {
            return vec![format!(
                "expires at {} exceeds {} while visible",
                item.expires_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeCheckpointRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalFreezeCheckpointRule for SeasonalFreezeCheckpointRatioRangeRule {
    fn check(&self, item: &SeasonalFreezeCheckpoint) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a seasonal freeze checkpoint.
#[derive(Debug, Clone)]
pub struct SeasonalFreezeCheckpointValidator {
    pub rules: Vec<Rc<dyn SeasonalFreezeCheckpointRule>>,
}

impl SeasonalFreezeCheckpointValidator {
    pub fn validate(&self, item: &SeasonalFreezeCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalFreezeCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalFreezeCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_freeze_checkpoint_validator() -> SeasonalFreezeCheckpointValidator {
    SeasonalFreezeCheckpointValidator {
        rules: vec![
            Rc::new(SeasonalFreezeCheckpointCodeLengthRule { max_length: 1000 })
                as Rc<dyn SeasonalFreezeCheckpointRule>,
            Rc::new(SeasonalFreezeCheckpointRequiredCodeRule {
                message: "seasonal freeze checkpoint needs an entry".to_string(),
            }),
            Rc::new(SeasonalFreezeCheckpointLabelLengthRule { max_length: 16 }),
            Rc::new(SeasonalFreezeCheckpointVisibleGuardRule { limit: 120 }),
            Rc::new(SeasonalFreezeCheckpointRatioRangeRule { min: 8.8, max: 262.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_freeze_checkpoint_validator();
        let item = new_seasonal_freeze_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SeasonalFreezeCheckpointCodeLengthRule { max_length: 64 };
        let mut item = new_seasonal_freeze_checkpoint("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalFreezeCheckpointRequiredCodeRule {
            message: "seasonal freeze checkpoint needs an entry".to_string(),
        };
        let mut item = new_seasonal_freeze_checkpoint("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = SeasonalFreezeCheckpointLabelLengthRule { max_length: 255 };
        let mut item = new_seasonal_freeze_checkpoint("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = SeasonalFreezeCheckpointRatioRangeRule { min: 9.1, max: 4928.9 };
        let mut item = new_seasonal_freeze_checkpoint("x", "y");
        item.ratio = 4930.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
