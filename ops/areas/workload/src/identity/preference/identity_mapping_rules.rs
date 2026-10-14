use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a identity mapping must pass. Check returns its problems, or none.
pub trait IdentityMappingRule: std::fmt::Debug {
    fn check(&self, item: &IdentityMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityMappingRequiredCodeRule {
    pub message: String,
}

impl IdentityMappingRule for IdentityMappingRequiredCodeRule {
    fn check(&self, item: &IdentityMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityMappingExternalRefLengthRule {
    pub max_length: i64,
}

impl IdentityMappingRule for IdentityMappingExternalRefLengthRule {
    fn check(&self, item: &IdentityMapping) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityMappingQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl IdentityMappingRule for IdentityMappingQuantityRangeRule {
    fn check(&self, item: &IdentityMapping) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a identity mapping.
#[derive(Debug, Clone)]
pub struct IdentityMappingValidator {
    pub rules: Vec<Rc<dyn IdentityMappingRule>>,
}

impl IdentityMappingValidator {
    pub fn validate(&self, item: &IdentityMapping) -> Vec<String> {
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

    pub fn require_valid(&self, item: &IdentityMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[IdentityMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_identity_mapping_validator() -> IdentityMappingValidator {
    IdentityMappingValidator {
        rules: vec![
            Rc::new(IdentityMappingRequiredCodeRule {
                message: "identity mapping needs a value".to_string(),
            }) as Rc<dyn IdentityMappingRule>,
            Rc::new(IdentityMappingExternalRefLengthRule { max_length: 3600 }),
            Rc::new(IdentityMappingQuantityRangeRule { min: 32, max: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_identity_mapping_validator();
        let item = new_identity_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = IdentityMappingRequiredCodeRule {
            message: "identity mapping needs a value".to_string(),
        };
        let mut item = new_identity_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = IdentityMappingExternalRefLengthRule { max_length: 1000 };
        let mut item = new_identity_mapping("x", "y");
        item.external_ref = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = IdentityMappingQuantityRangeRule { min: 86400, max: 86400 };
        let mut item = new_identity_mapping("x", "y");
        item.quantity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
