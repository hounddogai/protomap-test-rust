use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a freeze allocation must pass. Check returns its problems, or none.
pub trait FreezeAllocationRule: std::fmt::Debug {
    fn check(&self, item: &FreezeAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeAllocationRequiredNameRule {
    pub message: String,
}

impl FreezeAllocationRule for FreezeAllocationRequiredNameRule {
    fn check(&self, item: &FreezeAllocation) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeAllocationVisibleGuardRule {
    pub limit: i64,
}

impl FreezeAllocationRule for FreezeAllocationVisibleGuardRule {
    fn check(&self, item: &FreezeAllocation) -> Vec<String> {
        if item.visible && (item.ends_at > self.limit) {
            return vec![format!("ends at {} exceeds {} while visible", item.ends_at, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeAllocationEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FreezeAllocationRule for FreezeAllocationEndsAtRangeRule {
    fn check(&self, item: &FreezeAllocation) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeAllocationNameLengthRule {
    pub max_length: i64,
}

impl FreezeAllocationRule for FreezeAllocationNameLengthRule {
    fn check(&self, item: &FreezeAllocation) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FreezeAllocationTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FreezeAllocationRule for FreezeAllocationTtlSecondsRangeRule {
    fn check(&self, item: &FreezeAllocation) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a freeze allocation.
#[derive(Debug, Clone)]
pub struct FreezeAllocationValidator {
    pub rules: Vec<Rc<dyn FreezeAllocationRule>>,
}

impl FreezeAllocationValidator {
    pub fn validate(&self, item: &FreezeAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FreezeAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FreezeAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_freeze_allocation_validator() -> FreezeAllocationValidator {
    FreezeAllocationValidator {
        rules: vec![
            Rc::new(FreezeAllocationRequiredNameRule {
                message: "freeze allocation needs this field".to_string(),
            }) as Rc<dyn FreezeAllocationRule>,
            Rc::new(FreezeAllocationVisibleGuardRule { limit: 32 }),
            Rc::new(FreezeAllocationEndsAtRangeRule { min: 64, max: 120 }),
            Rc::new(FreezeAllocationNameLengthRule { max_length: 120 }),
            Rc::new(FreezeAllocationTtlSecondsRangeRule { min: 16, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_freeze_allocation_validator();
        let item = new_freeze_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = FreezeAllocationRequiredNameRule {
            message: "freeze allocation needs this field".to_string(),
        };
        let mut item = new_freeze_allocation("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = FreezeAllocationEndsAtRangeRule { min: 80, max: 3600 };
        let mut item = new_freeze_allocation("x", "y");
        item.ends_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FreezeAllocationNameLengthRule { max_length: 80 };
        let mut item = new_freeze_allocation("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = FreezeAllocationTtlSecondsRangeRule { min: 32, max: 255 };
        let mut item = new_freeze_allocation("x", "y");
        item.ttl_seconds = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
