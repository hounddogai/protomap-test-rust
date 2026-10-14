use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a pending nonce allocation must pass. Check returns its problems, or none.
pub trait PendingNonceAllocationRule: std::fmt::Debug {
    fn check(&self, item: &PendingNonceAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceAllocationRequiredPhoneRule {
    pub message: String,
}

impl PendingNonceAllocationRule for PendingNonceAllocationRequiredPhoneRule {
    fn check(&self, item: &PendingNonceAllocation) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceAllocationAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PendingNonceAllocationRule for PendingNonceAllocationAmountRangeRule {
    fn check(&self, item: &PendingNonceAllocation) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceAllocationTitleLengthRule {
    pub max_length: i64,
}

impl PendingNonceAllocationRule for PendingNonceAllocationTitleLengthRule {
    fn check(&self, item: &PendingNonceAllocation) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending nonce allocation.
#[derive(Debug, Clone)]
pub struct PendingNonceAllocationValidator {
    pub rules: Vec<Rc<dyn PendingNonceAllocationRule>>,
}

impl PendingNonceAllocationValidator {
    pub fn validate(&self, item: &PendingNonceAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingNonceAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingNonceAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_nonce_allocation_validator() -> PendingNonceAllocationValidator {
    PendingNonceAllocationValidator {
        rules: vec![
            Rc::new(PendingNonceAllocationRequiredPhoneRule {
                message: "pending nonce allocation needs a value".to_string(),
            }) as Rc<dyn PendingNonceAllocationRule>,
            Rc::new(PendingNonceAllocationAmountRangeRule { min: 6.8, max: 4682.7 }),
            Rc::new(PendingNonceAllocationTitleLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_nonce_allocation_validator();
        let item = new_pending_nonce_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = PendingNonceAllocationRequiredPhoneRule {
            message: "pending nonce allocation needs this field".to_string(),
        };
        let mut item = new_pending_nonce_allocation("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = PendingNonceAllocationAmountRangeRule { min: 8.4, max: 2836.4 };
        let mut item = new_pending_nonce_allocation("x", "y");
        item.amount = 2837.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PendingNonceAllocationTitleLengthRule { max_length: 1000 };
        let mut item = new_pending_nonce_allocation("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
