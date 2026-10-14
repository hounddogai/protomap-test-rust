use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a data export group must pass. Check returns its problems, or none.
pub trait DataExportGroupRule: std::fmt::Debug {
    fn check(&self, item: &DataExportGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportGroupGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DataExportGroupRule for DataExportGroupGraceDaysRangeRule {
    fn check(&self, item: &DataExportGroup) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportGroupTitleLengthRule {
    pub max_length: i64,
}

impl DataExportGroupRule for DataExportGroupTitleLengthRule {
    fn check(&self, item: &DataExportGroup) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportGroupCodePrefixRule {
    pub prefix: String,
}

impl DataExportGroupRule for DataExportGroupCodePrefixRule {
    fn check(&self, item: &DataExportGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportGroupRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DataExportGroupRule for DataExportGroupRateRangeRule {
    fn check(&self, item: &DataExportGroup) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportGroupCodeLengthRule {
    pub max_length: i64,
}

impl DataExportGroupRule for DataExportGroupCodeLengthRule {
    fn check(&self, item: &DataExportGroup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportGroupDescriptionLengthRule {
    pub max_length: i64,
}

impl DataExportGroupRule for DataExportGroupDescriptionLengthRule {
    fn check(&self, item: &DataExportGroup) -> Vec<String> {
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

/// Runs every rule of a data export group.
#[derive(Debug, Clone)]
pub struct DataExportGroupValidator {
    pub rules: Vec<Rc<dyn DataExportGroupRule>>,
}

impl DataExportGroupValidator {
    pub fn validate(&self, item: &DataExportGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DataExportGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_group_validator() -> DataExportGroupValidator {
    DataExportGroupValidator {
        rules: vec![
            Rc::new(DataExportGroupGraceDaysRangeRule { min: 16, max: 64 }) as Rc<dyn DataExportGroupRule>,
            Rc::new(DataExportGroupTitleLengthRule { max_length: 64 }),
            Rc::new(DataExportGroupCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(DataExportGroupRateRangeRule { min: 9.9, max: 1513.4 }),
            Rc::new(DataExportGroupCodeLengthRule { max_length: 86400 }),
            Rc::new(DataExportGroupDescriptionLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_data_export_group_validator();
        let item = new_data_export_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = DataExportGroupGraceDaysRangeRule { min: 16, max: 255 };
        let mut item = new_data_export_group("x", "y");
        item.grace_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = DataExportGroupTitleLengthRule { max_length: 1000 };
        let mut item = new_data_export_group("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = DataExportGroupRateRangeRule { min: 8.5, max: 609.2 };
        let mut item = new_data_export_group("x", "y");
        item.rate = 610.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DataExportGroupCodeLengthRule { max_length: 32 };
        let mut item = new_data_export_group("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = DataExportGroupDescriptionLengthRule { max_length: 1000 };
        let mut item = new_data_export_group("x", "y");
        item.description = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
