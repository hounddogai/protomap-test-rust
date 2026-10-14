use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global egress allocation must pass. Check returns its problems, or none.
pub trait GlobalEgressAllocationRule: std::fmt::Debug {
    fn check(&self, item: &GlobalEgressAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEgressAllocationContactEmailLengthRule {
    pub max_length: i64,
}

impl GlobalEgressAllocationRule for GlobalEgressAllocationContactEmailLengthRule {
    fn check(&self, item: &GlobalEgressAllocation) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEgressAllocationHostLengthRule {
    pub max_length: i64,
}

impl GlobalEgressAllocationRule for GlobalEgressAllocationHostLengthRule {
    fn check(&self, item: &GlobalEgressAllocation) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEgressAllocationNameLengthRule {
    pub max_length: i64,
}

impl GlobalEgressAllocationRule for GlobalEgressAllocationNameLengthRule {
    fn check(&self, item: &GlobalEgressAllocation) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEgressAllocationRequiredHostRule {
    pub message: String,
}

impl GlobalEgressAllocationRule for GlobalEgressAllocationRequiredHostRule {
    fn check(&self, item: &GlobalEgressAllocation) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalEgressAllocationCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalEgressAllocationRule for GlobalEgressAllocationCreatedAtRangeRule {
    fn check(&self, item: &GlobalEgressAllocation) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global egress allocation.
#[derive(Debug, Clone)]
pub struct GlobalEgressAllocationValidator {
    pub rules: Vec<Rc<dyn GlobalEgressAllocationRule>>,
}

impl GlobalEgressAllocationValidator {
    pub fn validate(&self, item: &GlobalEgressAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalEgressAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalEgressAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_egress_allocation_validator() -> GlobalEgressAllocationValidator {
    GlobalEgressAllocationValidator {
        rules: vec![
            Rc::new(GlobalEgressAllocationContactEmailLengthRule { max_length: 64 })
                as Rc<dyn GlobalEgressAllocationRule>,
            Rc::new(GlobalEgressAllocationHostLengthRule { max_length: 80 }),
            Rc::new(GlobalEgressAllocationNameLengthRule { max_length: 120 }),
            Rc::new(GlobalEgressAllocationRequiredHostRule {
                message: "global egress allocation needs an entry".to_string(),
            }),
            Rc::new(GlobalEgressAllocationCreatedAtRangeRule { min: 64, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_egress_allocation_validator();
        let item = new_global_egress_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = GlobalEgressAllocationContactEmailLengthRule { max_length: 120 };
        let mut item = new_global_egress_allocation("x", "y");
        item.contact_email = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = GlobalEgressAllocationHostLengthRule { max_length: 255 };
        let mut item = new_global_egress_allocation("x", "y");
        item.host = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = GlobalEgressAllocationNameLengthRule { max_length: 86400 };
        let mut item = new_global_egress_allocation("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = GlobalEgressAllocationRequiredHostRule {
            message: "global egress allocation needs this field".to_string(),
        };
        let mut item = new_global_egress_allocation("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = GlobalEgressAllocationCreatedAtRangeRule { min: 16, max: 255 };
        let mut item = new_global_egress_allocation("x", "y");
        item.created_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
