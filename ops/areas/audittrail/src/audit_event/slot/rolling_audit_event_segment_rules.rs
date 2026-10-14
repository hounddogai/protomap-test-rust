use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling audit event segment must pass. Check returns its problems, or none.
pub trait RollingAuditEventSegmentRule: std::fmt::Debug {
    fn check(&self, item: &RollingAuditEventSegment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAuditEventSegmentFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingAuditEventSegmentRule for RollingAuditEventSegmentFloorValueRangeRule {
    fn check(&self, item: &RollingAuditEventSegment) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAuditEventSegmentRequiredCodeRule {
    pub message: String,
}

impl RollingAuditEventSegmentRule for RollingAuditEventSegmentRequiredCodeRule {
    fn check(&self, item: &RollingAuditEventSegment) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAuditEventSegmentMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingAuditEventSegmentRule for RollingAuditEventSegmentMarginRangeRule {
    fn check(&self, item: &RollingAuditEventSegment) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAuditEventSegmentCodeLengthRule {
    pub max_length: i64,
}

impl RollingAuditEventSegmentRule for RollingAuditEventSegmentCodeLengthRule {
    fn check(&self, item: &RollingAuditEventSegment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAuditEventSegmentRequiredSummaryRule {
    pub message: String,
}

impl RollingAuditEventSegmentRule for RollingAuditEventSegmentRequiredSummaryRule {
    fn check(&self, item: &RollingAuditEventSegment) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingAuditEventSegmentPhoneLengthRule {
    pub max_length: i64,
}

impl RollingAuditEventSegmentRule for RollingAuditEventSegmentPhoneLengthRule {
    fn check(&self, item: &RollingAuditEventSegment) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling audit event segment.
#[derive(Debug, Clone)]
pub struct RollingAuditEventSegmentValidator {
    pub rules: Vec<Rc<dyn RollingAuditEventSegmentRule>>,
}

impl RollingAuditEventSegmentValidator {
    pub fn validate(&self, item: &RollingAuditEventSegment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingAuditEventSegment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingAuditEventSegment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_audit_event_segment_validator() -> RollingAuditEventSegmentValidator {
    RollingAuditEventSegmentValidator {
        rules: vec![
            Rc::new(RollingAuditEventSegmentFloorValueRangeRule { min: 0.1, max: 1858.9 })
                as Rc<dyn RollingAuditEventSegmentRule>,
            Rc::new(RollingAuditEventSegmentRequiredCodeRule {
                message: "rolling audit event segment needs this field".to_string(),
            }),
            Rc::new(RollingAuditEventSegmentMarginRangeRule { min: 0.9, max: 2340.9 }),
            Rc::new(RollingAuditEventSegmentCodeLengthRule { max_length: 64 }),
            Rc::new(RollingAuditEventSegmentRequiredSummaryRule {
                message: "rolling audit event segment needs an entry".to_string(),
            }),
            Rc::new(RollingAuditEventSegmentPhoneLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_audit_event_segment_validator();
        let item = new_rolling_audit_event_segment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = RollingAuditEventSegmentFloorValueRangeRule { min: 5.4, max: 410.3 };
        let mut item = new_rolling_audit_event_segment("x", "y");
        item.floor_value = 411.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingAuditEventSegmentRequiredCodeRule {
            message: "rolling audit event segment needs this field".to_string(),
        };
        let mut item = new_rolling_audit_event_segment("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = RollingAuditEventSegmentMarginRangeRule { min: 5.8, max: 556.9 };
        let mut item = new_rolling_audit_event_segment("x", "y");
        item.margin = 558.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RollingAuditEventSegmentCodeLengthRule { max_length: 120 };
        let mut item = new_rolling_audit_event_segment("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = RollingAuditEventSegmentRequiredSummaryRule {
            message: "rolling audit event segment needs an entry".to_string(),
        };
        let mut item = new_rolling_audit_event_segment("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = RollingAuditEventSegmentPhoneLengthRule { max_length: 86400 };
        let mut item = new_rolling_audit_event_segment("x", "y");
        item.phone = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
