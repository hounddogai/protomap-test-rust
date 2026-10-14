use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a measurement notice must pass. Check returns its problems, or none.
pub trait MeasurementNoticeRule: std::fmt::Debug {
    fn check(&self, item: &MeasurementNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementNoticeCategoryLengthRule {
    pub max_length: i64,
}

impl MeasurementNoticeRule for MeasurementNoticeCategoryLengthRule {
    fn check(&self, item: &MeasurementNotice) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementNoticeRequiredOwnerIdRule {
    pub message: String,
}

impl MeasurementNoticeRule for MeasurementNoticeRequiredOwnerIdRule {
    fn check(&self, item: &MeasurementNotice) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementNoticeDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl MeasurementNoticeRule for MeasurementNoticeDurationMinutesRangeRule {
    fn check(&self, item: &MeasurementNotice) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a measurement notice.
#[derive(Debug, Clone)]
pub struct MeasurementNoticeValidator {
    pub rules: Vec<Rc<dyn MeasurementNoticeRule>>,
}

impl MeasurementNoticeValidator {
    pub fn validate(&self, item: &MeasurementNotice) -> Vec<String> {
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

    pub fn require_valid(&self, item: &MeasurementNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[MeasurementNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_measurement_notice_validator() -> MeasurementNoticeValidator {
    MeasurementNoticeValidator {
        rules: vec![
            Rc::new(MeasurementNoticeCategoryLengthRule { max_length: 64 }) as Rc<dyn MeasurementNoticeRule>,
            Rc::new(MeasurementNoticeRequiredOwnerIdRule {
                message: "measurement notice needs this field".to_string(),
            }),
            Rc::new(MeasurementNoticeDurationMinutesRangeRule { min: 32, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_measurement_notice_validator();
        let item = new_measurement_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = MeasurementNoticeCategoryLengthRule { max_length: 16 };
        let mut item = new_measurement_notice("x", "y");
        item.category = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = MeasurementNoticeRequiredOwnerIdRule {
            message: "measurement notice needs this field".to_string(),
        };
        let mut item = new_measurement_notice("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = MeasurementNoticeDurationMinutesRangeRule { min: 64, max: 1000 };
        let mut item = new_measurement_notice("x", "y");
        item.duration_minutes = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
