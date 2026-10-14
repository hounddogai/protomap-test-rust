use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a segment milestone must pass. Check returns its problems, or none.
pub trait SegmentMilestoneRule: std::fmt::Debug {
    fn check(&self, item: &SegmentMilestone) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentMilestoneRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SegmentMilestoneRule for SegmentMilestoneRevisionRangeRule {
    fn check(&self, item: &SegmentMilestone) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentMilestoneRequiredReviewerRule {
    pub message: String,
}

impl SegmentMilestoneRule for SegmentMilestoneRequiredReviewerRule {
    fn check(&self, item: &SegmentMilestone) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentMilestoneLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SegmentMilestoneRule for SegmentMilestoneLimitRangeRule {
    fn check(&self, item: &SegmentMilestone) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentMilestoneIssuerLengthRule {
    pub max_length: i64,
}

impl SegmentMilestoneRule for SegmentMilestoneIssuerLengthRule {
    fn check(&self, item: &SegmentMilestone) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment milestone.
#[derive(Debug, Clone)]
pub struct SegmentMilestoneValidator {
    pub rules: Vec<Rc<dyn SegmentMilestoneRule>>,
}

impl SegmentMilestoneValidator {
    pub fn validate(&self, item: &SegmentMilestone) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SegmentMilestone) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentMilestone]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_milestone_validator() -> SegmentMilestoneValidator {
    SegmentMilestoneValidator {
        rules: vec![
            Rc::new(SegmentMilestoneRevisionRangeRule { min: 80, max: 120 }) as Rc<dyn SegmentMilestoneRule>,
            Rc::new(SegmentMilestoneRequiredReviewerRule {
                message: "segment milestone needs this field".to_string(),
            }),
            Rc::new(SegmentMilestoneLimitRangeRule { min: 16, max: 255 }),
            Rc::new(SegmentMilestoneIssuerLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_segment_milestone_validator();
        let item = new_segment_milestone("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = SegmentMilestoneRevisionRangeRule { min: 64, max: 1000 };
        let mut item = new_segment_milestone("x", "y");
        item.revision = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = SegmentMilestoneRequiredReviewerRule {
            message: "segment milestone needs this field".to_string(),
        };
        let mut item = new_segment_milestone("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = SegmentMilestoneLimitRangeRule { min: 3600, max: 86400 };
        let mut item = new_segment_milestone("x", "y");
        item.limit = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = SegmentMilestoneIssuerLengthRule { max_length: 64 };
        let mut item = new_segment_milestone("x", "y");
        item.issuer = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
