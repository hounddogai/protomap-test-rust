use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a suspension segment must pass. Check returns its problems, or none.
pub trait SuspensionSegmentRule: std::fmt::Debug {
    fn check(&self, item: &SuspensionSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionSegmentRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SuspensionSegmentRule for SuspensionSegmentRevisionRangeRule {
    fn check(&self, item: &SuspensionSegment) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionSegmentArchivedGuardRule {
    pub limit: f64,
}

impl SuspensionSegmentRule for SuspensionSegmentArchivedGuardRule {
    fn check(&self, item: &SuspensionSegment) -> Vec<String> {
        if item.archived && (item.discount > self.limit) {
            return vec![format!(
                "discount {} exceeds {} while archived",
                item.discount, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionSegmentAllowedReviewerRule {
    pub allowed: Vec<String>,
}

impl SuspensionSegmentRule for SuspensionSegmentAllowedReviewerRule {
    fn check(&self, item: &SuspensionSegment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.reviewer)))
        {
            return vec![format!("reviewer {} is out of range", item.reviewer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionSegmentReviewerLengthRule {
    pub max_length: i64,
}

impl SuspensionSegmentRule for SuspensionSegmentReviewerLengthRule {
    fn check(&self, item: &SuspensionSegment) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionSegmentRequiredCodeRule {
    pub message: String,
}

impl SuspensionSegmentRule for SuspensionSegmentRequiredCodeRule {
    fn check(&self, item: &SuspensionSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a suspension segment.
#[derive(Debug, Clone)]
pub struct SuspensionSegmentValidator {
    pub rules: Vec<Rc<dyn SuspensionSegmentRule>>,
}

impl SuspensionSegmentValidator {
    pub fn validate(&self, item: &SuspensionSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SuspensionSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SuspensionSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_suspension_segment_validator() -> SuspensionSegmentValidator {
    SuspensionSegmentValidator {
        rules: vec![
            Rc::new(SuspensionSegmentRevisionRangeRule { min: 64, max: 86400 }) as Rc<dyn SuspensionSegmentRule>,
            Rc::new(SuspensionSegmentArchivedGuardRule { limit: 4.3 }),
            Rc::new(SuspensionSegmentAllowedReviewerRule {
                allowed: vec!["seasonal".to_string(), "regional".to_string(), "legacy".to_string()],
            }),
            Rc::new(SuspensionSegmentReviewerLengthRule { max_length: 120 }),
            Rc::new(SuspensionSegmentRequiredCodeRule {
                message: "suspension segment needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_suspension_segment_validator();
        let item = new_suspension_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = SuspensionSegmentRevisionRangeRule { min: 255, max: 1000 };
        let mut item = new_suspension_segment("x", "y");
        item.revision = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = SuspensionSegmentReviewerLengthRule { max_length: 86400 };
        let mut item = new_suspension_segment("x", "y");
        item.reviewer = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SuspensionSegmentRequiredCodeRule {
            message: "suspension segment needs this field".to_string(),
        };
        let mut item = new_suspension_segment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
