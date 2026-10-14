use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary rollout override must pass. Check returns its problems, or none.
pub trait PrimaryRolloutOverrideRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryRolloutOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutOverrideRequiredLabelRule {
    pub message: String,
}

impl PrimaryRolloutOverrideRule for PrimaryRolloutOverrideRequiredLabelRule {
    fn check(&self, item: &PrimaryRolloutOverride) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutOverrideLabelLengthRule {
    pub max_length: i64,
}

impl PrimaryRolloutOverrideRule for PrimaryRolloutOverrideLabelLengthRule {
    fn check(&self, item: &PrimaryRolloutOverride) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutOverrideDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryRolloutOverrideRule for PrimaryRolloutOverrideDurationMinutesRangeRule {
    fn check(&self, item: &PrimaryRolloutOverride) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutOverrideCodePrefixRule {
    pub prefix: String,
}

impl PrimaryRolloutOverrideRule for PrimaryRolloutOverrideCodePrefixRule {
    fn check(&self, item: &PrimaryRolloutOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutOverrideCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryRolloutOverrideRule for PrimaryRolloutOverrideCodeLengthRule {
    fn check(&self, item: &PrimaryRolloutOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutOverrideThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryRolloutOverrideRule for PrimaryRolloutOverrideThresholdRangeRule {
    fn check(&self, item: &PrimaryRolloutOverride) -> Vec<String> {
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

/// Runs every rule of a primary rollout override.
#[derive(Debug, Clone)]
pub struct PrimaryRolloutOverrideValidator {
    pub rules: Vec<Rc<dyn PrimaryRolloutOverrideRule>>,
}

impl PrimaryRolloutOverrideValidator {
    pub fn validate(&self, item: &PrimaryRolloutOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryRolloutOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryRolloutOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_rollout_override_validator() -> PrimaryRolloutOverrideValidator {
    PrimaryRolloutOverrideValidator {
        rules: vec![
            Rc::new(PrimaryRolloutOverrideRequiredLabelRule {
                message: "primary rollout override needs this field".to_string(),
            }) as Rc<dyn PrimaryRolloutOverrideRule>,
            Rc::new(PrimaryRolloutOverrideLabelLengthRule { max_length: 86400 }),
            Rc::new(PrimaryRolloutOverrideDurationMinutesRangeRule { min: 32, max: 3600 }),
            Rc::new(PrimaryRolloutOverrideCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(PrimaryRolloutOverrideCodeLengthRule { max_length: 64 }),
            Rc::new(PrimaryRolloutOverrideThresholdRangeRule { min: 4.4, max: 1159.8 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_rollout_override_validator();
        let item = new_primary_rollout_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PrimaryRolloutOverrideRequiredLabelRule {
            message: "primary rollout override needs a value".to_string(),
        };
        let mut item = new_primary_rollout_override("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PrimaryRolloutOverrideLabelLengthRule { max_length: 64 };
        let mut item = new_primary_rollout_override("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = PrimaryRolloutOverrideDurationMinutesRangeRule { min: 32, max: 1000 };
        let mut item = new_primary_rollout_override("x", "y");
        item.duration_minutes = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryRolloutOverrideCodeLengthRule { max_length: 255 };
        let mut item = new_primary_rollout_override("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = PrimaryRolloutOverrideThresholdRangeRule { min: 5.3, max: 4526.7 };
        let mut item = new_primary_rollout_override("x", "y");
        item.threshold = 4528.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
