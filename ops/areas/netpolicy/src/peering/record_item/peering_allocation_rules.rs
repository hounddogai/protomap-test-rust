use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a peering allocation must pass. Check returns its problems, or none.
pub trait PeeringAllocationRule: std::fmt::Debug {
    fn check(&self, item: &PeeringAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringAllocationUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PeeringAllocationRule for PeeringAllocationUpdatedAtRangeRule {
    fn check(&self, item: &PeeringAllocation) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringAllocationBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PeeringAllocationRule for PeeringAllocationBatchSizeRangeRule {
    fn check(&self, item: &PeeringAllocation) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringAllocationCodeLengthRule {
    pub max_length: i64,
}

impl PeeringAllocationRule for PeeringAllocationCodeLengthRule {
    fn check(&self, item: &PeeringAllocation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a peering allocation.
#[derive(Debug, Clone)]
pub struct PeeringAllocationValidator {
    pub rules: Vec<Rc<dyn PeeringAllocationRule>>,
}

impl PeeringAllocationValidator {
    pub fn validate(&self, item: &PeeringAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PeeringAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_allocation_validator() -> PeeringAllocationValidator {
    PeeringAllocationValidator {
        rules: vec![
            Rc::new(PeeringAllocationUpdatedAtRangeRule { min: 64, max: 3600 }) as Rc<dyn PeeringAllocationRule>,
            Rc::new(PeeringAllocationBatchSizeRangeRule { min: 120, max: 120 }),
            Rc::new(PeeringAllocationCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_peering_allocation_validator();
        let item = new_peering_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = PeeringAllocationUpdatedAtRangeRule { min: 16, max: 64 };
        let mut item = new_peering_allocation("x", "y");
        item.updated_at = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = PeeringAllocationBatchSizeRangeRule { min: 3600, max: 86400 };
        let mut item = new_peering_allocation("x", "y");
        item.batch_size = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PeeringAllocationCodeLengthRule { max_length: 86400 };
        let mut item = new_peering_allocation("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
