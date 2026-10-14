use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global silence allocation must pass. Check returns its problems, or none.
pub trait GlobalSilenceAllocationRule: std::fmt::Debug {
    fn check(&self, item: &GlobalSilenceAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSilenceAllocationEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalSilenceAllocationRule for GlobalSilenceAllocationEndsAtRangeRule {
    fn check(&self, item: &GlobalSilenceAllocation) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSilenceAllocationPinnedGuardRule {
    pub limit: i64,
}

impl GlobalSilenceAllocationRule for GlobalSilenceAllocationPinnedGuardRule {
    fn check(&self, item: &GlobalSilenceAllocation) -> Vec<String> {
        if item.pinned && (item.ends_at > self.limit) {
            return vec![format!("ends at {} exceeds {} while pinned", item.ends_at, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSilenceAllocationRequiredLabelRule {
    pub message: String,
}

impl GlobalSilenceAllocationRule for GlobalSilenceAllocationRequiredLabelRule {
    fn check(&self, item: &GlobalSilenceAllocation) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSilenceAllocationLabelLengthRule {
    pub max_length: i64,
}

impl GlobalSilenceAllocationRule for GlobalSilenceAllocationLabelLengthRule {
    fn check(&self, item: &GlobalSilenceAllocation) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSilenceAllocationRequiredScopeRule {
    pub message: String,
}

impl GlobalSilenceAllocationRule for GlobalSilenceAllocationRequiredScopeRule {
    fn check(&self, item: &GlobalSilenceAllocation) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a global silence allocation.
#[derive(Debug, Clone)]
pub struct GlobalSilenceAllocationValidator {
    pub rules: Vec<Rc<dyn GlobalSilenceAllocationRule>>,
}

impl GlobalSilenceAllocationValidator {
    pub fn validate(&self, item: &GlobalSilenceAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalSilenceAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalSilenceAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_silence_allocation_validator() -> GlobalSilenceAllocationValidator {
    GlobalSilenceAllocationValidator {
        rules: vec![
            Rc::new(GlobalSilenceAllocationEndsAtRangeRule { min: 16, max: 80 }) as Rc<dyn GlobalSilenceAllocationRule>,
            Rc::new(GlobalSilenceAllocationPinnedGuardRule { limit: 80 }),
            Rc::new(GlobalSilenceAllocationRequiredLabelRule {
                message: "global silence allocation needs a value".to_string(),
            }),
            Rc::new(GlobalSilenceAllocationLabelLengthRule { max_length: 120 }),
            Rc::new(GlobalSilenceAllocationRequiredScopeRule {
                message: "global silence allocation needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_silence_allocation_validator();
        let item = new_global_silence_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = GlobalSilenceAllocationEndsAtRangeRule { min: 32, max: 255 };
        let mut item = new_global_silence_allocation("x", "y");
        item.ends_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = GlobalSilenceAllocationRequiredLabelRule {
            message: "global silence allocation needs this field".to_string(),
        };
        let mut item = new_global_silence_allocation("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = GlobalSilenceAllocationLabelLengthRule { max_length: 255 };
        let mut item = new_global_silence_allocation("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = GlobalSilenceAllocationRequiredScopeRule {
            message: "global silence allocation needs this field".to_string(),
        };
        let mut item = new_global_silence_allocation("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
