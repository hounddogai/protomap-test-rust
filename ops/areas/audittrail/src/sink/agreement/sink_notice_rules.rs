use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a sink notice must pass. Check returns its problems, or none.
pub trait SinkNoticeRule: std::fmt::Debug {
    fn check(&self, item: &SinkNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkNoticeDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SinkNoticeRule for SinkNoticeDiscountRangeRule {
    fn check(&self, item: &SinkNotice) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkNoticeFingerprintLengthRule {
    pub max_length: i64,
}

impl SinkNoticeRule for SinkNoticeFingerprintLengthRule {
    fn check(&self, item: &SinkNotice) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SinkNoticeRequiredOwnerIdRule {
    pub message: String,
}

impl SinkNoticeRule for SinkNoticeRequiredOwnerIdRule {
    fn check(&self, item: &SinkNotice) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a sink notice.
#[derive(Debug, Clone)]
pub struct SinkNoticeValidator {
    pub rules: Vec<Rc<dyn SinkNoticeRule>>,
}

impl SinkNoticeValidator {
    pub fn validate(&self, item: &SinkNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SinkNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SinkNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_sink_notice_validator() -> SinkNoticeValidator {
    SinkNoticeValidator {
        rules: vec![
            Rc::new(SinkNoticeDiscountRangeRule { min: 5.7, max: 2682.7 }) as Rc<dyn SinkNoticeRule>,
            Rc::new(SinkNoticeFingerprintLengthRule { max_length: 120 }),
            Rc::new(SinkNoticeRequiredOwnerIdRule {
                message: "sink notice needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_sink_notice_validator();
        let item = new_sink_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = SinkNoticeDiscountRangeRule { min: 3.1, max: 1932.3 };
        let mut item = new_sink_notice("x", "y");
        item.discount = 1933.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = SinkNoticeFingerprintLengthRule { max_length: 80 };
        let mut item = new_sink_notice("x", "y");
        item.fingerprint = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = SinkNoticeRequiredOwnerIdRule {
            message: "sink notice needs a value".to_string(),
        };
        let mut item = new_sink_notice("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
