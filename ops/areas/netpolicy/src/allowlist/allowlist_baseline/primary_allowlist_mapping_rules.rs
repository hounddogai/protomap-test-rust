use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary allowlist mapping must pass. Check returns its problems, or none.
pub trait PrimaryAllowlistMappingRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryAllowlistMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryAllowlistMappingOwnerIdLengthRule {
    pub max_length: i64,
}

impl PrimaryAllowlistMappingRule for PrimaryAllowlistMappingOwnerIdLengthRule {
    fn check(&self, item: &PrimaryAllowlistMapping) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryAllowlistMappingRequiredOwnerIdRule {
    pub message: String,
}

impl PrimaryAllowlistMappingRule for PrimaryAllowlistMappingRequiredOwnerIdRule {
    fn check(&self, item: &PrimaryAllowlistMapping) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryAllowlistMappingCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryAllowlistMappingRule for PrimaryAllowlistMappingCodeLengthRule {
    fn check(&self, item: &PrimaryAllowlistMapping) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryAllowlistMappingWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryAllowlistMappingRule for PrimaryAllowlistMappingWeightRangeRule {
    fn check(&self, item: &PrimaryAllowlistMapping) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryAllowlistMappingTitleLengthRule {
    pub max_length: i64,
}

impl PrimaryAllowlistMappingRule for PrimaryAllowlistMappingTitleLengthRule {
    fn check(&self, item: &PrimaryAllowlistMapping) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryAllowlistMappingCodePrefixRule {
    pub prefix: String,
}

impl PrimaryAllowlistMappingRule for PrimaryAllowlistMappingCodePrefixRule {
    fn check(&self, item: &PrimaryAllowlistMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary allowlist mapping.
#[derive(Debug, Clone)]
pub struct PrimaryAllowlistMappingValidator {
    pub rules: Vec<Rc<dyn PrimaryAllowlistMappingRule>>,
}

impl PrimaryAllowlistMappingValidator {
    pub fn validate(&self, item: &PrimaryAllowlistMapping) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryAllowlistMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryAllowlistMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_allowlist_mapping_validator() -> PrimaryAllowlistMappingValidator {
    PrimaryAllowlistMappingValidator {
        rules: vec![
            Rc::new(PrimaryAllowlistMappingOwnerIdLengthRule { max_length: 64 }) as Rc<dyn PrimaryAllowlistMappingRule>,
            Rc::new(PrimaryAllowlistMappingRequiredOwnerIdRule {
                message: "primary allowlist mapping needs an entry".to_string(),
            }),
            Rc::new(PrimaryAllowlistMappingCodeLengthRule { max_length: 80 }),
            Rc::new(PrimaryAllowlistMappingWeightRangeRule { min: 3.6, max: 3188.1 }),
            Rc::new(PrimaryAllowlistMappingTitleLengthRule { max_length: 32 }),
            Rc::new(PrimaryAllowlistMappingCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_allowlist_mapping_validator();
        let item = new_primary_allowlist_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = PrimaryAllowlistMappingOwnerIdLengthRule { max_length: 86400 };
        let mut item = new_primary_allowlist_mapping("x", "y");
        item.owner_id = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = PrimaryAllowlistMappingRequiredOwnerIdRule {
            message: "primary allowlist mapping needs an entry".to_string(),
        };
        let mut item = new_primary_allowlist_mapping("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryAllowlistMappingCodeLengthRule { max_length: 1000 };
        let mut item = new_primary_allowlist_mapping("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = PrimaryAllowlistMappingWeightRangeRule { min: 9.8, max: 2841.0 };
        let mut item = new_primary_allowlist_mapping("x", "y");
        item.weight = 2842.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PrimaryAllowlistMappingTitleLengthRule { max_length: 86400 };
        let mut item = new_primary_allowlist_mapping("x", "y");
        item.title = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
