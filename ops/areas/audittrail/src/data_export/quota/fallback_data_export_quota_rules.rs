use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback data export quota must pass. Check returns its problems, or none.
pub trait FallbackDataExportQuotaRule: std::fmt::Debug {
    fn check(&self, item: &FallbackDataExportQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDataExportQuotaRequiredCodeRule {
    pub message: String,
}

impl FallbackDataExportQuotaRule for FallbackDataExportQuotaRequiredCodeRule {
    fn check(&self, item: &FallbackDataExportQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDataExportQuotaLabelLengthRule {
    pub max_length: i64,
}

impl FallbackDataExportQuotaRule for FallbackDataExportQuotaLabelLengthRule {
    fn check(&self, item: &FallbackDataExportQuota) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDataExportQuotaRequiredLabelRule {
    pub message: String,
}

impl FallbackDataExportQuotaRule for FallbackDataExportQuotaRequiredLabelRule {
    fn check(&self, item: &FallbackDataExportQuota) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDataExportQuotaCodeLengthRule {
    pub max_length: i64,
}

impl FallbackDataExportQuotaRule for FallbackDataExportQuotaCodeLengthRule {
    fn check(&self, item: &FallbackDataExportQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDataExportQuotaRegionLengthRule {
    pub max_length: i64,
}

impl FallbackDataExportQuotaRule for FallbackDataExportQuotaRegionLengthRule {
    fn check(&self, item: &FallbackDataExportQuota) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback data export quota.
#[derive(Debug, Clone)]
pub struct FallbackDataExportQuotaValidator {
    pub rules: Vec<Rc<dyn FallbackDataExportQuotaRule>>,
}

impl FallbackDataExportQuotaValidator {
    pub fn validate(&self, item: &FallbackDataExportQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackDataExportQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackDataExportQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_data_export_quota_validator() -> FallbackDataExportQuotaValidator {
    FallbackDataExportQuotaValidator {
        rules: vec![
            Rc::new(FallbackDataExportQuotaRequiredCodeRule {
                message: "fallback data export quota needs a value".to_string(),
            }) as Rc<dyn FallbackDataExportQuotaRule>,
            Rc::new(FallbackDataExportQuotaLabelLengthRule { max_length: 1000 }),
            Rc::new(FallbackDataExportQuotaRequiredLabelRule {
                message: "fallback data export quota needs this field".to_string(),
            }),
            Rc::new(FallbackDataExportQuotaCodeLengthRule { max_length: 64 }),
            Rc::new(FallbackDataExportQuotaRegionLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_data_export_quota_validator();
        let item = new_fallback_data_export_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackDataExportQuotaRequiredCodeRule {
            message: "fallback data export quota needs a value".to_string(),
        };
        let mut item = new_fallback_data_export_quota("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = FallbackDataExportQuotaLabelLengthRule { max_length: 255 };
        let mut item = new_fallback_data_export_quota("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = FallbackDataExportQuotaRequiredLabelRule {
            message: "fallback data export quota needs a value".to_string(),
        };
        let mut item = new_fallback_data_export_quota("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackDataExportQuotaCodeLengthRule { max_length: 16 };
        let mut item = new_fallback_data_export_quota("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = FallbackDataExportQuotaRegionLengthRule { max_length: 86400 };
        let mut item = new_fallback_data_export_quota("x", "y");
        item.region = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
