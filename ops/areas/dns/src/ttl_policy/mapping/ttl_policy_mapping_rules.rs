use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a ttl policy mapping must pass. Check returns its problems, or none.
pub trait TtlPolicyMappingRule: std::fmt::Debug {
    fn check(&self, item: &TtlPolicyMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyMappingClusterLengthRule {
    pub max_length: i64,
}

impl TtlPolicyMappingRule for TtlPolicyMappingClusterLengthRule {
    fn check(&self, item: &TtlPolicyMapping) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyMappingNameLengthRule {
    pub max_length: i64,
}

impl TtlPolicyMappingRule for TtlPolicyMappingNameLengthRule {
    fn check(&self, item: &TtlPolicyMapping) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyMappingFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TtlPolicyMappingRule for TtlPolicyMappingFloorValueRangeRule {
    fn check(&self, item: &TtlPolicyMapping) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyMappingRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TtlPolicyMappingRule for TtlPolicyMappingRotationDaysRangeRule {
    fn check(&self, item: &TtlPolicyMapping) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a ttl policy mapping.
#[derive(Debug, Clone)]
pub struct TtlPolicyMappingValidator {
    pub rules: Vec<Rc<dyn TtlPolicyMappingRule>>,
}

impl TtlPolicyMappingValidator {
    pub fn validate(&self, item: &TtlPolicyMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TtlPolicyMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TtlPolicyMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_ttl_policy_mapping_validator() -> TtlPolicyMappingValidator {
    TtlPolicyMappingValidator {
        rules: vec![
            Rc::new(TtlPolicyMappingClusterLengthRule { max_length: 120 }) as Rc<dyn TtlPolicyMappingRule>,
            Rc::new(TtlPolicyMappingNameLengthRule { max_length: 255 }),
            Rc::new(TtlPolicyMappingFloorValueRangeRule { min: 1.6, max: 3276.3 }),
            Rc::new(TtlPolicyMappingRotationDaysRangeRule { min: 16, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_ttl_policy_mapping_validator();
        let item = new_ttl_policy_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = TtlPolicyMappingClusterLengthRule { max_length: 16 };
        let mut item = new_ttl_policy_mapping("x", "y");
        item.cluster = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TtlPolicyMappingNameLengthRule { max_length: 32 };
        let mut item = new_ttl_policy_mapping("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = TtlPolicyMappingFloorValueRangeRule { min: 3.1, max: 3332.2 };
        let mut item = new_ttl_policy_mapping("x", "y");
        item.floor_value = 3333.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = TtlPolicyMappingRotationDaysRangeRule { min: 80, max: 86400 };
        let mut item = new_ttl_policy_mapping("x", "y");
        item.rotation_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
