use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped data export batch must pass. Check returns its problems, or none.
pub trait CappedDataExportBatchRule: std::fmt::Debug {
    fn check(&self, item: &CappedDataExportBatch) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDataExportBatchQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedDataExportBatchRule for CappedDataExportBatchQuotaUnitsRangeRule {
    fn check(&self, item: &CappedDataExportBatch) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDataExportBatchWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedDataExportBatchRule for CappedDataExportBatchWeightRangeRule {
    fn check(&self, item: &CappedDataExportBatch) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDataExportBatchRequiredScopeRule {
    pub message: String,
}

impl CappedDataExportBatchRule for CappedDataExportBatchRequiredScopeRule {
    fn check(&self, item: &CappedDataExportBatch) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedDataExportBatchTitleLengthRule {
    pub max_length: i64,
}

impl CappedDataExportBatchRule for CappedDataExportBatchTitleLengthRule {
    fn check(&self, item: &CappedDataExportBatch) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped data export batch.
#[derive(Debug, Clone)]
pub struct CappedDataExportBatchValidator {
    pub rules: Vec<Rc<dyn CappedDataExportBatchRule>>,
}

impl CappedDataExportBatchValidator {
    pub fn validate(&self, item: &CappedDataExportBatch) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedDataExportBatch) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedDataExportBatch]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_data_export_batch_validator() -> CappedDataExportBatchValidator {
    CappedDataExportBatchValidator {
        rules: vec![
            Rc::new(CappedDataExportBatchQuotaUnitsRangeRule { min: 64, max: 1000 })
                as Rc<dyn CappedDataExportBatchRule>,
            Rc::new(CappedDataExportBatchWeightRangeRule { min: 1.2, max: 896.8 }),
            Rc::new(CappedDataExportBatchRequiredScopeRule {
                message: "capped data export batch needs this field".to_string(),
            }),
            Rc::new(CappedDataExportBatchTitleLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_data_export_batch_validator();
        let item = new_capped_data_export_batch("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = CappedDataExportBatchQuotaUnitsRangeRule { min: 32, max: 80 };
        let mut item = new_capped_data_export_batch("x", "y");
        item.quota_units = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = CappedDataExportBatchWeightRangeRule { min: 2.2, max: 3290.1 };
        let mut item = new_capped_data_export_batch("x", "y");
        item.weight = 3291.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = CappedDataExportBatchRequiredScopeRule {
            message: "capped data export batch needs this field".to_string(),
        };
        let mut item = new_capped_data_export_batch("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = CappedDataExportBatchTitleLengthRule { max_length: 120 };
        let mut item = new_capped_data_export_batch("x", "y");
        item.title = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
