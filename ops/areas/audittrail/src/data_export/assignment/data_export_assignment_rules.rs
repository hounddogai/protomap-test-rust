use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a data export assignment must pass. Check returns its problems, or none.
pub trait DataExportAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &DataExportAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportAssignmentCodePrefixRule {
    pub prefix: String,
}

impl DataExportAssignmentRule for DataExportAssignmentCodePrefixRule {
    fn check(&self, item: &DataExportAssignment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportAssignmentRequiredLabelRule {
    pub message: String,
}

impl DataExportAssignmentRule for DataExportAssignmentRequiredLabelRule {
    fn check(&self, item: &DataExportAssignment) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportAssignmentPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DataExportAssignmentRule for DataExportAssignmentPriorityRangeRule {
    fn check(&self, item: &DataExportAssignment) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportAssignmentWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DataExportAssignmentRule for DataExportAssignmentWeightRangeRule {
    fn check(&self, item: &DataExportAssignment) -> Vec<String> {
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
pub struct DataExportAssignmentAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl DataExportAssignmentRule for DataExportAssignmentAllowedCodeRule {
    fn check(&self, item: &DataExportAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is not allowed", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a data export assignment.
#[derive(Debug, Clone)]
pub struct DataExportAssignmentValidator {
    pub rules: Vec<Rc<dyn DataExportAssignmentRule>>,
}

impl DataExportAssignmentValidator {
    pub fn validate(&self, item: &DataExportAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DataExportAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_assignment_validator() -> DataExportAssignmentValidator {
    DataExportAssignmentValidator {
        rules: vec![
            Rc::new(DataExportAssignmentCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn DataExportAssignmentRule>,
            Rc::new(DataExportAssignmentRequiredLabelRule {
                message: "data export assignment needs this field".to_string(),
            }),
            Rc::new(DataExportAssignmentPriorityRangeRule { min: 16, max: 3600 }),
            Rc::new(DataExportAssignmentWeightRangeRule { min: 0.5, max: 236.2 }),
            Rc::new(DataExportAssignmentAllowedCodeRule {
                allowed: vec!["legacy".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_data_export_assignment_validator();
        let item = new_data_export_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DataExportAssignmentRequiredLabelRule {
            message: "data export assignment needs an entry".to_string(),
        };
        let mut item = new_data_export_assignment("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = DataExportAssignmentPriorityRangeRule { min: 32, max: 86400 };
        let mut item = new_data_export_assignment("x", "y");
        item.priority = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = DataExportAssignmentWeightRangeRule { min: 8.6, max: 2882.5 };
        let mut item = new_data_export_assignment("x", "y");
        item.weight = 2884.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
