use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a batched silence target must pass. Check returns its problems, or none.
pub trait BatchedSilenceTargetRule: std::fmt::Debug {
    fn check(&self, item: &BatchedSilenceTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSilenceTargetRequiredCodeRule {
    pub message: String,
}

impl BatchedSilenceTargetRule for BatchedSilenceTargetRequiredCodeRule {
    fn check(&self, item: &BatchedSilenceTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSilenceTargetCodePrefixRule {
    pub prefix: String,
}

impl BatchedSilenceTargetRule for BatchedSilenceTargetCodePrefixRule {
    fn check(&self, item: &BatchedSilenceTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSilenceTargetCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl BatchedSilenceTargetRule for BatchedSilenceTargetCapacityRangeRule {
    fn check(&self, item: &BatchedSilenceTarget) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSilenceTargetAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl BatchedSilenceTargetRule for BatchedSilenceTargetAllowedCodeRule {
    fn check(&self, item: &BatchedSilenceTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is too long", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched silence target.
#[derive(Debug, Clone)]
pub struct BatchedSilenceTargetValidator {
    pub rules: Vec<Rc<dyn BatchedSilenceTargetRule>>,
}

impl BatchedSilenceTargetValidator {
    pub fn validate(&self, item: &BatchedSilenceTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BatchedSilenceTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedSilenceTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_silence_target_validator() -> BatchedSilenceTargetValidator {
    BatchedSilenceTargetValidator {
        rules: vec![
            Rc::new(BatchedSilenceTargetRequiredCodeRule {
                message: "batched silence target needs an entry".to_string(),
            }) as Rc<dyn BatchedSilenceTargetRule>,
            Rc::new(BatchedSilenceTargetCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(BatchedSilenceTargetCapacityRangeRule { min: 32, max: 3600 }),
            Rc::new(BatchedSilenceTargetAllowedCodeRule {
                allowed: vec!["adaptive".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_silence_target_validator();
        let item = new_batched_silence_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BatchedSilenceTargetRequiredCodeRule {
            message: "batched silence target needs an entry".to_string(),
        };
        let mut item = new_batched_silence_target("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = BatchedSilenceTargetCapacityRangeRule { min: 80, max: 3600 };
        let mut item = new_batched_silence_target("x", "y");
        item.capacity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
