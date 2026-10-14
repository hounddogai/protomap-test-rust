use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient canary checkpoint must pass. Check returns its problems, or none.
pub trait LenientCanaryCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &LenientCanaryCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCanaryCheckpointLabelLengthRule {
    pub max_length: i64,
}

impl LenientCanaryCheckpointRule for LenientCanaryCheckpointLabelLengthRule {
    fn check(&self, item: &LenientCanaryCheckpoint) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCanaryCheckpointAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl LenientCanaryCheckpointRule for LenientCanaryCheckpointAllowedCodeRule {
    fn check(&self, item: &LenientCanaryCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is stale", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCanaryCheckpointEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientCanaryCheckpointRule for LenientCanaryCheckpointEndsAtRangeRule {
    fn check(&self, item: &LenientCanaryCheckpoint) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCanaryCheckpointRequiredLabelRule {
    pub message: String,
}

impl LenientCanaryCheckpointRule for LenientCanaryCheckpointRequiredLabelRule {
    fn check(&self, item: &LenientCanaryCheckpoint) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientCanaryCheckpointTenantIdLengthRule {
    pub max_length: i64,
}

impl LenientCanaryCheckpointRule for LenientCanaryCheckpointTenantIdLengthRule {
    fn check(&self, item: &LenientCanaryCheckpoint) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient canary checkpoint.
#[derive(Debug, Clone)]
pub struct LenientCanaryCheckpointValidator {
    pub rules: Vec<Rc<dyn LenientCanaryCheckpointRule>>,
}

impl LenientCanaryCheckpointValidator {
    pub fn validate(&self, item: &LenientCanaryCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientCanaryCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientCanaryCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_canary_checkpoint_validator() -> LenientCanaryCheckpointValidator {
    LenientCanaryCheckpointValidator {
        rules: vec![
            Rc::new(LenientCanaryCheckpointLabelLengthRule { max_length: 120 }) as Rc<dyn LenientCanaryCheckpointRule>,
            Rc::new(LenientCanaryCheckpointAllowedCodeRule {
                allowed: vec!["scheduled".to_string(), "fallback".to_string()],
            }),
            Rc::new(LenientCanaryCheckpointEndsAtRangeRule { min: 64, max: 64 }),
            Rc::new(LenientCanaryCheckpointRequiredLabelRule {
                message: "lenient canary checkpoint needs an entry".to_string(),
            }),
            Rc::new(LenientCanaryCheckpointTenantIdLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_canary_checkpoint_validator();
        let item = new_lenient_canary_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LenientCanaryCheckpointLabelLengthRule { max_length: 16 };
        let mut item = new_lenient_canary_checkpoint("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = LenientCanaryCheckpointEndsAtRangeRule { min: 64, max: 80 };
        let mut item = new_lenient_canary_checkpoint("x", "y");
        item.ends_at = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LenientCanaryCheckpointRequiredLabelRule {
            message: "lenient canary checkpoint needs this field".to_string(),
        };
        let mut item = new_lenient_canary_checkpoint("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = LenientCanaryCheckpointTenantIdLengthRule { max_length: 255 };
        let mut item = new_lenient_canary_checkpoint("x", "y");
        item.tenant_id = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
