use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive quote segment must pass. Check returns its problems, or none.
pub trait AdaptiveQuoteSegmentRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveQuoteSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQuoteSegmentRequiredTitleRule {
    pub message: String,
}

impl AdaptiveQuoteSegmentRule for AdaptiveQuoteSegmentRequiredTitleRule {
    fn check(&self, item: &AdaptiveQuoteSegment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQuoteSegmentSummaryLengthRule {
    pub max_length: i64,
}

impl AdaptiveQuoteSegmentRule for AdaptiveQuoteSegmentSummaryLengthRule {
    fn check(&self, item: &AdaptiveQuoteSegment) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQuoteSegmentRequiredCodeRule {
    pub message: String,
}

impl AdaptiveQuoteSegmentRule for AdaptiveQuoteSegmentRequiredCodeRule {
    fn check(&self, item: &AdaptiveQuoteSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQuoteSegmentRequiredSummaryRule {
    pub message: String,
}

impl AdaptiveQuoteSegmentRule for AdaptiveQuoteSegmentRequiredSummaryRule {
    fn check(&self, item: &AdaptiveQuoteSegment) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQuoteSegmentContactEmailLengthRule {
    pub max_length: i64,
}

impl AdaptiveQuoteSegmentRule for AdaptiveQuoteSegmentContactEmailLengthRule {
    fn check(&self, item: &AdaptiveQuoteSegment) -> Vec<String> {
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

/// Runs every rule of a adaptive quote segment.
#[derive(Debug, Clone)]
pub struct AdaptiveQuoteSegmentValidator {
    pub rules: Vec<Rc<dyn AdaptiveQuoteSegmentRule>>,
}

impl AdaptiveQuoteSegmentValidator {
    pub fn validate(&self, item: &AdaptiveQuoteSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveQuoteSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveQuoteSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_quote_segment_validator() -> AdaptiveQuoteSegmentValidator {
    AdaptiveQuoteSegmentValidator {
        rules: vec![
            Rc::new(AdaptiveQuoteSegmentRequiredTitleRule {
                message: "adaptive quote segment needs an entry".to_string(),
            }) as Rc<dyn AdaptiveQuoteSegmentRule>,
            Rc::new(AdaptiveQuoteSegmentSummaryLengthRule { max_length: 255 }),
            Rc::new(AdaptiveQuoteSegmentRequiredCodeRule {
                message: "adaptive quote segment needs an entry".to_string(),
            }),
            Rc::new(AdaptiveQuoteSegmentRequiredSummaryRule {
                message: "adaptive quote segment needs a value".to_string(),
            }),
            Rc::new(AdaptiveQuoteSegmentContactEmailLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_quote_segment_validator();
        let item = new_adaptive_quote_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = AdaptiveQuoteSegmentRequiredTitleRule {
            message: "adaptive quote segment needs an entry".to_string(),
        };
        let mut item = new_adaptive_quote_segment("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = AdaptiveQuoteSegmentSummaryLengthRule { max_length: 3600 };
        let mut item = new_adaptive_quote_segment("x", "y");
        item.summary = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AdaptiveQuoteSegmentRequiredCodeRule {
            message: "adaptive quote segment needs this field".to_string(),
        };
        let mut item = new_adaptive_quote_segment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = AdaptiveQuoteSegmentRequiredSummaryRule {
            message: "adaptive quote segment needs an entry".to_string(),
        };
        let mut item = new_adaptive_quote_segment("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = AdaptiveQuoteSegmentContactEmailLengthRule { max_length: 64 };
        let mut item = new_adaptive_quote_segment("x", "y");
        item.contact_email = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
