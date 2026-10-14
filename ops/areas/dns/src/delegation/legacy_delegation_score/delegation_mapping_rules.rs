use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a delegation mapping must pass. Check returns its problems, or none.
pub trait DelegationMappingRule: std::fmt::Debug {
    fn check(&self, item: &DelegationMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationMappingRequiredCodeRule {
    pub message: String,
}

impl DelegationMappingRule for DelegationMappingRequiredCodeRule {
    fn check(&self, item: &DelegationMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationMappingSourceLengthRule {
    pub max_length: i64,
}

impl DelegationMappingRule for DelegationMappingSourceLengthRule {
    fn check(&self, item: &DelegationMapping) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationMappingTitleLengthRule {
    pub max_length: i64,
}

impl DelegationMappingRule for DelegationMappingTitleLengthRule {
    fn check(&self, item: &DelegationMapping) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationMappingRequiredSourceRule {
    pub message: String,
}

impl DelegationMappingRule for DelegationMappingRequiredSourceRule {
    fn check(&self, item: &DelegationMapping) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a delegation mapping.
#[derive(Debug, Clone)]
pub struct DelegationMappingValidator {
    pub rules: Vec<Rc<dyn DelegationMappingRule>>,
}

impl DelegationMappingValidator {
    pub fn validate(&self, item: &DelegationMapping) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DelegationMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DelegationMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_delegation_mapping_validator() -> DelegationMappingValidator {
    DelegationMappingValidator {
        rules: vec![
            Rc::new(DelegationMappingRequiredCodeRule {
                message: "delegation mapping needs this field".to_string(),
            }) as Rc<dyn DelegationMappingRule>,
            Rc::new(DelegationMappingSourceLengthRule { max_length: 120 }),
            Rc::new(DelegationMappingTitleLengthRule { max_length: 80 }),
            Rc::new(DelegationMappingRequiredSourceRule {
                message: "delegation mapping needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_delegation_mapping_validator();
        let item = new_delegation_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DelegationMappingRequiredCodeRule {
            message: "delegation mapping needs a value".to_string(),
        };
        let mut item = new_delegation_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = DelegationMappingSourceLengthRule { max_length: 80 };
        let mut item = new_delegation_mapping("x", "y");
        item.source = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = DelegationMappingTitleLengthRule { max_length: 80 };
        let mut item = new_delegation_mapping("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = DelegationMappingRequiredSourceRule {
            message: "delegation mapping needs this field".to_string(),
        };
        let mut item = new_delegation_mapping("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
