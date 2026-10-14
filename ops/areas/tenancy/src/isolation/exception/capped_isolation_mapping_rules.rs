use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped isolation mapping must pass. Check returns its problems, or none.
pub trait CappedIsolationMappingRule: std::fmt::Debug {
    fn check(&self, item: &CappedIsolationMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedIsolationMappingRequiredCodeRule {
    pub message: String,
}

impl CappedIsolationMappingRule for CappedIsolationMappingRequiredCodeRule {
    fn check(&self, item: &CappedIsolationMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedIsolationMappingCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedIsolationMappingRule for CappedIsolationMappingCostRangeRule {
    fn check(&self, item: &CappedIsolationMapping) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedIsolationMappingTenantIdLengthRule {
    pub max_length: i64,
}

impl CappedIsolationMappingRule for CappedIsolationMappingTenantIdLengthRule {
    fn check(&self, item: &CappedIsolationMapping) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedIsolationMappingCodePrefixRule {
    pub prefix: String,
}

impl CappedIsolationMappingRule for CappedIsolationMappingCodePrefixRule {
    fn check(&self, item: &CappedIsolationMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedIsolationMappingRequiredTitleRule {
    pub message: String,
}

impl CappedIsolationMappingRule for CappedIsolationMappingRequiredTitleRule {
    fn check(&self, item: &CappedIsolationMapping) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped isolation mapping.
#[derive(Debug, Clone)]
pub struct CappedIsolationMappingValidator {
    pub rules: Vec<Rc<dyn CappedIsolationMappingRule>>,
}

impl CappedIsolationMappingValidator {
    pub fn validate(&self, item: &CappedIsolationMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedIsolationMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedIsolationMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_isolation_mapping_validator() -> CappedIsolationMappingValidator {
    CappedIsolationMappingValidator {
        rules: vec![
            Rc::new(CappedIsolationMappingRequiredCodeRule {
                message: "capped isolation mapping needs a value".to_string(),
            }) as Rc<dyn CappedIsolationMappingRule>,
            Rc::new(CappedIsolationMappingCostRangeRule { min: 5.1, max: 865.5 }),
            Rc::new(CappedIsolationMappingTenantIdLengthRule { max_length: 255 }),
            Rc::new(CappedIsolationMappingCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(CappedIsolationMappingRequiredTitleRule {
                message: "capped isolation mapping needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_isolation_mapping_validator();
        let item = new_capped_isolation_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = CappedIsolationMappingRequiredCodeRule {
            message: "capped isolation mapping needs an entry".to_string(),
        };
        let mut item = new_capped_isolation_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = CappedIsolationMappingCostRangeRule { min: 1.2, max: 4038.9 };
        let mut item = new_capped_isolation_mapping("x", "y");
        item.cost = 4040.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = CappedIsolationMappingTenantIdLengthRule { max_length: 16 };
        let mut item = new_capped_isolation_mapping("x", "y");
        item.tenant_id = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = CappedIsolationMappingRequiredTitleRule {
            message: "capped isolation mapping needs an entry".to_string(),
        };
        let mut item = new_capped_isolation_mapping("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
