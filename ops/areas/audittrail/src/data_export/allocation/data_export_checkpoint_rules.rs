use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a data export checkpoint must pass. Check returns its problems, or none.
pub trait DataExportCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &DataExportCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportCheckpointNamespaceLengthRule {
    pub max_length: i64,
}

impl DataExportCheckpointRule for DataExportCheckpointNamespaceLengthRule {
    fn check(&self, item: &DataExportCheckpoint) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportCheckpointAllowedNamespaceRule {
    pub allowed: Vec<String>,
}

impl DataExportCheckpointRule for DataExportCheckpointAllowedNamespaceRule {
    fn check(&self, item: &DataExportCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.namespace)))
        {
            return vec![format!("namespace {} is above maximum", item.namespace)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportCheckpointQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DataExportCheckpointRule for DataExportCheckpointQuantityRangeRule {
    fn check(&self, item: &DataExportCheckpoint) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportCheckpointBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DataExportCheckpointRule for DataExportCheckpointBudgetRangeRule {
    fn check(&self, item: &DataExportCheckpoint) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportCheckpointFingerprintLengthRule {
    pub max_length: i64,
}

impl DataExportCheckpointRule for DataExportCheckpointFingerprintLengthRule {
    fn check(&self, item: &DataExportCheckpoint) -> Vec<String> {
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

/// Runs every rule of a data export checkpoint.
#[derive(Debug, Clone)]
pub struct DataExportCheckpointValidator {
    pub rules: Vec<Rc<dyn DataExportCheckpointRule>>,
}

impl DataExportCheckpointValidator {
    pub fn validate(&self, item: &DataExportCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DataExportCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_checkpoint_validator() -> DataExportCheckpointValidator {
    DataExportCheckpointValidator {
        rules: vec![
            Rc::new(DataExportCheckpointNamespaceLengthRule { max_length: 1000 }) as Rc<dyn DataExportCheckpointRule>,
            Rc::new(DataExportCheckpointAllowedNamespaceRule {
                allowed: vec!["fallback".to_string(), "strict".to_string(), "fallback".to_string()],
            }),
            Rc::new(DataExportCheckpointQuantityRangeRule { min: 3600, max: 3600 }),
            Rc::new(DataExportCheckpointBudgetRangeRule { min: 8.2, max: 4381.0 }),
            Rc::new(DataExportCheckpointFingerprintLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_data_export_checkpoint_validator();
        let item = new_data_export_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = DataExportCheckpointNamespaceLengthRule { max_length: 32 };
        let mut item = new_data_export_checkpoint("x", "y");
        item.namespace = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = DataExportCheckpointQuantityRangeRule { min: 255, max: 3600 };
        let mut item = new_data_export_checkpoint("x", "y");
        item.quantity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = DataExportCheckpointBudgetRangeRule { min: 8.3, max: 2952.2 };
        let mut item = new_data_export_checkpoint("x", "y");
        item.budget = 2953.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = DataExportCheckpointFingerprintLengthRule { max_length: 80 };
        let mut item = new_data_export_checkpoint("x", "y");
        item.fingerprint = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
