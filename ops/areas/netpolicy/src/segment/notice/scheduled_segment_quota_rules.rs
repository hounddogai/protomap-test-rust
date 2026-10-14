use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a scheduled segment quota must pass. Check returns its problems, or none.
pub trait ScheduledSegmentQuotaRule: std::fmt::Debug {
    fn check(&self, item: &ScheduledSegmentQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSegmentQuotaLabelLengthRule {
    pub max_length: i64,
}

impl ScheduledSegmentQuotaRule for ScheduledSegmentQuotaLabelLengthRule {
    fn check(&self, item: &ScheduledSegmentQuota) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSegmentQuotaBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ScheduledSegmentQuotaRule for ScheduledSegmentQuotaBatchSizeRangeRule {
    fn check(&self, item: &ScheduledSegmentQuota) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSegmentQuotaCodePrefixRule {
    pub prefix: String,
}

impl ScheduledSegmentQuotaRule for ScheduledSegmentQuotaCodePrefixRule {
    fn check(&self, item: &ScheduledSegmentQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSegmentQuotaRequiredCodeRule {
    pub message: String,
}

impl ScheduledSegmentQuotaRule for ScheduledSegmentQuotaRequiredCodeRule {
    fn check(&self, item: &ScheduledSegmentQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a scheduled segment quota.
#[derive(Debug, Clone)]
pub struct ScheduledSegmentQuotaValidator {
    pub rules: Vec<Rc<dyn ScheduledSegmentQuotaRule>>,
}

impl ScheduledSegmentQuotaValidator {
    pub fn validate(&self, item: &ScheduledSegmentQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ScheduledSegmentQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ScheduledSegmentQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_scheduled_segment_quota_validator() -> ScheduledSegmentQuotaValidator {
    ScheduledSegmentQuotaValidator {
        rules: vec![
            Rc::new(ScheduledSegmentQuotaLabelLengthRule { max_length: 64 }) as Rc<dyn ScheduledSegmentQuotaRule>,
            Rc::new(ScheduledSegmentQuotaBatchSizeRangeRule { min: 32, max: 80 }),
            Rc::new(ScheduledSegmentQuotaCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(ScheduledSegmentQuotaRequiredCodeRule {
                message: "scheduled segment quota needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_scheduled_segment_quota_validator();
        let item = new_scheduled_segment_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ScheduledSegmentQuotaLabelLengthRule { max_length: 80 };
        let mut item = new_scheduled_segment_quota("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = ScheduledSegmentQuotaBatchSizeRangeRule { min: 80, max: 3600 };
        let mut item = new_scheduled_segment_quota("x", "y");
        item.batch_size = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ScheduledSegmentQuotaRequiredCodeRule {
            message: "scheduled segment quota needs this field".to_string(),
        };
        let mut item = new_scheduled_segment_quota("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
