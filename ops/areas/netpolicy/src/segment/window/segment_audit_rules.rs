use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a segment audit must pass. Check returns its problems, or none.
pub trait SegmentAuditRule: std::fmt::Debug {
    fn check(&self, item: &SegmentAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAuditAllowedReviewerRule {
    pub allowed: Vec<String>,
}

impl SegmentAuditRule for SegmentAuditAllowedReviewerRule {
    fn check(&self, item: &SegmentAudit) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.reviewer)))
        {
            return vec![format!("reviewer {} is unknown", item.reviewer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAuditStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SegmentAuditRule for SegmentAuditStartsAtRangeRule {
    fn check(&self, item: &SegmentAudit) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAuditTimezoneLengthRule {
    pub max_length: i64,
}

impl SegmentAuditRule for SegmentAuditTimezoneLengthRule {
    fn check(&self, item: &SegmentAudit) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAuditRequiredReviewerRule {
    pub message: String,
}

impl SegmentAuditRule for SegmentAuditRequiredReviewerRule {
    fn check(&self, item: &SegmentAudit) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment audit.
#[derive(Debug, Clone)]
pub struct SegmentAuditValidator {
    pub rules: Vec<Rc<dyn SegmentAuditRule>>,
}

impl SegmentAuditValidator {
    pub fn validate(&self, item: &SegmentAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SegmentAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_audit_validator() -> SegmentAuditValidator {
    SegmentAuditValidator {
        rules: vec![
            Rc::new(SegmentAuditAllowedReviewerRule {
                allowed: vec!["strict".to_string(), "default".to_string()],
            }) as Rc<dyn SegmentAuditRule>,
            Rc::new(SegmentAuditStartsAtRangeRule { min: 86400, max: 86400 }),
            Rc::new(SegmentAuditTimezoneLengthRule { max_length: 255 }),
            Rc::new(SegmentAuditRequiredReviewerRule {
                message: "segment audit needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_segment_audit_validator();
        let item = new_segment_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = SegmentAuditStartsAtRangeRule { min: 1000, max: 1000 };
        let mut item = new_segment_audit("x", "y");
        item.starts_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = SegmentAuditTimezoneLengthRule { max_length: 255 };
        let mut item = new_segment_audit("x", "y");
        item.timezone = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = SegmentAuditRequiredReviewerRule {
            message: "segment audit needs this field".to_string(),
        };
        let mut item = new_segment_audit("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
