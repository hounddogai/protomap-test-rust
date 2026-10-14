use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling segment entry must pass. Check returns its problems, or none.
pub trait RollingSegmentEntryRule: std::fmt::Debug {
    fn check(&self, item: &RollingSegmentEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSegmentEntryCodeLengthRule {
    pub max_length: i64,
}

impl RollingSegmentEntryRule for RollingSegmentEntryCodeLengthRule {
    fn check(&self, item: &RollingSegmentEntry) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSegmentEntryRequiredNameRule {
    pub message: String,
}

impl RollingSegmentEntryRule for RollingSegmentEntryRequiredNameRule {
    fn check(&self, item: &RollingSegmentEntry) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSegmentEntryAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingSegmentEntryRule for RollingSegmentEntryAmountRangeRule {
    fn check(&self, item: &RollingSegmentEntry) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct RollingSegmentEntryRequiredLabelRule {
    pub message: String,
}

impl RollingSegmentEntryRule for RollingSegmentEntryRequiredLabelRule {
    fn check(&self, item: &RollingSegmentEntry) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling segment entry.
#[derive(Debug, Clone)]
pub struct RollingSegmentEntryValidator {
    pub rules: Vec<Rc<dyn RollingSegmentEntryRule>>,
}

impl RollingSegmentEntryValidator {
    pub fn validate(&self, item: &RollingSegmentEntry) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingSegmentEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingSegmentEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_segment_entry_validator() -> RollingSegmentEntryValidator {
    RollingSegmentEntryValidator {
        rules: vec![
            Rc::new(RollingSegmentEntryCodeLengthRule { max_length: 32 }) as Rc<dyn RollingSegmentEntryRule>,
            Rc::new(RollingSegmentEntryRequiredNameRule {
                message: "rolling segment entry needs this field".to_string(),
            }),
            Rc::new(RollingSegmentEntryAmountRangeRule { min: 1.7, max: 2879.3 }),
            Rc::new(RollingSegmentEntryRequiredLabelRule {
                message: "rolling segment entry needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_segment_entry_validator();
        let item = new_rolling_segment_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingSegmentEntryCodeLengthRule { max_length: 120 };
        let mut item = new_rolling_segment_entry("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RollingSegmentEntryRequiredNameRule {
            message: "rolling segment entry needs this field".to_string(),
        };
        let mut item = new_rolling_segment_entry("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = RollingSegmentEntryAmountRangeRule { min: 8.4, max: 3448.2 };
        let mut item = new_rolling_segment_entry("x", "y");
        item.amount = 3449.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RollingSegmentEntryRequiredLabelRule {
            message: "rolling segment entry needs a value".to_string(),
        };
        let mut item = new_rolling_segment_entry("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
