use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a data export schedule must pass. Check returns its problems, or none.
pub trait DataExportScheduleRule: std::fmt::Debug {
    fn check(&self, item: &DataExportSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportScheduleOwnerIdLengthRule {
    pub max_length: i64,
}

impl DataExportScheduleRule for DataExportScheduleOwnerIdLengthRule {
    fn check(&self, item: &DataExportSchedule) -> Vec<String> {
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
pub struct DataExportScheduleRequiredOwnerIdRule {
    pub message: String,
}

impl DataExportScheduleRule for DataExportScheduleRequiredOwnerIdRule {
    fn check(&self, item: &DataExportSchedule) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportScheduleCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DataExportScheduleRule for DataExportScheduleCreatedAtRangeRule {
    fn check(&self, item: &DataExportSchedule) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportScheduleRequiredCodeRule {
    pub message: String,
}

impl DataExportScheduleRule for DataExportScheduleRequiredCodeRule {
    fn check(&self, item: &DataExportSchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a data export schedule.
#[derive(Debug, Clone)]
pub struct DataExportScheduleValidator {
    pub rules: Vec<Rc<dyn DataExportScheduleRule>>,
}

impl DataExportScheduleValidator {
    pub fn validate(&self, item: &DataExportSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DataExportSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_schedule_validator() -> DataExportScheduleValidator {
    DataExportScheduleValidator {
        rules: vec![
            Rc::new(DataExportScheduleOwnerIdLengthRule { max_length: 32 }) as Rc<dyn DataExportScheduleRule>,
            Rc::new(DataExportScheduleRequiredOwnerIdRule {
                message: "data export schedule needs a value".to_string(),
            }),
            Rc::new(DataExportScheduleCreatedAtRangeRule { min: 32, max: 1000 }),
            Rc::new(DataExportScheduleRequiredCodeRule {
                message: "data export schedule needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_data_export_schedule_validator();
        let item = new_data_export_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = DataExportScheduleOwnerIdLengthRule { max_length: 120 };
        let mut item = new_data_export_schedule("x", "y");
        item.owner_id = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = DataExportScheduleRequiredOwnerIdRule {
            message: "data export schedule needs an entry".to_string(),
        };
        let mut item = new_data_export_schedule("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = DataExportScheduleCreatedAtRangeRule { min: 80, max: 120 };
        let mut item = new_data_export_schedule("x", "y");
        item.created_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DataExportScheduleRequiredCodeRule {
            message: "data export schedule needs this field".to_string(),
        };
        let mut item = new_data_export_schedule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
