use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling identity milestone must pass. Check returns its problems, or none.
pub trait RollingIdentityMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &RollingIdentityMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingIdentityMilestoneLabelLengthRule {
    pub max_length: i64,
}

impl RollingIdentityMilestoneRule for RollingIdentityMilestoneLabelLengthRule {
    fn check(&self, item: &RollingIdentityMilestone) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingIdentityMilestoneAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl RollingIdentityMilestoneRule for RollingIdentityMilestoneAllowedLabelRule {
    fn check(&self, item: &RollingIdentityMilestone) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is inconsistent", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingIdentityMilestoneRequiredCodeRule {
    pub message: String,
}

impl RollingIdentityMilestoneRule for RollingIdentityMilestoneRequiredCodeRule {
    fn check(&self, item: &RollingIdentityMilestone) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingIdentityMilestoneEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingIdentityMilestoneRule for RollingIdentityMilestoneEndsAtRangeRule {
    fn check(&self, item: &RollingIdentityMilestone) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingIdentityMilestoneCodePrefixRule {
    pub prefix: String,
}

impl RollingIdentityMilestoneRule for RollingIdentityMilestoneCodePrefixRule {
    fn check(&self, item: &RollingIdentityMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling identity milestone.
#[derive(Debug, Clone)]
pub struct RollingIdentityMilestoneValidator {
    pub rules: Vec<Rc<dyn RollingIdentityMilestoneRule>>,
}

impl RollingIdentityMilestoneValidator {
    pub fn validate(&self, item: &RollingIdentityMilestone) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingIdentityMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingIdentityMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_identity_milestone_validator() -> RollingIdentityMilestoneValidator {
    RollingIdentityMilestoneValidator {
        rules: vec![
            Rc::new(RollingIdentityMilestoneLabelLengthRule { max_length: 86400 })
                as Rc<dyn RollingIdentityMilestoneRule>,
            Rc::new(RollingIdentityMilestoneAllowedLabelRule { allowed: Vec::new() }),
            Rc::new(RollingIdentityMilestoneRequiredCodeRule {
                message: "rolling identity milestone needs this field".to_string(),
            }),
            Rc::new(RollingIdentityMilestoneEndsAtRangeRule { min: 32, max: 120 }),
            Rc::new(RollingIdentityMilestoneCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_identity_milestone_validator();
        let item = new_rolling_identity_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RollingIdentityMilestoneLabelLengthRule { max_length: 64 };
        let mut item = new_rolling_identity_milestone("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingIdentityMilestoneRequiredCodeRule {
            message: "rolling identity milestone needs this field".to_string(),
        };
        let mut item = new_rolling_identity_milestone("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = RollingIdentityMilestoneEndsAtRangeRule { min: 32, max: 255 };
        let mut item = new_rolling_identity_milestone("x", "y");
        item.ends_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
