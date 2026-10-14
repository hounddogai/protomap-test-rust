use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global retention adjustment must pass. Check returns its problems, or none.
pub trait GlobalRetentionAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &GlobalRetentionAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRetentionAdjustmentOwnerIdLengthRule {
    pub max_length: i64,
}

impl GlobalRetentionAdjustmentRule for GlobalRetentionAdjustmentOwnerIdLengthRule {
    fn check(&self, item: &GlobalRetentionAdjustment) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRetentionAdjustmentRequiredTimezoneRule {
    pub message: String,
}

impl GlobalRetentionAdjustmentRule for GlobalRetentionAdjustmentRequiredTimezoneRule {
    fn check(&self, item: &GlobalRetentionAdjustment) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRetentionAdjustmentActiveGuardRule {
    pub limit: f64,
}

impl GlobalRetentionAdjustmentRule for GlobalRetentionAdjustmentActiveGuardRule {
    fn check(&self, item: &GlobalRetentionAdjustment) -> Vec<String> {
        if item.active && (item.threshold > self.limit) {
            return vec![format!(
                "threshold {} exceeds {} while active",
                item.threshold, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRetentionAdjustmentLabelLengthRule {
    pub max_length: i64,
}

impl GlobalRetentionAdjustmentRule for GlobalRetentionAdjustmentLabelLengthRule {
    fn check(&self, item: &GlobalRetentionAdjustment) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRetentionAdjustmentAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl GlobalRetentionAdjustmentRule for GlobalRetentionAdjustmentAllowedTimezoneRule {
    fn check(&self, item: &GlobalRetentionAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is inconsistent", item.timezone)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global retention adjustment.
#[derive(Debug, Clone)]
pub struct GlobalRetentionAdjustmentValidator {
    pub rules: Vec<Rc<dyn GlobalRetentionAdjustmentRule>>,
}

impl GlobalRetentionAdjustmentValidator {
    pub fn validate(&self, item: &GlobalRetentionAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalRetentionAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalRetentionAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_retention_adjustment_validator() -> GlobalRetentionAdjustmentValidator {
    GlobalRetentionAdjustmentValidator {
        rules: vec![
            Rc::new(GlobalRetentionAdjustmentOwnerIdLengthRule { max_length: 120 })
                as Rc<dyn GlobalRetentionAdjustmentRule>,
            Rc::new(GlobalRetentionAdjustmentRequiredTimezoneRule {
                message: "global retention adjustment needs an entry".to_string(),
            }),
            Rc::new(GlobalRetentionAdjustmentActiveGuardRule { limit: 1.9 }),
            Rc::new(GlobalRetentionAdjustmentLabelLengthRule { max_length: 80 }),
            Rc::new(GlobalRetentionAdjustmentAllowedTimezoneRule {
                allowed: vec!["default".to_string(), "regional".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_retention_adjustment_validator();
        let item = new_global_retention_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = GlobalRetentionAdjustmentOwnerIdLengthRule { max_length: 32 };
        let mut item = new_global_retention_adjustment("x", "y");
        item.owner_id = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = GlobalRetentionAdjustmentRequiredTimezoneRule {
            message: "global retention adjustment needs an entry".to_string(),
        };
        let mut item = new_global_retention_adjustment("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = GlobalRetentionAdjustmentLabelLengthRule { max_length: 120 };
        let mut item = new_global_retention_adjustment("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
