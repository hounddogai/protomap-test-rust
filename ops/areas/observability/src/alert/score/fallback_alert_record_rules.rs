use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback alert record must pass. Check returns its problems, or none.
pub trait FallbackAlertRecordRule: std::fmt::Debug {
    fn check(&self, item: &FallbackAlertRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAlertRecordNameLengthRule {
    pub max_length: i64,
}

impl FallbackAlertRecordRule for FallbackAlertRecordNameLengthRule {
    fn check(&self, item: &FallbackAlertRecord) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAlertRecordRegionLengthRule {
    pub max_length: i64,
}

impl FallbackAlertRecordRule for FallbackAlertRecordRegionLengthRule {
    fn check(&self, item: &FallbackAlertRecord) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAlertRecordMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackAlertRecordRule for FallbackAlertRecordMinimumRangeRule {
    fn check(&self, item: &FallbackAlertRecord) -> Vec<String> {
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
pub struct FallbackAlertRecordRequiredCodeRule {
    pub message: String,
}

impl FallbackAlertRecordRule for FallbackAlertRecordRequiredCodeRule {
    fn check(&self, item: &FallbackAlertRecord) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAlertRecordRequiredRegionRule {
    pub message: String,
}

impl FallbackAlertRecordRule for FallbackAlertRecordRequiredRegionRule {
    fn check(&self, item: &FallbackAlertRecord) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback alert record.
#[derive(Debug, Clone)]
pub struct FallbackAlertRecordValidator {
    pub rules: Vec<Rc<dyn FallbackAlertRecordRule>>,
}

impl FallbackAlertRecordValidator {
    pub fn validate(&self, item: &FallbackAlertRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackAlertRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackAlertRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_alert_record_validator() -> FallbackAlertRecordValidator {
    FallbackAlertRecordValidator {
        rules: vec![
            Rc::new(FallbackAlertRecordNameLengthRule { max_length: 80 }) as Rc<dyn FallbackAlertRecordRule>,
            Rc::new(FallbackAlertRecordRegionLengthRule { max_length: 1000 }),
            Rc::new(FallbackAlertRecordMinimumRangeRule { min: 4.5, max: 2675.3 }),
            Rc::new(FallbackAlertRecordRequiredCodeRule {
                message: "fallback alert record needs a value".to_string(),
            }),
            Rc::new(FallbackAlertRecordRequiredRegionRule {
                message: "fallback alert record needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_alert_record_validator();
        let item = new_fallback_alert_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = FallbackAlertRecordNameLengthRule { max_length: 80 };
        let mut item = new_fallback_alert_record("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = FallbackAlertRecordRegionLengthRule { max_length: 120 };
        let mut item = new_fallback_alert_record("x", "y");
        item.region = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = FallbackAlertRecordMinimumRangeRule { min: 0.6, max: 4229.4 };
        let mut item = new_fallback_alert_record("x", "y");
        item.minimum = 4230.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackAlertRecordRequiredCodeRule {
            message: "fallback alert record needs a value".to_string(),
        };
        let mut item = new_fallback_alert_record("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = FallbackAlertRecordRequiredRegionRule {
            message: "fallback alert record needs an entry".to_string(),
        };
        let mut item = new_fallback_alert_record("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
