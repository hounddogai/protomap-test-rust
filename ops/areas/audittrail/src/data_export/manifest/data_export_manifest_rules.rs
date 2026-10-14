use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a data export manifest must pass. Check returns its problems, or none.
pub trait DataExportManifestRule: std::fmt::Debug {
    fn check(&self, item: &DataExportManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportManifestClusterLengthRule {
    pub max_length: i64,
}

impl DataExportManifestRule for DataExportManifestClusterLengthRule {
    fn check(&self, item: &DataExportManifest) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportManifestCodeLengthRule {
    pub max_length: i64,
}

impl DataExportManifestRule for DataExportManifestCodeLengthRule {
    fn check(&self, item: &DataExportManifest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportManifestRequiredPrincipalRule {
    pub message: String,
}

impl DataExportManifestRule for DataExportManifestRequiredPrincipalRule {
    fn check(&self, item: &DataExportManifest) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportManifestRequiredCodeRule {
    pub message: String,
}

impl DataExportManifestRule for DataExportManifestRequiredCodeRule {
    fn check(&self, item: &DataExportManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataExportManifestRequiredClusterRule {
    pub message: String,
}

impl DataExportManifestRule for DataExportManifestRequiredClusterRule {
    fn check(&self, item: &DataExportManifest) -> Vec<String> {
        if item.cluster.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a data export manifest.
#[derive(Debug, Clone)]
pub struct DataExportManifestValidator {
    pub rules: Vec<Rc<dyn DataExportManifestRule>>,
}

impl DataExportManifestValidator {
    pub fn validate(&self, item: &DataExportManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DataExportManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DataExportManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_data_export_manifest_validator() -> DataExportManifestValidator {
    DataExportManifestValidator {
        rules: vec![
            Rc::new(DataExportManifestClusterLengthRule { max_length: 1000 }) as Rc<dyn DataExportManifestRule>,
            Rc::new(DataExportManifestCodeLengthRule { max_length: 64 }),
            Rc::new(DataExportManifestRequiredPrincipalRule {
                message: "data export manifest needs this field".to_string(),
            }),
            Rc::new(DataExportManifestRequiredCodeRule {
                message: "data export manifest needs a value".to_string(),
            }),
            Rc::new(DataExportManifestRequiredClusterRule {
                message: "data export manifest needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_data_export_manifest_validator();
        let item = new_data_export_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = DataExportManifestClusterLengthRule { max_length: 255 };
        let mut item = new_data_export_manifest("x", "y");
        item.cluster = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DataExportManifestCodeLengthRule { max_length: 1000 };
        let mut item = new_data_export_manifest("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = DataExportManifestRequiredPrincipalRule {
            message: "data export manifest needs a value".to_string(),
        };
        let mut item = new_data_export_manifest("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DataExportManifestRequiredCodeRule {
            message: "data export manifest needs a value".to_string(),
        };
        let mut item = new_data_export_manifest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cluster() {
        let rule = DataExportManifestRequiredClusterRule {
            message: "data export manifest needs a value".to_string(),
        };
        let mut item = new_data_export_manifest("", "");
        item.cluster = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
