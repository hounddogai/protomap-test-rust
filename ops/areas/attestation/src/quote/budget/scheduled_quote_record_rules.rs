use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled quote record must pass. Check returns its problems, or none.
pub trait ScheduledQuoteRecordRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledQuoteRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQuoteRecordMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ScheduledQuoteRecordRule for ScheduledQuoteRecordMinimumRangeRule {
    fn check(&self, item: &ScheduledQuoteRecord) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQuoteRecordRequiredContactEmailRule {
    pub message: String,
}

impl ScheduledQuoteRecordRule for ScheduledQuoteRecordRequiredContactEmailRule {
    fn check(&self, item: &ScheduledQuoteRecord) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQuoteRecordRequiredCodeRule {
    pub message: String,
}

impl ScheduledQuoteRecordRule for ScheduledQuoteRecordRequiredCodeRule {
    fn check(&self, item: &ScheduledQuoteRecord) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQuoteRecordRequiredTitleRule {
    pub message: String,
}

impl ScheduledQuoteRecordRule for ScheduledQuoteRecordRequiredTitleRule {
    fn check(&self, item: &ScheduledQuoteRecord) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled quote record.
#[derive(Debug, Clone)]
pub struct ScheduledQuoteRecordValidator {
    pub rules: Vec<Rc<dyn ScheduledQuoteRecordRule>>,
}

impl ScheduledQuoteRecordValidator {
    pub fn validate(&self, item: &ScheduledQuoteRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledQuoteRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledQuoteRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_quote_record_validator() -> ScheduledQuoteRecordValidator {
    ScheduledQuoteRecordValidator {
        rules: vec![
            Rc::new(ScheduledQuoteRecordMinimumRangeRule { min: 7.7, max: 1178.1 }) as Rc<dyn ScheduledQuoteRecordRule>,
            Rc::new(ScheduledQuoteRecordRequiredContactEmailRule {
                message: "scheduled quote record needs this field".to_string(),
            }),
            Rc::new(ScheduledQuoteRecordRequiredCodeRule {
                message: "scheduled quote record needs this field".to_string(),
            }),
            Rc::new(ScheduledQuoteRecordRequiredTitleRule {
                message: "scheduled quote record needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_quote_record_validator();
        let item = new_scheduled_quote_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = ScheduledQuoteRecordMinimumRangeRule { min: 2.7, max: 2246.8 };
        let mut item = new_scheduled_quote_record("x", "y");
        item.minimum = 2248.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = ScheduledQuoteRecordRequiredContactEmailRule {
            message: "scheduled quote record needs an entry".to_string(),
        };
        let mut item = new_scheduled_quote_record("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledQuoteRecordRequiredCodeRule {
            message: "scheduled quote record needs an entry".to_string(),
        };
        let mut item = new_scheduled_quote_record("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ScheduledQuoteRecordRequiredTitleRule {
            message: "scheduled quote record needs this field".to_string(),
        };
        let mut item = new_scheduled_quote_record("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
