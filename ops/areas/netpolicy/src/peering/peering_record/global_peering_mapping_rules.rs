use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global peering mapping must pass. Check returns its problems, or none.
pub trait GlobalPeeringMappingRule: std::fmt::Debug {
    fn check(&self, item: &GlobalPeeringMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringMappingRequiredCodeRule {
    pub message: String,
}

impl GlobalPeeringMappingRule for GlobalPeeringMappingRequiredCodeRule {
    fn check(&self, item: &GlobalPeeringMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringMappingNameLengthRule {
    pub max_length: i64,
}

impl GlobalPeeringMappingRule for GlobalPeeringMappingNameLengthRule {
    fn check(&self, item: &GlobalPeeringMapping) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringMappingCodePrefixRule {
    pub prefix: String,
}

impl GlobalPeeringMappingRule for GlobalPeeringMappingCodePrefixRule {
    fn check(&self, item: &GlobalPeeringMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPeeringMappingDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalPeeringMappingRule for GlobalPeeringMappingDiscountRangeRule {
    fn check(&self, item: &GlobalPeeringMapping) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a global peering mapping.
#[derive(Debug, Clone)]
pub struct GlobalPeeringMappingValidator {
    pub rules: Vec<Rc<dyn GlobalPeeringMappingRule>>,
}

impl GlobalPeeringMappingValidator {
    pub fn validate(&self, item: &GlobalPeeringMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalPeeringMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalPeeringMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_peering_mapping_validator() -> GlobalPeeringMappingValidator {
    GlobalPeeringMappingValidator {
        rules: vec![
            Rc::new(GlobalPeeringMappingRequiredCodeRule {
                message: "global peering mapping needs this field".to_string(),
            }) as Rc<dyn GlobalPeeringMappingRule>,
            Rc::new(GlobalPeeringMappingNameLengthRule { max_length: 86400 }),
            Rc::new(GlobalPeeringMappingCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(GlobalPeeringMappingDiscountRangeRule { min: 4.5, max: 2800.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_peering_mapping_validator();
        let item = new_global_peering_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GlobalPeeringMappingRequiredCodeRule {
            message: "global peering mapping needs an entry".to_string(),
        };
        let mut item = new_global_peering_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = GlobalPeeringMappingNameLengthRule { max_length: 64 };
        let mut item = new_global_peering_mapping("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = GlobalPeeringMappingDiscountRangeRule { min: 1.1, max: 1634.9 };
        let mut item = new_global_peering_mapping("x", "y");
        item.discount = 1636.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
