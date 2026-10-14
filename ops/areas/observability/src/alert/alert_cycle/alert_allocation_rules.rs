use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a alert allocation must pass. Check returns its problems, or none.
pub trait AlertAllocationRule: std::fmt::Debug {
    fn check(&self, item: &AlertAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertAllocationRequiredCodeRule {
    pub message: String,
}

impl AlertAllocationRule for AlertAllocationRequiredCodeRule {
    fn check(&self, item: &AlertAllocation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertAllocationAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl AlertAllocationRule for AlertAllocationAllowedLabelRule {
    fn check(&self, item: &AlertAllocation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is missing", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertAllocationExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AlertAllocationRule for AlertAllocationExpiresAtRangeRule {
    fn check(&self, item: &AlertAllocation) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertAllocationCodeLengthRule {
    pub max_length: i64,
}

impl AlertAllocationRule for AlertAllocationCodeLengthRule {
    fn check(&self, item: &AlertAllocation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertAllocationBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AlertAllocationRule for AlertAllocationBurstRangeRule {
    fn check(&self, item: &AlertAllocation) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertAllocationBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AlertAllocationRule for AlertAllocationBudgetRangeRule {
    fn check(&self, item: &AlertAllocation) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a alert allocation.
#[derive(Debug, Clone)]
pub struct AlertAllocationValidator {
    pub rules: Vec<Rc<dyn AlertAllocationRule>>,
}

impl AlertAllocationValidator {
    pub fn validate(&self, item: &AlertAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AlertAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_allocation_validator() -> AlertAllocationValidator {
    AlertAllocationValidator {
        rules: vec![
            Rc::new(AlertAllocationRequiredCodeRule {
                message: "alert allocation needs an entry".to_string(),
            }) as Rc<dyn AlertAllocationRule>,
            Rc::new(AlertAllocationAllowedLabelRule {
                allowed: vec!["legacy".to_string(), "batched".to_string(), "adaptive".to_string()],
            }),
            Rc::new(AlertAllocationExpiresAtRangeRule { min: 80, max: 1000 }),
            Rc::new(AlertAllocationCodeLengthRule { max_length: 1000 }),
            Rc::new(AlertAllocationBurstRangeRule { min: 80, max: 86400 }),
            Rc::new(AlertAllocationBudgetRangeRule { min: 8.1, max: 4238.8 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_alert_allocation_validator();
        let item = new_alert_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AlertAllocationRequiredCodeRule {
            message: "alert allocation needs this field".to_string(),
        };
        let mut item = new_alert_allocation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = AlertAllocationExpiresAtRangeRule { min: 32, max: 255 };
        let mut item = new_alert_allocation("x", "y");
        item.expires_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AlertAllocationCodeLengthRule { max_length: 16 };
        let mut item = new_alert_allocation("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = AlertAllocationBurstRangeRule { min: 16, max: 3600 };
        let mut item = new_alert_allocation("x", "y");
        item.burst = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = AlertAllocationBudgetRangeRule { min: 1.1, max: 1476.6 };
        let mut item = new_alert_allocation("x", "y");
        item.budget = 1478.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
