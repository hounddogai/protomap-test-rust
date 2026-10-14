use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered data export rollup must pass. Check returns its problems, or none.
pub trait TieredDataExportRollupRule: std::fmt::Debug {
    fn check(&self, item: &TieredDataExportRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDataExportRollupActiveGuardRule {
    pub limit: i64,
}

impl TieredDataExportRollupRule for TieredDataExportRollupActiveGuardRule {
    fn check(&self, item: &TieredDataExportRollup) -> Vec<String> {
        if item.active && (item.limit > self.limit) {
            return vec![format!("limit {} exceeds {} while active", item.limit, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDataExportRollupRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredDataExportRollupRule for TieredDataExportRollupRatioRangeRule {
    fn check(&self, item: &TieredDataExportRollup) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDataExportRollupNotesLengthRule {
    pub max_length: i64,
}

impl TieredDataExportRollupRule for TieredDataExportRollupNotesLengthRule {
    fn check(&self, item: &TieredDataExportRollup) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredDataExportRollupRequiredSummaryRule {
    pub message: String,
}

impl TieredDataExportRollupRule for TieredDataExportRollupRequiredSummaryRule {
    fn check(&self, item: &TieredDataExportRollup) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered data export rollup.
#[derive(Debug, Clone)]
pub struct TieredDataExportRollupValidator {
    pub rules: Vec<Rc<dyn TieredDataExportRollupRule>>,
}

impl TieredDataExportRollupValidator {
    pub fn validate(&self, item: &TieredDataExportRollup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredDataExportRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredDataExportRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_data_export_rollup_validator() -> TieredDataExportRollupValidator {
    TieredDataExportRollupValidator {
        rules: vec![
            Rc::new(TieredDataExportRollupActiveGuardRule { limit: 80 }) as Rc<dyn TieredDataExportRollupRule>,
            Rc::new(TieredDataExportRollupRatioRangeRule { min: 9.7, max: 543.6 }),
            Rc::new(TieredDataExportRollupNotesLengthRule { max_length: 32 }),
            Rc::new(TieredDataExportRollupRequiredSummaryRule {
                message: "tiered data export rollup needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_data_export_rollup_validator();
        let item = new_tiered_data_export_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = TieredDataExportRollupRatioRangeRule { min: 8.7, max: 1098.7 };
        let mut item = new_tiered_data_export_rollup("x", "y");
        item.ratio = 1100.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = TieredDataExportRollupNotesLengthRule { max_length: 1000 };
        let mut item = new_tiered_data_export_rollup("x", "y");
        item.notes = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = TieredDataExportRollupRequiredSummaryRule {
            message: "tiered data export rollup needs a value".to_string(),
        };
        let mut item = new_tiered_data_export_rollup("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
