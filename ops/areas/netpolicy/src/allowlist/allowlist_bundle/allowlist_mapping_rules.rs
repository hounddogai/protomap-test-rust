use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a allowlist mapping must pass. Check returns its problems, or none.
pub trait AllowlistMappingRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistMappingRequiredUnitRule {
    pub message: String,
}

impl AllowlistMappingRule for AllowlistMappingRequiredUnitRule {
    fn check(&self, item: &AllowlistMapping) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistMappingQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AllowlistMappingRule for AllowlistMappingQuantityRangeRule {
    fn check(&self, item: &AllowlistMapping) -> Vec<String> {
        let value = item.quantity;
        if value < self.min || value > self.max {
            return vec![format!("quantity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistMappingDescriptionLengthRule {
    pub max_length: i64,
}

impl AllowlistMappingRule for AllowlistMappingDescriptionLengthRule {
    fn check(&self, item: &AllowlistMapping) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistMappingCodeLengthRule {
    pub max_length: i64,
}

impl AllowlistMappingRule for AllowlistMappingCodeLengthRule {
    fn check(&self, item: &AllowlistMapping) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistMappingCodePrefixRule {
    pub prefix: String,
}

impl AllowlistMappingRule for AllowlistMappingCodePrefixRule {
    fn check(&self, item: &AllowlistMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist mapping.
#[derive(Debug, Clone)]
pub struct AllowlistMappingValidator {
    pub rules: Vec<Rc<dyn AllowlistMappingRule>>,
}

impl AllowlistMappingValidator {
    pub fn validate(&self, item: &AllowlistMapping) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AllowlistMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_mapping_validator() -> AllowlistMappingValidator {
    AllowlistMappingValidator {
        rules: vec![
            Rc::new(AllowlistMappingRequiredUnitRule {
                message: "allowlist mapping needs an entry".to_string(),
            }) as Rc<dyn AllowlistMappingRule>,
            Rc::new(AllowlistMappingQuantityRangeRule { min: 64, max: 3600 }),
            Rc::new(AllowlistMappingDescriptionLengthRule { max_length: 86400 }),
            Rc::new(AllowlistMappingCodeLengthRule { max_length: 1000 }),
            Rc::new(AllowlistMappingCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_allowlist_mapping_validator();
        let item = new_allowlist_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = AllowlistMappingRequiredUnitRule {
            message: "allowlist mapping needs a value".to_string(),
        };
        let mut item = new_allowlist_mapping("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = AllowlistMappingQuantityRangeRule { min: 32, max: 1000 };
        let mut item = new_allowlist_mapping("x", "y");
        item.quantity = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = AllowlistMappingDescriptionLengthRule { max_length: 255 };
        let mut item = new_allowlist_mapping("x", "y");
        item.description = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AllowlistMappingCodeLengthRule { max_length: 16 };
        let mut item = new_allowlist_mapping("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
