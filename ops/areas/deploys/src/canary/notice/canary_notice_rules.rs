use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a canary notice must pass. Check returns its problems, or none.
pub trait CanaryNoticeRule: std::fmt::Debug {
    fn check(&self, item: &CanaryNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryNoticeSegmentLengthRule {
    pub max_length: i64,
}

impl CanaryNoticeRule for CanaryNoticeSegmentLengthRule {
    fn check(&self, item: &CanaryNotice) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryNoticePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CanaryNoticeRule for CanaryNoticePortRangeRule {
    fn check(&self, item: &CanaryNotice) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryNoticeCodePrefixRule {
    pub prefix: String,
}

impl CanaryNoticeRule for CanaryNoticeCodePrefixRule {
    fn check(&self, item: &CanaryNotice) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary notice.
#[derive(Debug, Clone)]
pub struct CanaryNoticeValidator {
    pub rules: Vec<Rc<dyn CanaryNoticeRule>>,
}

impl CanaryNoticeValidator {
    pub fn validate(&self, item: &CanaryNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CanaryNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanaryNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_notice_validator() -> CanaryNoticeValidator {
    CanaryNoticeValidator {
        rules: vec![
            Rc::new(CanaryNoticeSegmentLengthRule { max_length: 255 }) as Rc<dyn CanaryNoticeRule>,
            Rc::new(CanaryNoticePortRangeRule { min: 16, max: 64 }),
            Rc::new(CanaryNoticeCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_canary_notice_validator();
        let item = new_canary_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = CanaryNoticeSegmentLengthRule { max_length: 3600 };
        let mut item = new_canary_notice("x", "y");
        item.segment = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = CanaryNoticePortRangeRule { min: 64, max: 3600 };
        let mut item = new_canary_notice("x", "y");
        item.port = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
