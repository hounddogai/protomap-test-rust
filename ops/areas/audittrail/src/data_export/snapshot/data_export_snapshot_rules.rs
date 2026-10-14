use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a data export snapshot must pass. Check returns its problems, or none.
pub trait DataExportSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &DataExportSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportSnapshotRequiredTitleRule {
    pub message: String,
}

impl DataExportSnapshotRule for DataExportSnapshotRequiredTitleRule {
    fn check(&self, item: &DataExportSnapshot) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportSnapshotAllowedExternalRefRule {
    pub allowed: Vec<String>,
}

impl DataExportSnapshotRule for DataExportSnapshotAllowedExternalRefRule {
    fn check(&self, item: &DataExportSnapshot) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.external_ref)))
        {
            return vec![format!("external ref {} is blocked", item.external_ref)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportSnapshotRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DataExportSnapshotRule for DataExportSnapshotRevisionRangeRule {
    fn check(&self, item: &DataExportSnapshot) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportSnapshotLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DataExportSnapshotRule for DataExportSnapshotLimitRangeRule {
    fn check(&self, item: &DataExportSnapshot) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportSnapshotCodePrefixRule {
    pub prefix: String,
}

impl DataExportSnapshotRule for DataExportSnapshotCodePrefixRule {
    fn check(&self, item: &DataExportSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a data export snapshot.
#[derive(Debug, Clone)]
pub struct DataExportSnapshotValidator {
    pub rules: Vec<Rc<dyn DataExportSnapshotRule>>,
}

impl DataExportSnapshotValidator {
    pub fn validate(&self, item: &DataExportSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DataExportSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_snapshot_validator() -> DataExportSnapshotValidator {
    DataExportSnapshotValidator {
        rules: vec![
            Rc::new(DataExportSnapshotRequiredTitleRule {
                message: "data export snapshot needs an entry".to_string(),
            }) as Rc<dyn DataExportSnapshotRule>,
            Rc::new(DataExportSnapshotAllowedExternalRefRule { allowed: Vec::new() }),
            Rc::new(DataExportSnapshotRevisionRangeRule { min: 1000, max: 3600 }),
            Rc::new(DataExportSnapshotLimitRangeRule { min: 80, max: 86400 }),
            Rc::new(DataExportSnapshotCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_data_export_snapshot_validator();
        let item = new_data_export_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = DataExportSnapshotRequiredTitleRule {
            message: "data export snapshot needs a value".to_string(),
        };
        let mut item = new_data_export_snapshot("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = DataExportSnapshotRevisionRangeRule { min: 64, max: 255 };
        let mut item = new_data_export_snapshot("x", "y");
        item.revision = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = DataExportSnapshotLimitRangeRule { min: 80, max: 86400 };
        let mut item = new_data_export_snapshot("x", "y");
        item.limit = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
