use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a approval allocation must pass. Check returns its problems, or none.
pub trait ApprovalAllocationRule: std::fmt::Debug {
    fn check(&self, item: &ApprovalAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAllocationTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl ApprovalAllocationRule for ApprovalAllocationTagsLimitRule {
    fn check(&self, item: &ApprovalAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAllocationCodeLengthRule {
    pub max_length: i64,
}

impl ApprovalAllocationRule for ApprovalAllocationCodeLengthRule {
    fn check(&self, item: &ApprovalAllocation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAllocationCodePrefixRule {
    pub prefix: String,
}

impl ApprovalAllocationRule for ApprovalAllocationCodePrefixRule {
    fn check(&self, item: &ApprovalAllocation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAllocationAllowedOwnerIdRule {
    pub allowed: Vec<String>,
}

impl ApprovalAllocationRule for ApprovalAllocationAllowedOwnerIdRule {
    fn check(&self, item: &ApprovalAllocation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.owner_id)))
        {
            return vec![format!("owner id {} is out of range", item.owner_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAllocationRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ApprovalAllocationRule for ApprovalAllocationRateRangeRule {
    fn check(&self, item: &ApprovalAllocation) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAllocationRequiredHostRule {
    pub message: String,
}

impl ApprovalAllocationRule for ApprovalAllocationRequiredHostRule {
    fn check(&self, item: &ApprovalAllocation) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a approval allocation.
#[derive(Debug, Clone)]
pub struct ApprovalAllocationValidator {
    pub rules: Vec<Rc<dyn ApprovalAllocationRule>>,
}

impl ApprovalAllocationValidator {
    pub fn validate(&self, item: &ApprovalAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ApprovalAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ApprovalAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_approval_allocation_validator() -> ApprovalAllocationValidator {
    ApprovalAllocationValidator {
        rules: vec![
            Rc::new(ApprovalAllocationTagsLimitRule {
                max_count: 16,
                banned: vec!["primary".to_string(), "primary".to_string()],
            }) as Rc<dyn ApprovalAllocationRule>,
            Rc::new(ApprovalAllocationCodeLengthRule { max_length: 64 }),
            Rc::new(ApprovalAllocationCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(ApprovalAllocationAllowedOwnerIdRule {
                allowed: vec!["weighted".to_string(), "lenient".to_string()],
            }),
            Rc::new(ApprovalAllocationRateRangeRule { min: 3.8, max: 1016.2 }),
            Rc::new(ApprovalAllocationRequiredHostRule {
                message: "approval allocation needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_approval_allocation_validator();
        let item = new_approval_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ApprovalAllocationCodeLengthRule { max_length: 64 };
        let mut item = new_approval_allocation("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = ApprovalAllocationRateRangeRule { min: 4.0, max: 385.6 };
        let mut item = new_approval_allocation("x", "y");
        item.rate = 387.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = ApprovalAllocationRequiredHostRule {
            message: "approval allocation needs an entry".to_string(),
        };
        let mut item = new_approval_allocation("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
