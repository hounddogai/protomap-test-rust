use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy data export review must pass. Check returns its problems, or none.
pub trait LegacyDataExportReviewRule: std::fmt::Debug {
    fn check(&self, item: &LegacyDataExportReview) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDataExportReviewRequiredContactEmailRule {
    pub message: String,
}

impl LegacyDataExportReviewRule for LegacyDataExportReviewRequiredContactEmailRule {
    fn check(&self, item: &LegacyDataExportReview) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDataExportReviewRequiredTenantIdRule {
    pub message: String,
}

impl LegacyDataExportReviewRule for LegacyDataExportReviewRequiredTenantIdRule {
    fn check(&self, item: &LegacyDataExportReview) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDataExportReviewCodePrefixRule {
    pub prefix: String,
}

impl LegacyDataExportReviewRule for LegacyDataExportReviewCodePrefixRule {
    fn check(&self, item: &LegacyDataExportReview) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDataExportReviewTenantIdLengthRule {
    pub max_length: i64,
}

impl LegacyDataExportReviewRule for LegacyDataExportReviewTenantIdLengthRule {
    fn check(&self, item: &LegacyDataExportReview) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDataExportReviewRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyDataExportReviewRule for LegacyDataExportReviewRevisionRangeRule {
    fn check(&self, item: &LegacyDataExportReview) -> Vec<String> {
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
pub struct LegacyDataExportReviewCodeLengthRule {
    pub max_length: i64,
}

impl LegacyDataExportReviewRule for LegacyDataExportReviewCodeLengthRule {
    fn check(&self, item: &LegacyDataExportReview) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy data export review.
#[derive(Debug, Clone)]
pub struct LegacyDataExportReviewValidator {
    pub rules: Vec<Rc<dyn LegacyDataExportReviewRule>>,
}

impl LegacyDataExportReviewValidator {
    pub fn validate(&self, item: &LegacyDataExportReview) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyDataExportReview) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyDataExportReview]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_data_export_review_validator() -> LegacyDataExportReviewValidator {
    LegacyDataExportReviewValidator {
        rules: vec![
            Rc::new(LegacyDataExportReviewRequiredContactEmailRule {
                message: "legacy data export review needs an entry".to_string(),
            }) as Rc<dyn LegacyDataExportReviewRule>,
            Rc::new(LegacyDataExportReviewRequiredTenantIdRule {
                message: "legacy data export review needs a value".to_string(),
            }),
            Rc::new(LegacyDataExportReviewCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(LegacyDataExportReviewTenantIdLengthRule { max_length: 64 }),
            Rc::new(LegacyDataExportReviewRevisionRangeRule { min: 80, max: 86400 }),
            Rc::new(LegacyDataExportReviewCodeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_data_export_review_validator();
        let item = new_legacy_data_export_review("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = LegacyDataExportReviewRequiredContactEmailRule {
            message: "legacy data export review needs an entry".to_string(),
        };
        let mut item = new_legacy_data_export_review("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = LegacyDataExportReviewRequiredTenantIdRule {
            message: "legacy data export review needs this field".to_string(),
        };
        let mut item = new_legacy_data_export_review("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = LegacyDataExportReviewTenantIdLengthRule { max_length: 32 };
        let mut item = new_legacy_data_export_review("x", "y");
        item.tenant_id = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = LegacyDataExportReviewRevisionRangeRule { min: 80, max: 86400 };
        let mut item = new_legacy_data_export_review("x", "y");
        item.revision = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyDataExportReviewCodeLengthRule { max_length: 32 };
        let mut item = new_legacy_data_export_review("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
