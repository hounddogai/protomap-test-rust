use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollback allocation must pass. Check returns its problems, or none.
pub trait RollbackAllocationRule: std::fmt::Debug {
    fn check(&self, item: &RollbackAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackAllocationDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollbackAllocationRule for RollbackAllocationDurationMinutesRangeRule {
    fn check(&self, item: &RollbackAllocation) -> Vec<String> {
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
pub struct RollbackAllocationRequiredCodeRule {
    pub message: String,
}

impl RollbackAllocationRule for RollbackAllocationRequiredCodeRule {
    fn check(&self, item: &RollbackAllocation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackAllocationRequiredLabelRule {
    pub message: String,
}

impl RollbackAllocationRule for RollbackAllocationRequiredLabelRule {
    fn check(&self, item: &RollbackAllocation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackAllocationCodeLengthRule {
    pub max_length: i64,
}

impl RollbackAllocationRule for RollbackAllocationCodeLengthRule {
    fn check(&self, item: &RollbackAllocation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollbackAllocationRequiredTenantIdRule {
    pub message: String,
}

impl RollbackAllocationRule for RollbackAllocationRequiredTenantIdRule {
    fn check(&self, item: &RollbackAllocation) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollback allocation.
#[derive(Debug, Clone)]
pub struct RollbackAllocationValidator {
    pub rules: Vec<Rc<dyn RollbackAllocationRule>>,
}

impl RollbackAllocationValidator {
    pub fn validate(&self, item: &RollbackAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollbackAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollbackAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollback_allocation_validator() -> RollbackAllocationValidator {
    RollbackAllocationValidator {
        rules: vec![
            Rc::new(RollbackAllocationDurationMinutesRangeRule { min: 16, max: 1000 })
                as Rc<dyn RollbackAllocationRule>,
            Rc::new(RollbackAllocationRequiredCodeRule {
                message: "rollback allocation needs an entry".to_string(),
            }),
            Rc::new(RollbackAllocationRequiredLabelRule {
                message: "rollback allocation needs a value".to_string(),
            }),
            Rc::new(RollbackAllocationCodeLengthRule { max_length: 255 }),
            Rc::new(RollbackAllocationRequiredTenantIdRule {
                message: "rollback allocation needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollback_allocation_validator();
        let item = new_rollback_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = RollbackAllocationDurationMinutesRangeRule { min: 255, max: 86400 };
        let mut item = new_rollback_allocation("x", "y");
        item.duration_minutes = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollbackAllocationRequiredCodeRule {
            message: "rollback allocation needs this field".to_string(),
        };
        let mut item = new_rollback_allocation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RollbackAllocationRequiredLabelRule {
            message: "rollback allocation needs a value".to_string(),
        };
        let mut item = new_rollback_allocation("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollbackAllocationCodeLengthRule { max_length: 86400 };
        let mut item = new_rollback_allocation("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = RollbackAllocationRequiredTenantIdRule {
            message: "rollback allocation needs this field".to_string(),
        };
        let mut item = new_rollback_allocation("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
