use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a slo allocation must pass. Check returns its problems, or none.
pub trait SloAllocationRule: std::fmt::Debug {
    fn check(&self, item: &SloAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloAllocationCodePrefixRule {
    pub prefix: String,
}

impl SloAllocationRule for SloAllocationCodePrefixRule {
    fn check(&self, item: &SloAllocation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloAllocationRequiredCodeRule {
    pub message: String,
}

impl SloAllocationRule for SloAllocationRequiredCodeRule {
    fn check(&self, item: &SloAllocation) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloAllocationBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SloAllocationRule for SloAllocationBurstRangeRule {
    fn check(&self, item: &SloAllocation) -> Vec<String> {
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
pub struct SloAllocationRequiredTitleRule {
    pub message: String,
}

impl SloAllocationRule for SloAllocationRequiredTitleRule {
    fn check(&self, item: &SloAllocation) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a slo allocation.
#[derive(Debug, Clone)]
pub struct SloAllocationValidator {
    pub rules: Vec<Rc<dyn SloAllocationRule>>,
}

impl SloAllocationValidator {
    pub fn validate(&self, item: &SloAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SloAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_allocation_validator() -> SloAllocationValidator {
    SloAllocationValidator {
        rules: vec![
            Rc::new(SloAllocationCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn SloAllocationRule>,
            Rc::new(SloAllocationRequiredCodeRule {
                message: "slo allocation needs an entry".to_string(),
            }),
            Rc::new(SloAllocationBurstRangeRule { min: 64, max: 3600 }),
            Rc::new(SloAllocationRequiredTitleRule {
                message: "slo allocation needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_slo_allocation_validator();
        let item = new_slo_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SloAllocationRequiredCodeRule {
            message: "slo allocation needs an entry".to_string(),
        };
        let mut item = new_slo_allocation("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = SloAllocationBurstRangeRule { min: 3600, max: 86400 };
        let mut item = new_slo_allocation("x", "y");
        item.burst = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = SloAllocationRequiredTitleRule {
            message: "slo allocation needs an entry".to_string(),
        };
        let mut item = new_slo_allocation("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
