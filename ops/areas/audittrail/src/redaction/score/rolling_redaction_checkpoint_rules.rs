use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling redaction checkpoint must pass. Check returns its problems, or none.
pub trait RollingRedactionCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &RollingRedactionCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRedactionCheckpointCodeLengthRule {
    pub max_length: i64,
}

impl RollingRedactionCheckpointRule for RollingRedactionCheckpointCodeLengthRule {
    fn check(&self, item: &RollingRedactionCheckpoint) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRedactionCheckpointRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingRedactionCheckpointRule for RollingRedactionCheckpointRateRangeRule {
    fn check(&self, item: &RollingRedactionCheckpoint) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRedactionCheckpointRequiredCodeRule {
    pub message: String,
}

impl RollingRedactionCheckpointRule for RollingRedactionCheckpointRequiredCodeRule {
    fn check(&self, item: &RollingRedactionCheckpoint) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRedactionCheckpointAllowedSegmentRule {
    pub allowed: Vec<String>,
}

impl RollingRedactionCheckpointRule for RollingRedactionCheckpointAllowedSegmentRule {
    fn check(&self, item: &RollingRedactionCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.segment)))
        {
            return vec![format!("segment {} is above maximum", item.segment)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingRedactionCheckpointFingerprintLengthRule {
    pub max_length: i64,
}

impl RollingRedactionCheckpointRule for RollingRedactionCheckpointFingerprintLengthRule {
    fn check(&self, item: &RollingRedactionCheckpoint) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling redaction checkpoint.
#[derive(Debug, Clone)]
pub struct RollingRedactionCheckpointValidator {
    pub rules: Vec<Rc<dyn RollingRedactionCheckpointRule>>,
}

impl RollingRedactionCheckpointValidator {
    pub fn validate(&self, item: &RollingRedactionCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingRedactionCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingRedactionCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_redaction_checkpoint_validator() -> RollingRedactionCheckpointValidator {
    RollingRedactionCheckpointValidator {
        rules: vec![
            Rc::new(RollingRedactionCheckpointCodeLengthRule { max_length: 120 })
                as Rc<dyn RollingRedactionCheckpointRule>,
            Rc::new(RollingRedactionCheckpointRateRangeRule { min: 7.0, max: 3998.9 }),
            Rc::new(RollingRedactionCheckpointRequiredCodeRule {
                message: "rolling redaction checkpoint needs this field".to_string(),
            }),
            Rc::new(RollingRedactionCheckpointAllowedSegmentRule {
                allowed: vec!["fallback".to_string(), "strict".to_string()],
            }),
            Rc::new(RollingRedactionCheckpointFingerprintLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_redaction_checkpoint_validator();
        let item = new_rolling_redaction_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingRedactionCheckpointCodeLengthRule { max_length: 1000 };
        let mut item = new_rolling_redaction_checkpoint("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = RollingRedactionCheckpointRateRangeRule { min: 5.7, max: 1997.1 };
        let mut item = new_rolling_redaction_checkpoint("x", "y");
        item.rate = 1998.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingRedactionCheckpointRequiredCodeRule {
            message: "rolling redaction checkpoint needs an entry".to_string(),
        };
        let mut item = new_rolling_redaction_checkpoint("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = RollingRedactionCheckpointFingerprintLengthRule { max_length: 1000 };
        let mut item = new_rolling_redaction_checkpoint("x", "y");
        item.fingerprint = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
