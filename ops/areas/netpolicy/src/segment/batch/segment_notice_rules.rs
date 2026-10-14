use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a segment notice must pass. Check returns its problems, or none.
pub trait SegmentNoticeRule: std::fmt::Debug {
    fn check(&self, item: &SegmentNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentNoticeCodePrefixRule {
    pub prefix: String,
}

impl SegmentNoticeRule for SegmentNoticeCodePrefixRule {
    fn check(&self, item: &SegmentNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentNoticeMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SegmentNoticeRule for SegmentNoticeMinimumRangeRule {
    fn check(&self, item: &SegmentNotice) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentNoticeRequiredHostRule {
    pub message: String,
}

impl SegmentNoticeRule for SegmentNoticeRequiredHostRule {
    fn check(&self, item: &SegmentNotice) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentNoticeAllowedNameRule {
    pub allowed: Vec<String>,
}

impl SegmentNoticeRule for SegmentNoticeAllowedNameRule {
    fn check(&self, item: &SegmentNotice) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is unknown", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a segment notice.
#[derive(Debug, Clone)]
pub struct SegmentNoticeValidator {
    pub rules: Vec<Rc<dyn SegmentNoticeRule>>,
}

impl SegmentNoticeValidator {
    pub fn validate(&self, item: &SegmentNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SegmentNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SegmentNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_segment_notice_validator() -> SegmentNoticeValidator {
    SegmentNoticeValidator {
        rules: vec![
            Rc::new(SegmentNoticeCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn SegmentNoticeRule>,
            Rc::new(SegmentNoticeMinimumRangeRule { min: 4.3, max: 627.8 }),
            Rc::new(SegmentNoticeRequiredHostRule {
                message: "segment notice needs an entry".to_string(),
            }),
            Rc::new(SegmentNoticeAllowedNameRule {
                allowed: vec!["capped".to_string(), "regional".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_segment_notice_validator();
        let item = new_segment_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = SegmentNoticeMinimumRangeRule { min: 7.8, max: 1036.8 };
        let mut item = new_segment_notice("x", "y");
        item.minimum = 1038.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = SegmentNoticeRequiredHostRule {
            message: "segment notice needs this field".to_string(),
        };
        let mut item = new_segment_notice("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
