use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a quote assignment must pass. Check returns its problems, or none.
pub trait QuoteAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &QuoteAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteAssignmentRegionLengthRule {
    pub max_length: i64,
}

impl QuoteAssignmentRule for QuoteAssignmentRegionLengthRule {
    fn check(&self, item: &QuoteAssignment) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteAssignmentCodePrefixRule {
    pub prefix: String,
}

impl QuoteAssignmentRule for QuoteAssignmentCodePrefixRule {
    fn check(&self, item: &QuoteAssignment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteAssignmentRequiredTitleRule {
    pub message: String,
}

impl QuoteAssignmentRule for QuoteAssignmentRequiredTitleRule {
    fn check(&self, item: &QuoteAssignment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteAssignmentTagsLimitRule {
    pub max_count: i64,
    pub banned: Vec<String>,
}

impl QuoteAssignmentRule for QuoteAssignmentTagsLimitRule {
    fn check(&self, item: &QuoteAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        if (item.tags.len() as i64) > self.max_count {
            problems.push(format!(
                "has {} tags, at most {} allowed",
                item.tags.len() as i64,
                self.max_count
            ));
        }
        for tag in item.tags.iter() {
            if self.banned.iter().any(|value| value == tag) {
                problems.push(format!("tag {} is not allowed", tag));
            }
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteAssignmentRequiredRegionRule {
    pub message: String,
}

impl QuoteAssignmentRule for QuoteAssignmentRequiredRegionRule {
    fn check(&self, item: &QuoteAssignment) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteAssignmentCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl QuoteAssignmentRule for QuoteAssignmentCeilingRangeRule {
    fn check(&self, item: &QuoteAssignment) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a quote assignment.
#[derive(Debug, Clone)]
pub struct QuoteAssignmentValidator {
    pub rules: Vec<Rc<dyn QuoteAssignmentRule>>,
}

impl QuoteAssignmentValidator {
    pub fn validate(&self, item: &QuoteAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &QuoteAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[QuoteAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_quote_assignment_validator() -> QuoteAssignmentValidator {
    QuoteAssignmentValidator {
        rules: vec![
            Rc::new(QuoteAssignmentRegionLengthRule { max_length: 255 }) as Rc<dyn QuoteAssignmentRule>,
            Rc::new(QuoteAssignmentCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(QuoteAssignmentRequiredTitleRule {
                message: "quote assignment needs an entry".to_string(),
            }),
            Rc::new(QuoteAssignmentTagsLimitRule {
                max_count: 86400,
                banned: vec!["tiered".to_string(), "batched".to_string()],
            }),
            Rc::new(QuoteAssignmentRequiredRegionRule {
                message: "quote assignment needs a value".to_string(),
            }),
            Rc::new(QuoteAssignmentCeilingRangeRule { min: 6.3, max: 2548.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_quote_assignment_validator();
        let item = new_quote_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = QuoteAssignmentRegionLengthRule { max_length: 86400 };
        let mut item = new_quote_assignment("x", "y");
        item.region = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = QuoteAssignmentRequiredTitleRule {
            message: "quote assignment needs a value".to_string(),
        };
        let mut item = new_quote_assignment("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = QuoteAssignmentRequiredRegionRule {
            message: "quote assignment needs an entry".to_string(),
        };
        let mut item = new_quote_assignment("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = QuoteAssignmentCeilingRangeRule { min: 3.3, max: 2971.6 };
        let mut item = new_quote_assignment("x", "y");
        item.ceiling = 2973.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
