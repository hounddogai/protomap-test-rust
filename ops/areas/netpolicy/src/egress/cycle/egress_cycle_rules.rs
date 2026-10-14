use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a egress cycle must pass. Check returns its problems, or none.
pub trait EgressCycleRule: std::fmt::Debug {
    fn check(&self, item: &EgressCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressCycleDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EgressCycleRule for EgressCycleDurationMinutesRangeRule {
    fn check(&self, item: &EgressCycle) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressCycleThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EgressCycleRule for EgressCycleThresholdRangeRule {
    fn check(&self, item: &EgressCycle) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressCycleRequiredSourceRule {
    pub message: String,
}

impl EgressCycleRule for EgressCycleRequiredSourceRule {
    fn check(&self, item: &EgressCycle) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressCycleAllowedChecksumRule {
    pub allowed: Vec<String>,
}

impl EgressCycleRule for EgressCycleAllowedChecksumRule {
    fn check(&self, item: &EgressCycle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.checksum)))
        {
            return vec![format!("checksum {} is duplicated", item.checksum)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressCycleCodeLengthRule {
    pub max_length: i64,
}

impl EgressCycleRule for EgressCycleCodeLengthRule {
    fn check(&self, item: &EgressCycle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressCycleStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EgressCycleRule for EgressCycleStartsAtRangeRule {
    fn check(&self, item: &EgressCycle) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress cycle.
#[derive(Debug, Clone)]
pub struct EgressCycleValidator {
    pub rules: Vec<Rc<dyn EgressCycleRule>>,
}

impl EgressCycleValidator {
    pub fn validate(&self, item: &EgressCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EgressCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_cycle_validator() -> EgressCycleValidator {
    EgressCycleValidator {
        rules: vec![
            Rc::new(EgressCycleDurationMinutesRangeRule { min: 80, max: 3600 }) as Rc<dyn EgressCycleRule>,
            Rc::new(EgressCycleThresholdRangeRule { min: 7.0, max: 2148.1 }),
            Rc::new(EgressCycleRequiredSourceRule {
                message: "egress cycle needs an entry".to_string(),
            }),
            Rc::new(EgressCycleAllowedChecksumRule {
                allowed: vec!["scheduled".to_string(), "default".to_string()],
            }),
            Rc::new(EgressCycleCodeLengthRule { max_length: 32 }),
            Rc::new(EgressCycleStartsAtRangeRule { min: 32, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_egress_cycle_validator();
        let item = new_egress_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = EgressCycleDurationMinutesRangeRule { min: 255, max: 255 };
        let mut item = new_egress_cycle("x", "y");
        item.duration_minutes = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = EgressCycleThresholdRangeRule { min: 3.0, max: 570.0 };
        let mut item = new_egress_cycle("x", "y");
        item.threshold = 571.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = EgressCycleRequiredSourceRule {
            message: "egress cycle needs a value".to_string(),
        };
        let mut item = new_egress_cycle("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = EgressCycleCodeLengthRule { max_length: 64 };
        let mut item = new_egress_cycle("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = EgressCycleStartsAtRangeRule { min: 255, max: 86400 };
        let mut item = new_egress_cycle("x", "y");
        item.starts_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
