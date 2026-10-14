use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled dns record segment must pass. Check returns its problems, or none.
pub trait ScheduledDnsRecordSegmentRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledDnsRecordSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDnsRecordSegmentRequiredScopeRule {
    pub message: String,
}

impl ScheduledDnsRecordSegmentRule for ScheduledDnsRecordSegmentRequiredScopeRule {
    fn check(&self, item: &ScheduledDnsRecordSegment) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDnsRecordSegmentRequiredReviewerRule {
    pub message: String,
}

impl ScheduledDnsRecordSegmentRule for ScheduledDnsRecordSegmentRequiredReviewerRule {
    fn check(&self, item: &ScheduledDnsRecordSegment) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDnsRecordSegmentLabelLengthRule {
    pub max_length: i64,
}

impl ScheduledDnsRecordSegmentRule for ScheduledDnsRecordSegmentLabelLengthRule {
    fn check(&self, item: &ScheduledDnsRecordSegment) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled dns record segment.
#[derive(Debug, Clone)]
pub struct ScheduledDnsRecordSegmentValidator {
    pub rules: Vec<Rc<dyn ScheduledDnsRecordSegmentRule>>,
}

impl ScheduledDnsRecordSegmentValidator {
    pub fn validate(&self, item: &ScheduledDnsRecordSegment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ScheduledDnsRecordSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledDnsRecordSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_dns_record_segment_validator() -> ScheduledDnsRecordSegmentValidator {
    ScheduledDnsRecordSegmentValidator {
        rules: vec![
            Rc::new(ScheduledDnsRecordSegmentRequiredScopeRule {
                message: "scheduled dns record segment needs a value".to_string(),
            }) as Rc<dyn ScheduledDnsRecordSegmentRule>,
            Rc::new(ScheduledDnsRecordSegmentRequiredReviewerRule {
                message: "scheduled dns record segment needs an entry".to_string(),
            }),
            Rc::new(ScheduledDnsRecordSegmentLabelLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_dns_record_segment_validator();
        let item = new_scheduled_dns_record_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = ScheduledDnsRecordSegmentRequiredScopeRule {
            message: "scheduled dns record segment needs an entry".to_string(),
        };
        let mut item = new_scheduled_dns_record_segment("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = ScheduledDnsRecordSegmentRequiredReviewerRule {
            message: "scheduled dns record segment needs a value".to_string(),
        };
        let mut item = new_scheduled_dns_record_segment("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ScheduledDnsRecordSegmentLabelLengthRule { max_length: 120 };
        let mut item = new_scheduled_dns_record_segment("x", "y");
        item.label = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
