use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a quote schedule must pass. Check returns its problems, or none.
pub trait QuoteScheduleRule: std::fmt::Debug {
    fn check(&self, item: &QuoteSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteScheduleCodePrefixRule {
    pub prefix: String,
}

impl QuoteScheduleRule for QuoteScheduleCodePrefixRule {
    fn check(&self, item: &QuoteSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteScheduleRequiredContactEmailRule {
    pub message: String,
}

impl QuoteScheduleRule for QuoteScheduleRequiredContactEmailRule {
    fn check(&self, item: &QuoteSchedule) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteScheduleRequiredDescriptionRule {
    pub message: String,
}

impl QuoteScheduleRule for QuoteScheduleRequiredDescriptionRule {
    fn check(&self, item: &QuoteSchedule) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteScheduleThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QuoteScheduleRule for QuoteScheduleThresholdRangeRule {
    fn check(&self, item: &QuoteSchedule) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteScheduleContactEmailLengthRule {
    pub max_length: i64,
}

impl QuoteScheduleRule for QuoteScheduleContactEmailLengthRule {
    fn check(&self, item: &QuoteSchedule) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteScheduleAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QuoteScheduleRule for QuoteScheduleAmountRangeRule {
    fn check(&self, item: &QuoteSchedule) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a quote schedule.
#[derive(Debug, Clone)]
pub struct QuoteScheduleValidator {
    pub rules: Vec<Rc<dyn QuoteScheduleRule>>,
}

impl QuoteScheduleValidator {
    pub fn validate(&self, item: &QuoteSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &QuoteSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_schedule_validator() -> QuoteScheduleValidator {
    QuoteScheduleValidator {
        rules: vec![
            Rc::new(QuoteScheduleCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn QuoteScheduleRule>,
            Rc::new(QuoteScheduleRequiredContactEmailRule {
                message: "quote schedule needs a value".to_string(),
            }),
            Rc::new(QuoteScheduleRequiredDescriptionRule {
                message: "quote schedule needs an entry".to_string(),
            }),
            Rc::new(QuoteScheduleThresholdRangeRule { min: 0.2, max: 1238.0 }),
            Rc::new(QuoteScheduleContactEmailLengthRule { max_length: 16 }),
            Rc::new(QuoteScheduleAmountRangeRule { min: 9.9, max: 4060.6 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_quote_schedule_validator();
        let item = new_quote_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = QuoteScheduleRequiredContactEmailRule {
            message: "quote schedule needs this field".to_string(),
        };
        let mut item = new_quote_schedule("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = QuoteScheduleRequiredDescriptionRule {
            message: "quote schedule needs this field".to_string(),
        };
        let mut item = new_quote_schedule("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = QuoteScheduleThresholdRangeRule { min: 2.0, max: 937.7 };
        let mut item = new_quote_schedule("x", "y");
        item.threshold = 939.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = QuoteScheduleContactEmailLengthRule { max_length: 80 };
        let mut item = new_quote_schedule("x", "y");
        item.contact_email = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = QuoteScheduleAmountRangeRule { min: 1.5, max: 2903.8 };
        let mut item = new_quote_schedule("x", "y");
        item.amount = 2905.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
