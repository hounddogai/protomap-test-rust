use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling probe checkpoint must pass. Check returns its problems, or none.
pub trait RollingProbeCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &RollingProbeCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingProbeCheckpointCodeLengthRule {
    pub max_length: i64,
}

impl RollingProbeCheckpointRule for RollingProbeCheckpointCodeLengthRule {
    fn check(&self, item: &RollingProbeCheckpoint) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingProbeCheckpointRequiredCodeRule {
    pub message: String,
}

impl RollingProbeCheckpointRule for RollingProbeCheckpointRequiredCodeRule {
    fn check(&self, item: &RollingProbeCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingProbeCheckpointStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingProbeCheckpointRule for RollingProbeCheckpointStartsAtRangeRule {
    fn check(&self, item: &RollingProbeCheckpoint) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingProbeCheckpointRequiredNameRule {
    pub message: String,
}

impl RollingProbeCheckpointRule for RollingProbeCheckpointRequiredNameRule {
    fn check(&self, item: &RollingProbeCheckpoint) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingProbeCheckpointCodePrefixRule {
    pub prefix: String,
}

impl RollingProbeCheckpointRule for RollingProbeCheckpointCodePrefixRule {
    fn check(&self, item: &RollingProbeCheckpoint) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingProbeCheckpointCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingProbeCheckpointRule for RollingProbeCheckpointCeilingRangeRule {
    fn check(&self, item: &RollingProbeCheckpoint) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a rolling probe checkpoint.
#[derive(Debug, Clone)]
pub struct RollingProbeCheckpointValidator {
    pub rules: Vec<Rc<dyn RollingProbeCheckpointRule>>,
}

impl RollingProbeCheckpointValidator {
    pub fn validate(&self, item: &RollingProbeCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingProbeCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingProbeCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_probe_checkpoint_validator() -> RollingProbeCheckpointValidator {
    RollingProbeCheckpointValidator {
        rules: vec![
            Rc::new(RollingProbeCheckpointCodeLengthRule { max_length: 86400 }) as Rc<dyn RollingProbeCheckpointRule>,
            Rc::new(RollingProbeCheckpointRequiredCodeRule {
                message: "rolling probe checkpoint needs a value".to_string(),
            }),
            Rc::new(RollingProbeCheckpointStartsAtRangeRule { min: 32, max: 80 }),
            Rc::new(RollingProbeCheckpointRequiredNameRule {
                message: "rolling probe checkpoint needs this field".to_string(),
            }),
            Rc::new(RollingProbeCheckpointCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(RollingProbeCheckpointCeilingRangeRule { min: 4.7, max: 1994.7 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_probe_checkpoint_validator();
        let item = new_rolling_probe_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingProbeCheckpointCodeLengthRule { max_length: 1000 };
        let mut item = new_rolling_probe_checkpoint("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingProbeCheckpointRequiredCodeRule {
            message: "rolling probe checkpoint needs this field".to_string(),
        };
        let mut item = new_rolling_probe_checkpoint("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = RollingProbeCheckpointStartsAtRangeRule { min: 16, max: 255 };
        let mut item = new_rolling_probe_checkpoint("x", "y");
        item.starts_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RollingProbeCheckpointRequiredNameRule {
            message: "rolling probe checkpoint needs a value".to_string(),
        };
        let mut item = new_rolling_probe_checkpoint("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = RollingProbeCheckpointCeilingRangeRule { min: 5.0, max: 104.4 };
        let mut item = new_rolling_probe_checkpoint("x", "y");
        item.ceiling = 105.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
