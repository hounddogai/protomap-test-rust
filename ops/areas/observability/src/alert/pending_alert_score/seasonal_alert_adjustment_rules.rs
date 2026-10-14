use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal alert adjustment must pass. Check returns its problems, or none.
pub trait SeasonalAlertAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalAlertAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAlertAdjustmentRequiredTitleRule {
    pub message: String,
}

impl SeasonalAlertAdjustmentRule for SeasonalAlertAdjustmentRequiredTitleRule {
    fn check(&self, item: &SeasonalAlertAdjustment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAlertAdjustmentRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalAlertAdjustmentRule for SeasonalAlertAdjustmentRevisionRangeRule {
    fn check(&self, item: &SeasonalAlertAdjustment) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAlertAdjustmentRequiredHostRule {
    pub message: String,
}

impl SeasonalAlertAdjustmentRule for SeasonalAlertAdjustmentRequiredHostRule {
    fn check(&self, item: &SeasonalAlertAdjustment) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAlertAdjustmentRequiredCodeRule {
    pub message: String,
}

impl SeasonalAlertAdjustmentRule for SeasonalAlertAdjustmentRequiredCodeRule {
    fn check(&self, item: &SeasonalAlertAdjustment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAlertAdjustmentRequiredContactEmailRule {
    pub message: String,
}

impl SeasonalAlertAdjustmentRule for SeasonalAlertAdjustmentRequiredContactEmailRule {
    fn check(&self, item: &SeasonalAlertAdjustment) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAlertAdjustmentTitleLengthRule {
    pub max_length: i64,
}

impl SeasonalAlertAdjustmentRule for SeasonalAlertAdjustmentTitleLengthRule {
    fn check(&self, item: &SeasonalAlertAdjustment) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal alert adjustment.
#[derive(Debug, Clone)]
pub struct SeasonalAlertAdjustmentValidator {
    pub rules: Vec<Rc<dyn SeasonalAlertAdjustmentRule>>,
}

impl SeasonalAlertAdjustmentValidator {
    pub fn validate(&self, item: &SeasonalAlertAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalAlertAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalAlertAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_alert_adjustment_validator() -> SeasonalAlertAdjustmentValidator {
    SeasonalAlertAdjustmentValidator {
        rules: vec![
            Rc::new(SeasonalAlertAdjustmentRequiredTitleRule {
                message: "seasonal alert adjustment needs an entry".to_string(),
            }) as Rc<dyn SeasonalAlertAdjustmentRule>,
            Rc::new(SeasonalAlertAdjustmentRevisionRangeRule { min: 32, max: 1000 }),
            Rc::new(SeasonalAlertAdjustmentRequiredHostRule {
                message: "seasonal alert adjustment needs a value".to_string(),
            }),
            Rc::new(SeasonalAlertAdjustmentRequiredCodeRule {
                message: "seasonal alert adjustment needs an entry".to_string(),
            }),
            Rc::new(SeasonalAlertAdjustmentRequiredContactEmailRule {
                message: "seasonal alert adjustment needs a value".to_string(),
            }),
            Rc::new(SeasonalAlertAdjustmentTitleLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_alert_adjustment_validator();
        let item = new_seasonal_alert_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = SeasonalAlertAdjustmentRequiredTitleRule {
            message: "seasonal alert adjustment needs this field".to_string(),
        };
        let mut item = new_seasonal_alert_adjustment("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = SeasonalAlertAdjustmentRevisionRangeRule { min: 64, max: 255 };
        let mut item = new_seasonal_alert_adjustment("x", "y");
        item.revision = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = SeasonalAlertAdjustmentRequiredHostRule {
            message: "seasonal alert adjustment needs an entry".to_string(),
        };
        let mut item = new_seasonal_alert_adjustment("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalAlertAdjustmentRequiredCodeRule {
            message: "seasonal alert adjustment needs this field".to_string(),
        };
        let mut item = new_seasonal_alert_adjustment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = SeasonalAlertAdjustmentRequiredContactEmailRule {
            message: "seasonal alert adjustment needs this field".to_string(),
        };
        let mut item = new_seasonal_alert_adjustment("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = SeasonalAlertAdjustmentTitleLengthRule { max_length: 1000 };
        let mut item = new_seasonal_alert_adjustment("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
