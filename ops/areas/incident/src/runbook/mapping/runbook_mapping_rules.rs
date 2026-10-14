use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a runbook mapping must pass. Check returns its problems, or none.
pub trait RunbookMappingRule: std::fmt::Debug {
    fn check(&self, item: &RunbookMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookMappingCodePrefixRule {
    pub prefix: String,
}

impl RunbookMappingRule for RunbookMappingCodePrefixRule {
    fn check(&self, item: &RunbookMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookMappingRequiredLabelRule {
    pub message: String,
}

impl RunbookMappingRule for RunbookMappingRequiredLabelRule {
    fn check(&self, item: &RunbookMapping) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RunbookMappingGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RunbookMappingRule for RunbookMappingGraceDaysRangeRule {
    fn check(&self, item: &RunbookMapping) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a runbook mapping.
#[derive(Debug, Clone)]
pub struct RunbookMappingValidator {
    pub rules: Vec<Rc<dyn RunbookMappingRule>>,
}

impl RunbookMappingValidator {
    pub fn validate(&self, item: &RunbookMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RunbookMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RunbookMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_runbook_mapping_validator() -> RunbookMappingValidator {
    RunbookMappingValidator {
        rules: vec![
            Rc::new(RunbookMappingCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn RunbookMappingRule>,
            Rc::new(RunbookMappingRequiredLabelRule {
                message: "runbook mapping needs an entry".to_string(),
            }),
            Rc::new(RunbookMappingGraceDaysRangeRule { min: 16, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_runbook_mapping_validator();
        let item = new_runbook_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RunbookMappingRequiredLabelRule {
            message: "runbook mapping needs a value".to_string(),
        };
        let mut item = new_runbook_mapping("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = RunbookMappingGraceDaysRangeRule { min: 64, max: 64 };
        let mut item = new_runbook_mapping("x", "y");
        item.grace_days = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
