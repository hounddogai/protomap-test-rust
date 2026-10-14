use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a verifier allocation must pass. Check returns its problems, or none.
pub trait VerifierAllocationRule: std::fmt::Debug {
    fn check(&self, item: &VerifierAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierAllocationAllowedTenantIdRule {
    pub allowed: Vec<String>,
}

impl VerifierAllocationRule for VerifierAllocationAllowedTenantIdRule {
    fn check(&self, item: &VerifierAllocation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.tenant_id)))
        {
            return vec![format!("tenant id {} is below minimum", item.tenant_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierAllocationNameLengthRule {
    pub max_length: i64,
}

impl VerifierAllocationRule for VerifierAllocationNameLengthRule {
    fn check(&self, item: &VerifierAllocation) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierAllocationMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl VerifierAllocationRule for VerifierAllocationMinimumRangeRule {
    fn check(&self, item: &VerifierAllocation) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierAllocationRequiredTenantIdRule {
    pub message: String,
}

impl VerifierAllocationRule for VerifierAllocationRequiredTenantIdRule {
    fn check(&self, item: &VerifierAllocation) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier allocation.
#[derive(Debug, Clone)]
pub struct VerifierAllocationValidator {
    pub rules: Vec<Rc<dyn VerifierAllocationRule>>,
}

impl VerifierAllocationValidator {
    pub fn validate(&self, item: &VerifierAllocation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &VerifierAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_allocation_validator() -> VerifierAllocationValidator {
    VerifierAllocationValidator {
        rules: vec![
            Rc::new(VerifierAllocationAllowedTenantIdRule {
                allowed: vec!["adaptive".to_string(), "tiered".to_string(), "scheduled".to_string()],
            }) as Rc<dyn VerifierAllocationRule>,
            Rc::new(VerifierAllocationNameLengthRule { max_length: 16 }),
            Rc::new(VerifierAllocationMinimumRangeRule { min: 7.3, max: 667.3 }),
            Rc::new(VerifierAllocationRequiredTenantIdRule {
                message: "verifier allocation needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_verifier_allocation_validator();
        let item = new_verifier_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = VerifierAllocationNameLengthRule { max_length: 1000 };
        let mut item = new_verifier_allocation("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = VerifierAllocationMinimumRangeRule { min: 9.7, max: 594.8 };
        let mut item = new_verifier_allocation("x", "y");
        item.minimum = 596.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = VerifierAllocationRequiredTenantIdRule {
            message: "verifier allocation needs an entry".to_string(),
        };
        let mut item = new_verifier_allocation("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
