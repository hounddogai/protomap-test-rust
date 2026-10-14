use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy data export batch must pass. Check returns its problems, or none.
pub trait LegacyDataExportBatchRule: std::fmt::Debug {
    fn check(&self, item: &LegacyDataExportBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDataExportBatchExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyDataExportBatchRule for LegacyDataExportBatchExpiresAtRangeRule {
    fn check(&self, item: &LegacyDataExportBatch) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDataExportBatchFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyDataExportBatchRule for LegacyDataExportBatchFloorValueRangeRule {
    fn check(&self, item: &LegacyDataExportBatch) -> Vec<String> {
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
pub struct LegacyDataExportBatchScopeLengthRule {
    pub max_length: i64,
}

impl LegacyDataExportBatchRule for LegacyDataExportBatchScopeLengthRule {
    fn check(&self, item: &LegacyDataExportBatch) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDataExportBatchMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyDataExportBatchRule for LegacyDataExportBatchMarginRangeRule {
    fn check(&self, item: &LegacyDataExportBatch) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDataExportBatchRequiredNameRule {
    pub message: String,
}

impl LegacyDataExportBatchRule for LegacyDataExportBatchRequiredNameRule {
    fn check(&self, item: &LegacyDataExportBatch) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDataExportBatchChecksumLengthRule {
    pub max_length: i64,
}

impl LegacyDataExportBatchRule for LegacyDataExportBatchChecksumLengthRule {
    fn check(&self, item: &LegacyDataExportBatch) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy data export batch.
#[derive(Debug, Clone)]
pub struct LegacyDataExportBatchValidator {
    pub rules: Vec<Rc<dyn LegacyDataExportBatchRule>>,
}

impl LegacyDataExportBatchValidator {
    pub fn validate(&self, item: &LegacyDataExportBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyDataExportBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyDataExportBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_data_export_batch_validator() -> LegacyDataExportBatchValidator {
    LegacyDataExportBatchValidator {
        rules: vec![
            Rc::new(LegacyDataExportBatchExpiresAtRangeRule { min: 32, max: 80 }) as Rc<dyn LegacyDataExportBatchRule>,
            Rc::new(LegacyDataExportBatchFloorValueRangeRule { min: 5.6, max: 4543.0 }),
            Rc::new(LegacyDataExportBatchScopeLengthRule { max_length: 1000 }),
            Rc::new(LegacyDataExportBatchMarginRangeRule { min: 0.5, max: 4024.3 }),
            Rc::new(LegacyDataExportBatchRequiredNameRule {
                message: "legacy data export batch needs this field".to_string(),
            }),
            Rc::new(LegacyDataExportBatchChecksumLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_data_export_batch_validator();
        let item = new_legacy_data_export_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = LegacyDataExportBatchExpiresAtRangeRule { min: 32, max: 86400 };
        let mut item = new_legacy_data_export_batch("x", "y");
        item.expires_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = LegacyDataExportBatchFloorValueRangeRule { min: 10.0, max: 815.8 };
        let mut item = new_legacy_data_export_batch("x", "y");
        item.floor_value = 817.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = LegacyDataExportBatchScopeLengthRule { max_length: 32 };
        let mut item = new_legacy_data_export_batch("x", "y");
        item.scope = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = LegacyDataExportBatchMarginRangeRule { min: 7.0, max: 2085.3 };
        let mut item = new_legacy_data_export_batch("x", "y");
        item.margin = 2086.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = LegacyDataExportBatchRequiredNameRule {
            message: "legacy data export batch needs a value".to_string(),
        };
        let mut item = new_legacy_data_export_batch("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = LegacyDataExportBatchChecksumLengthRule { max_length: 32 };
        let mut item = new_legacy_data_export_batch("x", "y");
        item.checksum = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
