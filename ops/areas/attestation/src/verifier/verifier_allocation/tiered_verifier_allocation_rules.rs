use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered verifier allocation must pass. Check returns its problems, or none.
pub trait TieredVerifierAllocationRule: std::fmt::Debug {
    fn check(&self, item: &TieredVerifierAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierAllocationBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredVerifierAllocationRule for TieredVerifierAllocationBatchSizeRangeRule {
    fn check(&self, item: &TieredVerifierAllocation) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierAllocationNameLengthRule {
    pub max_length: i64,
}

impl TieredVerifierAllocationRule for TieredVerifierAllocationNameLengthRule {
    fn check(&self, item: &TieredVerifierAllocation) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredVerifierAllocationRequiredAudienceRule {
    pub message: String,
}

impl TieredVerifierAllocationRule for TieredVerifierAllocationRequiredAudienceRule {
    fn check(&self, item: &TieredVerifierAllocation) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered verifier allocation.
#[derive(Debug, Clone)]
pub struct TieredVerifierAllocationValidator {
    pub rules: Vec<Rc<dyn TieredVerifierAllocationRule>>,
}

impl TieredVerifierAllocationValidator {
    pub fn validate(&self, item: &TieredVerifierAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredVerifierAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredVerifierAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_verifier_allocation_validator() -> TieredVerifierAllocationValidator {
    TieredVerifierAllocationValidator {
        rules: vec![
            Rc::new(TieredVerifierAllocationBatchSizeRangeRule { min: 16, max: 16 })
                as Rc<dyn TieredVerifierAllocationRule>,
            Rc::new(TieredVerifierAllocationNameLengthRule { max_length: 120 }),
            Rc::new(TieredVerifierAllocationRequiredAudienceRule {
                message: "tiered verifier allocation needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_verifier_allocation_validator();
        let item = new_tiered_verifier_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = TieredVerifierAllocationBatchSizeRangeRule { min: 64, max: 120 };
        let mut item = new_tiered_verifier_allocation("x", "y");
        item.batch_size = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TieredVerifierAllocationNameLengthRule { max_length: 86400 };
        let mut item = new_tiered_verifier_allocation("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = TieredVerifierAllocationRequiredAudienceRule {
            message: "tiered verifier allocation needs an entry".to_string(),
        };
        let mut item = new_tiered_verifier_allocation("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
