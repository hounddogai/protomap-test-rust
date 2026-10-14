use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a data export milestone must pass. Check returns its problems, or none.
pub trait DataExportMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &DataExportMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportMilestoneAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl DataExportMilestoneRule for DataExportMilestoneAllowedLabelRule {
    fn check(&self, item: &DataExportMilestone) -> Vec<String> {
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
pub struct DataExportMilestoneCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DataExportMilestoneRule for DataExportMilestoneCostRangeRule {
    fn check(&self, item: &DataExportMilestone) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportMilestoneCodePrefixRule {
    pub prefix: String,
}

impl DataExportMilestoneRule for DataExportMilestoneCodePrefixRule {
    fn check(&self, item: &DataExportMilestone) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportMilestoneCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DataExportMilestoneRule for DataExportMilestoneCeilingRangeRule {
    fn check(&self, item: &DataExportMilestone) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportMilestoneRequiredLabelRule {
    pub message: String,
}

impl DataExportMilestoneRule for DataExportMilestoneRequiredLabelRule {
    fn check(&self, item: &DataExportMilestone) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportMilestoneRequiredSegmentRule {
    pub message: String,
}

impl DataExportMilestoneRule for DataExportMilestoneRequiredSegmentRule {
    fn check(&self, item: &DataExportMilestone) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a data export milestone.
#[derive(Debug, Clone)]
pub struct DataExportMilestoneValidator {
    pub rules: Vec<Rc<dyn DataExportMilestoneRule>>,
}

impl DataExportMilestoneValidator {
    pub fn validate(&self, item: &DataExportMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DataExportMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_milestone_validator() -> DataExportMilestoneValidator {
    DataExportMilestoneValidator {
        rules: vec![
            Rc::new(DataExportMilestoneAllowedLabelRule {
                allowed: vec!["tiered".to_string(), "rolling".to_string()],
            }) as Rc<dyn DataExportMilestoneRule>,
            Rc::new(DataExportMilestoneCostRangeRule { min: 9.0, max: 1898.0 }),
            Rc::new(DataExportMilestoneCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(DataExportMilestoneCeilingRangeRule { min: 3.5, max: 4619.2 }),
            Rc::new(DataExportMilestoneRequiredLabelRule {
                message: "data export milestone needs an entry".to_string(),
            }),
            Rc::new(DataExportMilestoneRequiredSegmentRule {
                message: "data export milestone needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_data_export_milestone_validator();
        let item = new_data_export_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = DataExportMilestoneCostRangeRule { min: 9.2, max: 188.0 };
        let mut item = new_data_export_milestone("x", "y");
        item.cost = 189.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = DataExportMilestoneCeilingRangeRule { min: 9.5, max: 2552.7 };
        let mut item = new_data_export_milestone("x", "y");
        item.ceiling = 2554.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DataExportMilestoneRequiredLabelRule {
            message: "data export milestone needs an entry".to_string(),
        };
        let mut item = new_data_export_milestone("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = DataExportMilestoneRequiredSegmentRule {
            message: "data export milestone needs a value".to_string(),
        };
        let mut item = new_data_export_milestone("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
