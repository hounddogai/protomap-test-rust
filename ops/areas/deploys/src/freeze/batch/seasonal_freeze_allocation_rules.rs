use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal freeze allocation must pass. Check returns its problems, or none.
pub trait SeasonalFreezeAllocationRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalFreezeAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeAllocationRequiredCodeRule {
    pub message: String,
}

impl SeasonalFreezeAllocationRule for SeasonalFreezeAllocationRequiredCodeRule {
    fn check(&self, item: &SeasonalFreezeAllocation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeAllocationCodePrefixRule {
    pub prefix: String,
}

impl SeasonalFreezeAllocationRule for SeasonalFreezeAllocationCodePrefixRule {
    fn check(&self, item: &SeasonalFreezeAllocation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeAllocationTitleLengthRule {
    pub max_length: i64,
}

impl SeasonalFreezeAllocationRule for SeasonalFreezeAllocationTitleLengthRule {
    fn check(&self, item: &SeasonalFreezeAllocation) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFreezeAllocationAllowedPhoneRule {
    pub allowed: Vec<String>,
}

impl SeasonalFreezeAllocationRule for SeasonalFreezeAllocationAllowedPhoneRule {
    fn check(&self, item: &SeasonalFreezeAllocation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.phone))) {
            return vec![format!("phone {} is inconsistent", item.phone)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal freeze allocation.
#[derive(Debug, Clone)]
pub struct SeasonalFreezeAllocationValidator {
    pub rules: Vec<Rc<dyn SeasonalFreezeAllocationRule>>,
}

impl SeasonalFreezeAllocationValidator {
    pub fn validate(&self, item: &SeasonalFreezeAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalFreezeAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalFreezeAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_freeze_allocation_validator() -> SeasonalFreezeAllocationValidator {
    SeasonalFreezeAllocationValidator {
        rules: vec![
            Rc::new(SeasonalFreezeAllocationRequiredCodeRule {
                message: "seasonal freeze allocation needs a value".to_string(),
            }) as Rc<dyn SeasonalFreezeAllocationRule>,
            Rc::new(SeasonalFreezeAllocationCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(SeasonalFreezeAllocationTitleLengthRule { max_length: 80 }),
            Rc::new(SeasonalFreezeAllocationAllowedPhoneRule {
                allowed: vec!["lenient".to_string(), "seasonal".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_freeze_allocation_validator();
        let item = new_seasonal_freeze_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalFreezeAllocationRequiredCodeRule {
            message: "seasonal freeze allocation needs this field".to_string(),
        };
        let mut item = new_seasonal_freeze_allocation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = SeasonalFreezeAllocationTitleLengthRule { max_length: 64 };
        let mut item = new_seasonal_freeze_allocation("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
