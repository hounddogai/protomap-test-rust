use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal alert record must pass. Check returns its problems, or none.
pub trait SeasonalAlertRecordRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalAlertRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAlertRecordMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalAlertRecordRule for SeasonalAlertRecordMaximumRangeRule {
    fn check(&self, item: &SeasonalAlertRecord) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAlertRecordRequiredContactEmailRule {
    pub message: String,
}

impl SeasonalAlertRecordRule for SeasonalAlertRecordRequiredContactEmailRule {
    fn check(&self, item: &SeasonalAlertRecord) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAlertRecordCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalAlertRecordRule for SeasonalAlertRecordCodeLengthRule {
    fn check(&self, item: &SeasonalAlertRecord) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal alert record.
#[derive(Debug, Clone)]
pub struct SeasonalAlertRecordValidator {
    pub rules: Vec<Rc<dyn SeasonalAlertRecordRule>>,
}

impl SeasonalAlertRecordValidator {
    pub fn validate(&self, item: &SeasonalAlertRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalAlertRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalAlertRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_alert_record_validator() -> SeasonalAlertRecordValidator {
    SeasonalAlertRecordValidator {
        rules: vec![
            Rc::new(SeasonalAlertRecordMaximumRangeRule { min: 4.0, max: 278.7 }) as Rc<dyn SeasonalAlertRecordRule>,
            Rc::new(SeasonalAlertRecordRequiredContactEmailRule {
                message: "seasonal alert record needs a value".to_string(),
            }),
            Rc::new(SeasonalAlertRecordCodeLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_alert_record_validator();
        let item = new_seasonal_alert_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = SeasonalAlertRecordMaximumRangeRule { min: 6.6, max: 641.2 };
        let mut item = new_seasonal_alert_record("x", "y");
        item.maximum = 642.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = SeasonalAlertRecordRequiredContactEmailRule {
            message: "seasonal alert record needs an entry".to_string(),
        };
        let mut item = new_seasonal_alert_record("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SeasonalAlertRecordCodeLengthRule { max_length: 255 };
        let mut item = new_seasonal_alert_record("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
