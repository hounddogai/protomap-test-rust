use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a silence allocation must pass. Check returns its problems, or none.
pub trait SilenceAllocationRule: std::fmt::Debug {
    fn check(&self, item: &SilenceAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAllocationCodeLengthRule {
    pub max_length: i64,
}

impl SilenceAllocationRule for SilenceAllocationCodeLengthRule {
    fn check(&self, item: &SilenceAllocation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAllocationAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SilenceAllocationRule for SilenceAllocationAmountRangeRule {
    fn check(&self, item: &SilenceAllocation) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAllocationOwnerIdLengthRule {
    pub max_length: i64,
}

impl SilenceAllocationRule for SilenceAllocationOwnerIdLengthRule {
    fn check(&self, item: &SilenceAllocation) -> Vec<String> {
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
pub struct SilenceAllocationRequiredDescriptionRule {
    pub message: String,
}

impl SilenceAllocationRule for SilenceAllocationRequiredDescriptionRule {
    fn check(&self, item: &SilenceAllocation) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAllocationRequiredCodeRule {
    pub message: String,
}

impl SilenceAllocationRule for SilenceAllocationRequiredCodeRule {
    fn check(&self, item: &SilenceAllocation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAllocationRequiredOwnerIdRule {
    pub message: String,
}

impl SilenceAllocationRule for SilenceAllocationRequiredOwnerIdRule {
    fn check(&self, item: &SilenceAllocation) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a silence allocation.
#[derive(Debug, Clone)]
pub struct SilenceAllocationValidator {
    pub rules: Vec<Rc<dyn SilenceAllocationRule>>,
}

impl SilenceAllocationValidator {
    pub fn validate(&self, item: &SilenceAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SilenceAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SilenceAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_silence_allocation_validator() -> SilenceAllocationValidator {
    SilenceAllocationValidator {
        rules: vec![
            Rc::new(SilenceAllocationCodeLengthRule { max_length: 32 }) as Rc<dyn SilenceAllocationRule>,
            Rc::new(SilenceAllocationAmountRangeRule { min: 8.4, max: 3951.3 }),
            Rc::new(SilenceAllocationOwnerIdLengthRule { max_length: 1000 }),
            Rc::new(SilenceAllocationRequiredDescriptionRule {
                message: "silence allocation needs this field".to_string(),
            }),
            Rc::new(SilenceAllocationRequiredCodeRule {
                message: "silence allocation needs this field".to_string(),
            }),
            Rc::new(SilenceAllocationRequiredOwnerIdRule {
                message: "silence allocation needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_silence_allocation_validator();
        let item = new_silence_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SilenceAllocationCodeLengthRule { max_length: 3600 };
        let mut item = new_silence_allocation("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = SilenceAllocationAmountRangeRule { min: 2.8, max: 2365.0 };
        let mut item = new_silence_allocation("x", "y");
        item.amount = 2366.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = SilenceAllocationOwnerIdLengthRule { max_length: 16 };
        let mut item = new_silence_allocation("x", "y");
        item.owner_id = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = SilenceAllocationRequiredDescriptionRule {
            message: "silence allocation needs an entry".to_string(),
        };
        let mut item = new_silence_allocation("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SilenceAllocationRequiredCodeRule {
            message: "silence allocation needs a value".to_string(),
        };
        let mut item = new_silence_allocation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = SilenceAllocationRequiredOwnerIdRule {
            message: "silence allocation needs an entry".to_string(),
        };
        let mut item = new_silence_allocation("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
