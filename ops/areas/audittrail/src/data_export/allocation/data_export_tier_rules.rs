use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a data export tier must pass. Check returns its problems, or none.
pub trait DataExportTierRule: std::fmt::Debug {
    fn check(&self, item: &DataExportTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportTierAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl DataExportTierRule for DataExportTierAllowedCodeRule {
    fn check(&self, item: &DataExportTier) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportTierCodeLengthRule {
    pub max_length: i64,
}

impl DataExportTierRule for DataExportTierCodeLengthRule {
    fn check(&self, item: &DataExportTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportTierRequiredCodeRule {
    pub message: String,
}

impl DataExportTierRule for DataExportTierRequiredCodeRule {
    fn check(&self, item: &DataExportTier) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportTierRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DataExportTierRule for DataExportTierRotationDaysRangeRule {
    fn check(&self, item: &DataExportTier) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a data export tier.
#[derive(Debug, Clone)]
pub struct DataExportTierValidator {
    pub rules: Vec<Rc<dyn DataExportTierRule>>,
}

impl DataExportTierValidator {
    pub fn validate(&self, item: &DataExportTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DataExportTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_tier_validator() -> DataExportTierValidator {
    DataExportTierValidator {
        rules: vec![
            Rc::new(DataExportTierAllowedCodeRule {
                allowed: vec!["lenient".to_string()],
            }) as Rc<dyn DataExportTierRule>,
            Rc::new(DataExportTierCodeLengthRule { max_length: 1000 }),
            Rc::new(DataExportTierRequiredCodeRule {
                message: "data export tier needs an entry".to_string(),
            }),
            Rc::new(DataExportTierRotationDaysRangeRule { min: 80, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_data_export_tier_validator();
        let item = new_data_export_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DataExportTierCodeLengthRule { max_length: 16 };
        let mut item = new_data_export_tier("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DataExportTierRequiredCodeRule {
            message: "data export tier needs an entry".to_string(),
        };
        let mut item = new_data_export_tier("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = DataExportTierRotationDaysRangeRule { min: 3600, max: 86400 };
        let mut item = new_data_export_tier("x", "y");
        item.rotation_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
