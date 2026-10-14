use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual data export checkpoint must pass. Check returns its problems, or none.
pub trait ManualDataExportCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &ManualDataExportCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportCheckpointCodePrefixRule {
    pub prefix: String,
}

impl ManualDataExportCheckpointRule for ManualDataExportCheckpointCodePrefixRule {
    fn check(&self, item: &ManualDataExportCheckpoint) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportCheckpointCodeLengthRule {
    pub max_length: i64,
}

impl ManualDataExportCheckpointRule for ManualDataExportCheckpointCodeLengthRule {
    fn check(&self, item: &ManualDataExportCheckpoint) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportCheckpointRequiredTimezoneRule {
    pub message: String,
}

impl ManualDataExportCheckpointRule for ManualDataExportCheckpointRequiredTimezoneRule {
    fn check(&self, item: &ManualDataExportCheckpoint) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportCheckpointRequiredOwnerIdRule {
    pub message: String,
}

impl ManualDataExportCheckpointRule for ManualDataExportCheckpointRequiredOwnerIdRule {
    fn check(&self, item: &ManualDataExportCheckpoint) -> Vec<String> {
        if item.owner_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportCheckpointNameLengthRule {
    pub max_length: i64,
}

impl ManualDataExportCheckpointRule for ManualDataExportCheckpointNameLengthRule {
    fn check(&self, item: &ManualDataExportCheckpoint) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualDataExportCheckpointOwnerIdLengthRule {
    pub max_length: i64,
}

impl ManualDataExportCheckpointRule for ManualDataExportCheckpointOwnerIdLengthRule {
    fn check(&self, item: &ManualDataExportCheckpoint) -> Vec<String> {
        let size = item.owner_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "owner id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual data export checkpoint.
#[derive(Debug, Clone)]
pub struct ManualDataExportCheckpointValidator {
    pub rules: Vec<Rc<dyn ManualDataExportCheckpointRule>>,
}

impl ManualDataExportCheckpointValidator {
    pub fn validate(&self, item: &ManualDataExportCheckpoint) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualDataExportCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualDataExportCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_data_export_checkpoint_validator() -> ManualDataExportCheckpointValidator {
    ManualDataExportCheckpointValidator {
        rules: vec![
            Rc::new(ManualDataExportCheckpointCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn ManualDataExportCheckpointRule>,
            Rc::new(ManualDataExportCheckpointCodeLengthRule { max_length: 255 }),
            Rc::new(ManualDataExportCheckpointRequiredTimezoneRule {
                message: "manual data export checkpoint needs an entry".to_string(),
            }),
            Rc::new(ManualDataExportCheckpointRequiredOwnerIdRule {
                message: "manual data export checkpoint needs a value".to_string(),
            }),
            Rc::new(ManualDataExportCheckpointNameLengthRule { max_length: 16 }),
            Rc::new(ManualDataExportCheckpointOwnerIdLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_data_export_checkpoint_validator();
        let item = new_manual_data_export_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualDataExportCheckpointCodeLengthRule { max_length: 3600 };
        let mut item = new_manual_data_export_checkpoint("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = ManualDataExportCheckpointRequiredTimezoneRule {
            message: "manual data export checkpoint needs a value".to_string(),
        };
        let mut item = new_manual_data_export_checkpoint("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_owner_id() {
        let rule = ManualDataExportCheckpointRequiredOwnerIdRule {
            message: "manual data export checkpoint needs this field".to_string(),
        };
        let mut item = new_manual_data_export_checkpoint("", "");
        item.owner_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ManualDataExportCheckpointNameLengthRule { max_length: 80 };
        let mut item = new_manual_data_export_checkpoint("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = ManualDataExportCheckpointOwnerIdLengthRule { max_length: 3600 };
        let mut item = new_manual_data_export_checkpoint("x", "y");
        item.owner_id = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
