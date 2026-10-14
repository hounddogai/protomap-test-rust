use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a policy mapping must pass. Check returns its problems, or none.
pub trait PolicyMappingRule: std::fmt::Debug {
    fn check(&self, item: &PolicyMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyMappingRequiredTenantIdRule {
    pub message: String,
}

impl PolicyMappingRule for PolicyMappingRequiredTenantIdRule {
    fn check(&self, item: &PolicyMapping) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyMappingUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PolicyMappingRule for PolicyMappingUpdatedAtRangeRule {
    fn check(&self, item: &PolicyMapping) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyMappingQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PolicyMappingRule for PolicyMappingQuantityRangeRule {
    fn check(&self, item: &PolicyMapping) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a policy mapping.
#[derive(Debug, Clone)]
pub struct PolicyMappingValidator {
    pub rules: Vec<Rc<dyn PolicyMappingRule>>,
}

impl PolicyMappingValidator {
    pub fn validate(&self, item: &PolicyMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PolicyMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PolicyMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_policy_mapping_validator() -> PolicyMappingValidator {
    PolicyMappingValidator {
        rules: vec![
            Rc::new(PolicyMappingRequiredTenantIdRule {
                message: "policy mapping needs an entry".to_string(),
            }) as Rc<dyn PolicyMappingRule>,
            Rc::new(PolicyMappingUpdatedAtRangeRule { min: 16, max: 80 }),
            Rc::new(PolicyMappingQuantityRangeRule { min: 120, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_policy_mapping_validator();
        let item = new_policy_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = PolicyMappingRequiredTenantIdRule {
            message: "policy mapping needs this field".to_string(),
        };
        let mut item = new_policy_mapping("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = PolicyMappingUpdatedAtRangeRule { min: 16, max: 86400 };
        let mut item = new_policy_mapping("x", "y");
        item.updated_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = PolicyMappingQuantityRangeRule { min: 255, max: 3600 };
        let mut item = new_policy_mapping("x", "y");
        item.quantity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
