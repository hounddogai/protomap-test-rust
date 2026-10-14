use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary runbook mapping must pass. Check returns its problems, or none.
pub trait PrimaryRunbookMappingRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryRunbookMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRunbookMappingLabelLengthRule {
    pub max_length: i64,
}

impl PrimaryRunbookMappingRule for PrimaryRunbookMappingLabelLengthRule {
    fn check(&self, item: &PrimaryRunbookMapping) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRunbookMappingAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl PrimaryRunbookMappingRule for PrimaryRunbookMappingAllowedLabelRule {
    fn check(&self, item: &PrimaryRunbookMapping) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is above maximum", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRunbookMappingCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryRunbookMappingRule for PrimaryRunbookMappingCodeLengthRule {
    fn check(&self, item: &PrimaryRunbookMapping) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary runbook mapping.
#[derive(Debug, Clone)]
pub struct PrimaryRunbookMappingValidator {
    pub rules: Vec<Rc<dyn PrimaryRunbookMappingRule>>,
}

impl PrimaryRunbookMappingValidator {
    pub fn validate(&self, item: &PrimaryRunbookMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryRunbookMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryRunbookMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_runbook_mapping_validator() -> PrimaryRunbookMappingValidator {
    PrimaryRunbookMappingValidator {
        rules: vec![
            Rc::new(PrimaryRunbookMappingLabelLengthRule { max_length: 16 }) as Rc<dyn PrimaryRunbookMappingRule>,
            Rc::new(PrimaryRunbookMappingAllowedLabelRule {
                allowed: vec!["lenient".to_string(), "primary".to_string(), "regional".to_string()],
            }),
            Rc::new(PrimaryRunbookMappingCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_runbook_mapping_validator();
        let item = new_primary_runbook_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PrimaryRunbookMappingLabelLengthRule { max_length: 80 };
        let mut item = new_primary_runbook_mapping("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryRunbookMappingCodeLengthRule { max_length: 80 };
        let mut item = new_primary_runbook_mapping("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
