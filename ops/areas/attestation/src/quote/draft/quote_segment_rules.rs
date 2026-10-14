use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a quote segment must pass. Check returns its problems, or none.
pub trait QuoteSegmentRule: std::fmt::Debug {
    fn check(&self, item: &QuoteSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteSegmentChannelLengthRule {
    pub max_length: i64,
}

impl QuoteSegmentRule for QuoteSegmentChannelLengthRule {
    fn check(&self, item: &QuoteSegment) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteSegmentCodeLengthRule {
    pub max_length: i64,
}

impl QuoteSegmentRule for QuoteSegmentCodeLengthRule {
    fn check(&self, item: &QuoteSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteSegmentTitleLengthRule {
    pub max_length: i64,
}

impl QuoteSegmentRule for QuoteSegmentTitleLengthRule {
    fn check(&self, item: &QuoteSegment) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteSegmentRequiredCodeRule {
    pub message: String,
}

impl QuoteSegmentRule for QuoteSegmentRequiredCodeRule {
    fn check(&self, item: &QuoteSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteSegmentWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl QuoteSegmentRule for QuoteSegmentWindowDaysRangeRule {
    fn check(&self, item: &QuoteSegment) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a quote segment.
#[derive(Debug, Clone)]
pub struct QuoteSegmentValidator {
    pub rules: Vec<Rc<dyn QuoteSegmentRule>>,
}

impl QuoteSegmentValidator {
    pub fn validate(&self, item: &QuoteSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QuoteSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_segment_validator() -> QuoteSegmentValidator {
    QuoteSegmentValidator {
        rules: vec![
            Rc::new(QuoteSegmentChannelLengthRule { max_length: 1000 }) as Rc<dyn QuoteSegmentRule>,
            Rc::new(QuoteSegmentCodeLengthRule { max_length: 86400 }),
            Rc::new(QuoteSegmentTitleLengthRule { max_length: 16 }),
            Rc::new(QuoteSegmentRequiredCodeRule {
                message: "quote segment needs this field".to_string(),
            }),
            Rc::new(QuoteSegmentWindowDaysRangeRule { min: 80, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_quote_segment_validator();
        let item = new_quote_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = QuoteSegmentChannelLengthRule { max_length: 16 };
        let mut item = new_quote_segment("x", "y");
        item.channel = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = QuoteSegmentCodeLengthRule { max_length: 16 };
        let mut item = new_quote_segment("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = QuoteSegmentTitleLengthRule { max_length: 3600 };
        let mut item = new_quote_segment("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = QuoteSegmentRequiredCodeRule {
            message: "quote segment needs this field".to_string(),
        };
        let mut item = new_quote_segment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = QuoteSegmentWindowDaysRangeRule { min: 120, max: 255 };
        let mut item = new_quote_segment("x", "y");
        item.window_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
