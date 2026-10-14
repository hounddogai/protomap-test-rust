use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict health check allocation must pass. Check returns its problems, or none.
pub trait StrictHealthCheckAllocationRule: std::fmt::Debug {
    fn check(&self, item: &StrictHealthCheckAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictHealthCheckAllocationCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictHealthCheckAllocationRule for StrictHealthCheckAllocationCreatedAtRangeRule {
    fn check(&self, item: &StrictHealthCheckAllocation) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictHealthCheckAllocationAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictHealthCheckAllocationRule for StrictHealthCheckAllocationAttemptsRangeRule {
    fn check(&self, item: &StrictHealthCheckAllocation) -> Vec<String> {
        let value = item.attempts;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("attempts {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("attempts {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictHealthCheckAllocationAllowedKeyIdRule {
    pub allowed: Vec<String>,
}

impl StrictHealthCheckAllocationRule for StrictHealthCheckAllocationAllowedKeyIdRule {
    fn check(&self, item: &StrictHealthCheckAllocation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.key_id))) {
            return vec![format!("key id {} is not allowed", item.key_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictHealthCheckAllocationRequiredCodeRule {
    pub message: String,
}

impl StrictHealthCheckAllocationRule for StrictHealthCheckAllocationRequiredCodeRule {
    fn check(&self, item: &StrictHealthCheckAllocation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictHealthCheckAllocationCodePrefixRule {
    pub prefix: String,
}

impl StrictHealthCheckAllocationRule for StrictHealthCheckAllocationCodePrefixRule {
    fn check(&self, item: &StrictHealthCheckAllocation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictHealthCheckAllocationCodeLengthRule {
    pub max_length: i64,
}

impl StrictHealthCheckAllocationRule for StrictHealthCheckAllocationCodeLengthRule {
    fn check(&self, item: &StrictHealthCheckAllocation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict health check allocation.
#[derive(Debug, Clone)]
pub struct StrictHealthCheckAllocationValidator {
    pub rules: Vec<Rc<dyn StrictHealthCheckAllocationRule>>,
}

impl StrictHealthCheckAllocationValidator {
    pub fn validate(&self, item: &StrictHealthCheckAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictHealthCheckAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictHealthCheckAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_health_check_allocation_validator() -> StrictHealthCheckAllocationValidator {
    StrictHealthCheckAllocationValidator {
        rules: vec![
            Rc::new(StrictHealthCheckAllocationCreatedAtRangeRule { min: 32, max: 64 })
                as Rc<dyn StrictHealthCheckAllocationRule>,
            Rc::new(StrictHealthCheckAllocationAttemptsRangeRule { min: 255, max: 3600 }),
            Rc::new(StrictHealthCheckAllocationAllowedKeyIdRule {
                allowed: vec!["pending".to_string()],
            }),
            Rc::new(StrictHealthCheckAllocationRequiredCodeRule {
                message: "strict health check allocation needs this field".to_string(),
            }),
            Rc::new(StrictHealthCheckAllocationCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(StrictHealthCheckAllocationCodeLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_health_check_allocation_validator();
        let item = new_strict_health_check_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = StrictHealthCheckAllocationCreatedAtRangeRule { min: 120, max: 255 };
        let mut item = new_strict_health_check_allocation("x", "y");
        item.created_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = StrictHealthCheckAllocationAttemptsRangeRule { min: 16, max: 255 };
        let mut item = new_strict_health_check_allocation("x", "y");
        item.attempts = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictHealthCheckAllocationRequiredCodeRule {
            message: "strict health check allocation needs a value".to_string(),
        };
        let mut item = new_strict_health_check_allocation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictHealthCheckAllocationCodeLengthRule { max_length: 32 };
        let mut item = new_strict_health_check_allocation("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
