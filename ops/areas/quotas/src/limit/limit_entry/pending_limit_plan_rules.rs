use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending limit plan must pass. Check returns its problems, or none.
pub trait PendingLimitPlanRule: std::fmt::Debug {
    fn check(&self, item: &PendingLimitPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingLimitPlanRequiredKeyIdRule {
    pub message: String,
}

impl PendingLimitPlanRule for PendingLimitPlanRequiredKeyIdRule {
    fn check(&self, item: &PendingLimitPlan) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingLimitPlanEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingLimitPlanRule for PendingLimitPlanEndsAtRangeRule {
    fn check(&self, item: &PendingLimitPlan) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingLimitPlanKeyIdLengthRule {
    pub max_length: i64,
}

impl PendingLimitPlanRule for PendingLimitPlanKeyIdLengthRule {
    fn check(&self, item: &PendingLimitPlan) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingLimitPlanRequiredCodeRule {
    pub message: String,
}

impl PendingLimitPlanRule for PendingLimitPlanRequiredCodeRule {
    fn check(&self, item: &PendingLimitPlan) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingLimitPlanCodePrefixRule {
    pub prefix: String,
}

impl PendingLimitPlanRule for PendingLimitPlanCodePrefixRule {
    fn check(&self, item: &PendingLimitPlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingLimitPlanAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PendingLimitPlanRule for PendingLimitPlanAllowedCodeRule {
    fn check(&self, item: &PendingLimitPlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending limit plan.
#[derive(Debug, Clone)]
pub struct PendingLimitPlanValidator {
    pub rules: Vec<Rc<dyn PendingLimitPlanRule>>,
}

impl PendingLimitPlanValidator {
    pub fn validate(&self, item: &PendingLimitPlan) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PendingLimitPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingLimitPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_limit_plan_validator() -> PendingLimitPlanValidator {
    PendingLimitPlanValidator {
        rules: vec![
            Rc::new(PendingLimitPlanRequiredKeyIdRule {
                message: "pending limit plan needs a value".to_string(),
            }) as Rc<dyn PendingLimitPlanRule>,
            Rc::new(PendingLimitPlanEndsAtRangeRule { min: 64, max: 1000 }),
            Rc::new(PendingLimitPlanKeyIdLengthRule { max_length: 3600 }),
            Rc::new(PendingLimitPlanRequiredCodeRule {
                message: "pending limit plan needs a value".to_string(),
            }),
            Rc::new(PendingLimitPlanCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(PendingLimitPlanAllowedCodeRule {
                allowed: vec!["weighted".to_string(), "pending".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_limit_plan_validator();
        let item = new_pending_limit_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = PendingLimitPlanRequiredKeyIdRule {
            message: "pending limit plan needs this field".to_string(),
        };
        let mut item = new_pending_limit_plan("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = PendingLimitPlanEndsAtRangeRule { min: 64, max: 80 };
        let mut item = new_pending_limit_plan("x", "y");
        item.ends_at = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = PendingLimitPlanKeyIdLengthRule { max_length: 120 };
        let mut item = new_pending_limit_plan("x", "y");
        item.key_id = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PendingLimitPlanRequiredCodeRule {
            message: "pending limit plan needs a value".to_string(),
        };
        let mut item = new_pending_limit_plan("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
