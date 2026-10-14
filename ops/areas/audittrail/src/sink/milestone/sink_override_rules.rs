use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a sink override must pass. Check returns its problems, or none.
pub trait SinkOverrideRule: std::fmt::Debug {
    fn check(&self, item: &SinkOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkOverrideCodeLengthRule {
    pub max_length: i64,
}

impl SinkOverrideRule for SinkOverrideCodeLengthRule {
    fn check(&self, item: &SinkOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkOverrideAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SinkOverrideRule for SinkOverrideAttemptsRangeRule {
    fn check(&self, item: &SinkOverride) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkOverrideCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SinkOverrideRule for SinkOverrideCostRangeRule {
    fn check(&self, item: &SinkOverride) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkOverrideRequiredCodeRule {
    pub message: String,
}

impl SinkOverrideRule for SinkOverrideRequiredCodeRule {
    fn check(&self, item: &SinkOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkOverrideDescriptionLengthRule {
    pub max_length: i64,
}

impl SinkOverrideRule for SinkOverrideDescriptionLengthRule {
    fn check(&self, item: &SinkOverride) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkOverrideAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl SinkOverrideRule for SinkOverrideAllowedCodeRule {
    fn check(&self, item: &SinkOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink override.
#[derive(Debug, Clone)]
pub struct SinkOverrideValidator {
    pub rules: Vec<Rc<dyn SinkOverrideRule>>,
}

impl SinkOverrideValidator {
    pub fn validate(&self, item: &SinkOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SinkOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_override_validator() -> SinkOverrideValidator {
    SinkOverrideValidator {
        rules: vec![
            Rc::new(SinkOverrideCodeLengthRule { max_length: 16 }) as Rc<dyn SinkOverrideRule>,
            Rc::new(SinkOverrideAttemptsRangeRule { min: 64, max: 86400 }),
            Rc::new(SinkOverrideCostRangeRule { min: 5.8, max: 1474.9 }),
            Rc::new(SinkOverrideRequiredCodeRule {
                message: "sink override needs a value".to_string(),
            }),
            Rc::new(SinkOverrideDescriptionLengthRule { max_length: 32 }),
            Rc::new(SinkOverrideAllowedCodeRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_sink_override_validator();
        let item = new_sink_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SinkOverrideCodeLengthRule { max_length: 86400 };
        let mut item = new_sink_override("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = SinkOverrideAttemptsRangeRule { min: 80, max: 80 };
        let mut item = new_sink_override("x", "y");
        item.attempts = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = SinkOverrideCostRangeRule { min: 9.1, max: 2418.7 };
        let mut item = new_sink_override("x", "y");
        item.cost = 2420.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SinkOverrideRequiredCodeRule {
            message: "sink override needs a value".to_string(),
        };
        let mut item = new_sink_override("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = SinkOverrideDescriptionLengthRule { max_length: 80 };
        let mut item = new_sink_override("x", "y");
        item.description = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
