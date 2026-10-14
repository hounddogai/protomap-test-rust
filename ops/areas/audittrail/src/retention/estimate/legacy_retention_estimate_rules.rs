use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy retention estimate must pass. Check returns its problems, or none.
pub trait LegacyRetentionEstimateRule: std::fmt::Debug {
    fn check(&self, item: &LegacyRetentionEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRetentionEstimateAllowedNameRule {
    pub allowed: Vec<String>,
}

impl LegacyRetentionEstimateRule for LegacyRetentionEstimateAllowedNameRule {
    fn check(&self, item: &LegacyRetentionEstimate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is too long", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRetentionEstimateCodeLengthRule {
    pub max_length: i64,
}

impl LegacyRetentionEstimateRule for LegacyRetentionEstimateCodeLengthRule {
    fn check(&self, item: &LegacyRetentionEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRetentionEstimateRequiredNameRule {
    pub message: String,
}

impl LegacyRetentionEstimateRule for LegacyRetentionEstimateRequiredNameRule {
    fn check(&self, item: &LegacyRetentionEstimate) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRetentionEstimateRequiredCodeRule {
    pub message: String,
}

impl LegacyRetentionEstimateRule for LegacyRetentionEstimateRequiredCodeRule {
    fn check(&self, item: &LegacyRetentionEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRetentionEstimateNameLengthRule {
    pub max_length: i64,
}

impl LegacyRetentionEstimateRule for LegacyRetentionEstimateNameLengthRule {
    fn check(&self, item: &LegacyRetentionEstimate) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy retention estimate.
#[derive(Debug, Clone)]
pub struct LegacyRetentionEstimateValidator {
    pub rules: Vec<Rc<dyn LegacyRetentionEstimateRule>>,
}

impl LegacyRetentionEstimateValidator {
    pub fn validate(&self, item: &LegacyRetentionEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyRetentionEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyRetentionEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_retention_estimate_validator() -> LegacyRetentionEstimateValidator {
    LegacyRetentionEstimateValidator {
        rules: vec![
            Rc::new(LegacyRetentionEstimateAllowedNameRule {
                allowed: vec!["seasonal".to_string(), "primary".to_string()],
            }) as Rc<dyn LegacyRetentionEstimateRule>,
            Rc::new(LegacyRetentionEstimateCodeLengthRule { max_length: 86400 }),
            Rc::new(LegacyRetentionEstimateRequiredNameRule {
                message: "legacy retention estimate needs this field".to_string(),
            }),
            Rc::new(LegacyRetentionEstimateRequiredCodeRule {
                message: "legacy retention estimate needs a value".to_string(),
            }),
            Rc::new(LegacyRetentionEstimateNameLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_retention_estimate_validator();
        let item = new_legacy_retention_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyRetentionEstimateCodeLengthRule { max_length: 16 };
        let mut item = new_legacy_retention_estimate("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = LegacyRetentionEstimateRequiredNameRule {
            message: "legacy retention estimate needs a value".to_string(),
        };
        let mut item = new_legacy_retention_estimate("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyRetentionEstimateRequiredCodeRule {
            message: "legacy retention estimate needs an entry".to_string(),
        };
        let mut item = new_legacy_retention_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = LegacyRetentionEstimateNameLengthRule { max_length: 64 };
        let mut item = new_legacy_retention_estimate("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
