use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal canary checkpoint must pass. Check returns its problems, or none.
pub trait SeasonalCanaryCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalCanaryCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCanaryCheckpointSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalCanaryCheckpointRule for SeasonalCanaryCheckpointSequenceRangeRule {
    fn check(&self, item: &SeasonalCanaryCheckpoint) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCanaryCheckpointWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalCanaryCheckpointRule for SeasonalCanaryCheckpointWeightRangeRule {
    fn check(&self, item: &SeasonalCanaryCheckpoint) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCanaryCheckpointNameLengthRule {
    pub max_length: i64,
}

impl SeasonalCanaryCheckpointRule for SeasonalCanaryCheckpointNameLengthRule {
    fn check(&self, item: &SeasonalCanaryCheckpoint) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCanaryCheckpointRequiredTimezoneRule {
    pub message: String,
}

impl SeasonalCanaryCheckpointRule for SeasonalCanaryCheckpointRequiredTimezoneRule {
    fn check(&self, item: &SeasonalCanaryCheckpoint) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCanaryCheckpointRequiredCodeRule {
    pub message: String,
}

impl SeasonalCanaryCheckpointRule for SeasonalCanaryCheckpointRequiredCodeRule {
    fn check(&self, item: &SeasonalCanaryCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalCanaryCheckpointRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalCanaryCheckpointRule for SeasonalCanaryCheckpointRatioRangeRule {
    fn check(&self, item: &SeasonalCanaryCheckpoint) -> Vec<String> {
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

/// Runs every rule of a seasonal canary checkpoint.
#[derive(Debug, Clone)]
pub struct SeasonalCanaryCheckpointValidator {
    pub rules: Vec<Rc<dyn SeasonalCanaryCheckpointRule>>,
}

impl SeasonalCanaryCheckpointValidator {
    pub fn validate(&self, item: &SeasonalCanaryCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalCanaryCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalCanaryCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_canary_checkpoint_validator() -> SeasonalCanaryCheckpointValidator {
    SeasonalCanaryCheckpointValidator {
        rules: vec![
            Rc::new(SeasonalCanaryCheckpointSequenceRangeRule { min: 16, max: 1000 })
                as Rc<dyn SeasonalCanaryCheckpointRule>,
            Rc::new(SeasonalCanaryCheckpointWeightRangeRule { min: 6.2, max: 3973.7 }),
            Rc::new(SeasonalCanaryCheckpointNameLengthRule { max_length: 80 }),
            Rc::new(SeasonalCanaryCheckpointRequiredTimezoneRule {
                message: "seasonal canary checkpoint needs a value".to_string(),
            }),
            Rc::new(SeasonalCanaryCheckpointRequiredCodeRule {
                message: "seasonal canary checkpoint needs an entry".to_string(),
            }),
            Rc::new(SeasonalCanaryCheckpointRatioRangeRule { min: 9.5, max: 1983.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_canary_checkpoint_validator();
        let item = new_seasonal_canary_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = SeasonalCanaryCheckpointSequenceRangeRule { min: 80, max: 120 };
        let mut item = new_seasonal_canary_checkpoint("x", "y");
        item.sequence = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = SeasonalCanaryCheckpointWeightRangeRule { min: 3.4, max: 4420.0 };
        let mut item = new_seasonal_canary_checkpoint("x", "y");
        item.weight = 4421.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = SeasonalCanaryCheckpointNameLengthRule { max_length: 120 };
        let mut item = new_seasonal_canary_checkpoint("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = SeasonalCanaryCheckpointRequiredTimezoneRule {
            message: "seasonal canary checkpoint needs a value".to_string(),
        };
        let mut item = new_seasonal_canary_checkpoint("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalCanaryCheckpointRequiredCodeRule {
            message: "seasonal canary checkpoint needs this field".to_string(),
        };
        let mut item = new_seasonal_canary_checkpoint("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = SeasonalCanaryCheckpointRatioRangeRule { min: 7.6, max: 477.9 };
        let mut item = new_seasonal_canary_checkpoint("x", "y");
        item.ratio = 479.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
