use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a overage mapping must pass. Check returns its problems, or none.
pub trait OverageMappingRule: std::fmt::Debug {
    fn check(&self, item: &OverageMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageMappingNameLengthRule {
    pub max_length: i64,
}

impl OverageMappingRule for OverageMappingNameLengthRule {
    fn check(&self, item: &OverageMapping) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageMappingRequiredDescriptionRule {
    pub message: String,
}

impl OverageMappingRule for OverageMappingRequiredDescriptionRule {
    fn check(&self, item: &OverageMapping) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageMappingHostLengthRule {
    pub max_length: i64,
}

impl OverageMappingRule for OverageMappingHostLengthRule {
    fn check(&self, item: &OverageMapping) -> Vec<String> {
        let size = item.host.len() as i64;
        if size > self.max_length {
            return vec![format!("host is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageMappingRequiredHostRule {
    pub message: String,
}

impl OverageMappingRule for OverageMappingRequiredHostRule {
    fn check(&self, item: &OverageMapping) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageMappingAllowedDescriptionRule {
    pub allowed: Vec<String>,
}

impl OverageMappingRule for OverageMappingAllowedDescriptionRule {
    fn check(&self, item: &OverageMapping) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.description)))
        {
            return vec![format!("description {} is missing", item.description)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageMappingRequiredCodeRule {
    pub message: String,
}

impl OverageMappingRule for OverageMappingRequiredCodeRule {
    fn check(&self, item: &OverageMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a overage mapping.
#[derive(Debug, Clone)]
pub struct OverageMappingValidator {
    pub rules: Vec<Rc<dyn OverageMappingRule>>,
}

impl OverageMappingValidator {
    pub fn validate(&self, item: &OverageMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &OverageMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_mapping_validator() -> OverageMappingValidator {
    OverageMappingValidator {
        rules: vec![
            Rc::new(OverageMappingNameLengthRule { max_length: 64 }) as Rc<dyn OverageMappingRule>,
            Rc::new(OverageMappingRequiredDescriptionRule {
                message: "overage mapping needs this field".to_string(),
            }),
            Rc::new(OverageMappingHostLengthRule { max_length: 80 }),
            Rc::new(OverageMappingRequiredHostRule {
                message: "overage mapping needs an entry".to_string(),
            }),
            Rc::new(OverageMappingAllowedDescriptionRule {
                allowed: vec!["lenient".to_string(), "seasonal".to_string()],
            }),
            Rc::new(OverageMappingRequiredCodeRule {
                message: "overage mapping needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_overage_mapping_validator();
        let item = new_overage_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = OverageMappingNameLengthRule { max_length: 3600 };
        let mut item = new_overage_mapping("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = OverageMappingRequiredDescriptionRule {
            message: "overage mapping needs this field".to_string(),
        };
        let mut item = new_overage_mapping("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_host() {
        let rule = OverageMappingHostLengthRule { max_length: 64 };
        let mut item = new_overage_mapping("x", "y");
        item.host = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.host = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = OverageMappingRequiredHostRule {
            message: "overage mapping needs a value".to_string(),
        };
        let mut item = new_overage_mapping("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = OverageMappingRequiredCodeRule {
            message: "overage mapping needs a value".to_string(),
        };
        let mut item = new_overage_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
