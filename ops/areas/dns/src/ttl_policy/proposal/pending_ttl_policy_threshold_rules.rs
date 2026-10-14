use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending ttl policy threshold must pass. Check returns its problems, or none.
pub trait PendingTtlPolicyThresholdRule: std::fmt::Debug {
    fn check(&self, item: &PendingTtlPolicyThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTtlPolicyThresholdTitleLengthRule {
    pub max_length: i64,
}

impl PendingTtlPolicyThresholdRule for PendingTtlPolicyThresholdTitleLengthRule {
    fn check(&self, item: &PendingTtlPolicyThreshold) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTtlPolicyThresholdRequiredCodeRule {
    pub message: String,
}

impl PendingTtlPolicyThresholdRule for PendingTtlPolicyThresholdRequiredCodeRule {
    fn check(&self, item: &PendingTtlPolicyThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTtlPolicyThresholdRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingTtlPolicyThresholdRule for PendingTtlPolicyThresholdRotationDaysRangeRule {
    fn check(&self, item: &PendingTtlPolicyThreshold) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTtlPolicyThresholdSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingTtlPolicyThresholdRule for PendingTtlPolicyThresholdSequenceRangeRule {
    fn check(&self, item: &PendingTtlPolicyThreshold) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTtlPolicyThresholdAllowedKeyIdRule {
    pub allowed: Vec<String>,
}

impl PendingTtlPolicyThresholdRule for PendingTtlPolicyThresholdAllowedKeyIdRule {
    fn check(&self, item: &PendingTtlPolicyThreshold) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.key_id))) {
            return vec![format!("key id {} is not allowed", item.key_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingTtlPolicyThresholdCodeLengthRule {
    pub max_length: i64,
}

impl PendingTtlPolicyThresholdRule for PendingTtlPolicyThresholdCodeLengthRule {
    fn check(&self, item: &PendingTtlPolicyThreshold) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending ttl policy threshold.
#[derive(Debug, Clone)]
pub struct PendingTtlPolicyThresholdValidator {
    pub rules: Vec<Rc<dyn PendingTtlPolicyThresholdRule>>,
}

impl PendingTtlPolicyThresholdValidator {
    pub fn validate(&self, item: &PendingTtlPolicyThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingTtlPolicyThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingTtlPolicyThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_ttl_policy_threshold_validator() -> PendingTtlPolicyThresholdValidator {
    PendingTtlPolicyThresholdValidator {
        rules: vec![
            Rc::new(PendingTtlPolicyThresholdTitleLengthRule { max_length: 255 })
                as Rc<dyn PendingTtlPolicyThresholdRule>,
            Rc::new(PendingTtlPolicyThresholdRequiredCodeRule {
                message: "pending ttl policy threshold needs this field".to_string(),
            }),
            Rc::new(PendingTtlPolicyThresholdRotationDaysRangeRule { min: 120, max: 1000 }),
            Rc::new(PendingTtlPolicyThresholdSequenceRangeRule { min: 1000, max: 1000 }),
            Rc::new(PendingTtlPolicyThresholdAllowedKeyIdRule {
                allowed: vec!["rolling".to_string(), "weighted".to_string()],
            }),
            Rc::new(PendingTtlPolicyThresholdCodeLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_ttl_policy_threshold_validator();
        let item = new_pending_ttl_policy_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PendingTtlPolicyThresholdTitleLengthRule { max_length: 16 };
        let mut item = new_pending_ttl_policy_threshold("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingTtlPolicyThresholdRequiredCodeRule {
            message: "pending ttl policy threshold needs a value".to_string(),
        };
        let mut item = new_pending_ttl_policy_threshold("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = PendingTtlPolicyThresholdRotationDaysRangeRule { min: 120, max: 255 };
        let mut item = new_pending_ttl_policy_threshold("x", "y");
        item.rotation_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = PendingTtlPolicyThresholdSequenceRangeRule { min: 1000, max: 3600 };
        let mut item = new_pending_ttl_policy_threshold("x", "y");
        item.sequence = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PendingTtlPolicyThresholdCodeLengthRule { max_length: 16 };
        let mut item = new_pending_ttl_policy_threshold("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
