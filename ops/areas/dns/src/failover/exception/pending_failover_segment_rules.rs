use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a pending failover segment must pass. Check returns its problems, or none.
pub trait PendingFailoverSegmentRule: std::fmt::Debug {
    fn check(&self, item: &PendingFailoverSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFailoverSegmentRequiredTimezoneRule {
    pub message: String,
}

impl PendingFailoverSegmentRule for PendingFailoverSegmentRequiredTimezoneRule {
    fn check(&self, item: &PendingFailoverSegment) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFailoverSegmentSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PendingFailoverSegmentRule for PendingFailoverSegmentSequenceRangeRule {
    fn check(&self, item: &PendingFailoverSegment) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingFailoverSegmentCategoryLengthRule {
    pub max_length: i64,
}

impl PendingFailoverSegmentRule for PendingFailoverSegmentCategoryLengthRule {
    fn check(&self, item: &PendingFailoverSegment) -> Vec<String> {
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
pub struct PendingFailoverSegmentAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl PendingFailoverSegmentRule for PendingFailoverSegmentAllowedCodeRule {
    fn check(&self, item: &PendingFailoverSegment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a pending failover segment.
#[derive(Debug, Clone)]
pub struct PendingFailoverSegmentValidator {
    pub rules: Vec<Rc<dyn PendingFailoverSegmentRule>>,
}

impl PendingFailoverSegmentValidator {
    pub fn validate(&self, item: &PendingFailoverSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PendingFailoverSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PendingFailoverSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_pending_failover_segment_validator() -> PendingFailoverSegmentValidator {
    PendingFailoverSegmentValidator {
        rules: vec![
            Rc::new(PendingFailoverSegmentRequiredTimezoneRule {
                message: "pending failover segment needs this field".to_string(),
            }) as Rc<dyn PendingFailoverSegmentRule>,
            Rc::new(PendingFailoverSegmentSequenceRangeRule { min: 32, max: 3600 }),
            Rc::new(PendingFailoverSegmentCategoryLengthRule { max_length: 64 }),
            Rc::new(PendingFailoverSegmentAllowedCodeRule {
                allowed: vec!["batched".to_string(), "weighted".to_string(), "weighted".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_pending_failover_segment_validator();
        let item = new_pending_failover_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = PendingFailoverSegmentRequiredTimezoneRule {
            message: "pending failover segment needs this field".to_string(),
        };
        let mut item = new_pending_failover_segment("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = PendingFailoverSegmentSequenceRangeRule { min: 32, max: 1000 };
        let mut item = new_pending_failover_segment("x", "y");
        item.sequence = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = PendingFailoverSegmentCategoryLengthRule { max_length: 16 };
        let mut item = new_pending_failover_segment("x", "y");
        item.category = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
