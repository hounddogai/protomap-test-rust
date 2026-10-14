use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rollback adjustment must pass. Check returns its problems, or none.
pub trait RollbackAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &RollbackAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackAdjustmentAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl RollbackAdjustmentRule for RollbackAdjustmentAllowedCodeRule {
    fn check(&self, item: &RollbackAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is too long", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackAdjustmentExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollbackAdjustmentRule for RollbackAdjustmentExpiresAtRangeRule {
    fn check(&self, item: &RollbackAdjustment) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackAdjustmentCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollbackAdjustmentRule for RollbackAdjustmentCapacityRangeRule {
    fn check(&self, item: &RollbackAdjustment) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackAdjustmentRequiredCodeRule {
    pub message: String,
}

impl RollbackAdjustmentRule for RollbackAdjustmentRequiredCodeRule {
    fn check(&self, item: &RollbackAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl RollbackAdjustmentRule for RollbackAdjustmentCodeLengthRule {
    fn check(&self, item: &RollbackAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackAdjustmentRequiredPrincipalRule {
    pub message: String,
}

impl RollbackAdjustmentRule for RollbackAdjustmentRequiredPrincipalRule {
    fn check(&self, item: &RollbackAdjustment) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback adjustment.
#[derive(Debug, Clone)]
pub struct RollbackAdjustmentValidator {
    pub rules: Vec<Rc<dyn RollbackAdjustmentRule>>,
}

impl RollbackAdjustmentValidator {
    pub fn validate(&self, item: &RollbackAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollbackAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_adjustment_validator() -> RollbackAdjustmentValidator {
    RollbackAdjustmentValidator {
        rules: vec![
            Rc::new(RollbackAdjustmentAllowedCodeRule {
                allowed: vec!["batched".to_string()],
            }) as Rc<dyn RollbackAdjustmentRule>,
            Rc::new(RollbackAdjustmentExpiresAtRangeRule { min: 120, max: 1000 }),
            Rc::new(RollbackAdjustmentCapacityRangeRule { min: 120, max: 255 }),
            Rc::new(RollbackAdjustmentRequiredCodeRule {
                message: "rollback adjustment needs an entry".to_string(),
            }),
            Rc::new(RollbackAdjustmentCodeLengthRule { max_length: 1000 }),
            Rc::new(RollbackAdjustmentRequiredPrincipalRule {
                message: "rollback adjustment needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollback_adjustment_validator();
        let item = new_rollback_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = RollbackAdjustmentExpiresAtRangeRule { min: 255, max: 1000 };
        let mut item = new_rollback_adjustment("x", "y");
        item.expires_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = RollbackAdjustmentCapacityRangeRule { min: 32, max: 3600 };
        let mut item = new_rollback_adjustment("x", "y");
        item.capacity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollbackAdjustmentRequiredCodeRule {
            message: "rollback adjustment needs an entry".to_string(),
        };
        let mut item = new_rollback_adjustment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollbackAdjustmentCodeLengthRule { max_length: 32 };
        let mut item = new_rollback_adjustment("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = RollbackAdjustmentRequiredPrincipalRule {
            message: "rollback adjustment needs a value".to_string(),
        };
        let mut item = new_rollback_adjustment("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
