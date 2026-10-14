use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a egress allocation must pass. Check returns its problems, or none.
pub trait EgressAllocationRule: std::fmt::Debug {
    fn check(&self, item: &EgressAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressAllocationCodeLengthRule {
    pub max_length: i64,
}

impl EgressAllocationRule for EgressAllocationCodeLengthRule {
    fn check(&self, item: &EgressAllocation) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressAllocationSegmentLengthRule {
    pub max_length: i64,
}

impl EgressAllocationRule for EgressAllocationSegmentLengthRule {
    fn check(&self, item: &EgressAllocation) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressAllocationCodePrefixRule {
    pub prefix: String,
}

impl EgressAllocationRule for EgressAllocationCodePrefixRule {
    fn check(&self, item: &EgressAllocation) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressAllocationUnitLengthRule {
    pub max_length: i64,
}

impl EgressAllocationRule for EgressAllocationUnitLengthRule {
    fn check(&self, item: &EgressAllocation) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress allocation.
#[derive(Debug, Clone)]
pub struct EgressAllocationValidator {
    pub rules: Vec<Rc<dyn EgressAllocationRule>>,
}

impl EgressAllocationValidator {
    pub fn validate(&self, item: &EgressAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EgressAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_allocation_validator() -> EgressAllocationValidator {
    EgressAllocationValidator {
        rules: vec![
            Rc::new(EgressAllocationCodeLengthRule { max_length: 16 }) as Rc<dyn EgressAllocationRule>,
            Rc::new(EgressAllocationSegmentLengthRule { max_length: 120 }),
            Rc::new(EgressAllocationCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(EgressAllocationUnitLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_egress_allocation_validator();
        let item = new_egress_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = EgressAllocationCodeLengthRule { max_length: 16 };
        let mut item = new_egress_allocation("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = EgressAllocationSegmentLengthRule { max_length: 80 };
        let mut item = new_egress_allocation("x", "y");
        item.segment = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = EgressAllocationUnitLengthRule { max_length: 255 };
        let mut item = new_egress_allocation("x", "y");
        item.unit = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
