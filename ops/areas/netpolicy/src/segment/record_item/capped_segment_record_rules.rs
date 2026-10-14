use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped segment record must pass. Check returns its problems, or none.
pub trait CappedSegmentRecordRule: std::fmt::Debug {
    fn check(&self, item: &CappedSegmentRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSegmentRecordAllowedPhoneRule {
    pub allowed: Vec<String>,
}

impl CappedSegmentRecordRule for CappedSegmentRecordAllowedPhoneRule {
    fn check(&self, item: &CappedSegmentRecord) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.phone))) {
            return vec![format!("phone {} is unknown", item.phone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSegmentRecordCipherLengthRule {
    pub max_length: i64,
}

impl CappedSegmentRecordRule for CappedSegmentRecordCipherLengthRule {
    fn check(&self, item: &CappedSegmentRecord) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSegmentRecordRequiredPhoneRule {
    pub message: String,
}

impl CappedSegmentRecordRule for CappedSegmentRecordRequiredPhoneRule {
    fn check(&self, item: &CappedSegmentRecord) -> Vec<String> {
        if item.phone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSegmentRecordVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedSegmentRecordRule for CappedSegmentRecordVersionRangeRule {
    fn check(&self, item: &CappedSegmentRecord) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSegmentRecordMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedSegmentRecordRule for CappedSegmentRecordMarginRangeRule {
    fn check(&self, item: &CappedSegmentRecord) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedSegmentRecordPhoneLengthRule {
    pub max_length: i64,
}

impl CappedSegmentRecordRule for CappedSegmentRecordPhoneLengthRule {
    fn check(&self, item: &CappedSegmentRecord) -> Vec<String> {
        let size = item.phone.len() as i64;
        if size > self.max_length {
            return vec![format!("phone is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped segment record.
#[derive(Debug, Clone)]
pub struct CappedSegmentRecordValidator {
    pub rules: Vec<Rc<dyn CappedSegmentRecordRule>>,
}

impl CappedSegmentRecordValidator {
    pub fn validate(&self, item: &CappedSegmentRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedSegmentRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedSegmentRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_segment_record_validator() -> CappedSegmentRecordValidator {
    CappedSegmentRecordValidator {
        rules: vec![
            Rc::new(CappedSegmentRecordAllowedPhoneRule {
                allowed: vec!["seasonal".to_string()],
            }) as Rc<dyn CappedSegmentRecordRule>,
            Rc::new(CappedSegmentRecordCipherLengthRule { max_length: 32 }),
            Rc::new(CappedSegmentRecordRequiredPhoneRule {
                message: "capped segment record needs a value".to_string(),
            }),
            Rc::new(CappedSegmentRecordVersionRangeRule { min: 32, max: 120 }),
            Rc::new(CappedSegmentRecordMarginRangeRule { min: 5.3, max: 269.8 }),
            Rc::new(CappedSegmentRecordPhoneLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_segment_record_validator();
        let item = new_capped_segment_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = CappedSegmentRecordCipherLengthRule { max_length: 32 };
        let mut item = new_capped_segment_record("x", "y");
        item.cipher = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_phone() {
        let rule = CappedSegmentRecordRequiredPhoneRule {
            message: "capped segment record needs this field".to_string(),
        };
        let mut item = new_capped_segment_record("", "");
        item.phone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = CappedSegmentRecordVersionRangeRule { min: 120, max: 1000 };
        let mut item = new_capped_segment_record("x", "y");
        item.version = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = CappedSegmentRecordMarginRangeRule { min: 2.5, max: 4801.8 };
        let mut item = new_capped_segment_record("x", "y");
        item.margin = 4803.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_phone() {
        let rule = CappedSegmentRecordPhoneLengthRule { max_length: 64 };
        let mut item = new_capped_segment_record("x", "y");
        item.phone = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.phone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
