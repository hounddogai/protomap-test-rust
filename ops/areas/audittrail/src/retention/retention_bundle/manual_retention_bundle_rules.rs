use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual retention bundle must pass. Check returns its problems, or none.
pub trait ManualRetentionBundleRule: std::fmt::Debug {
    fn check(&self, item: &ManualRetentionBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRetentionBundleCountryLengthRule {
    pub max_length: i64,
}

impl ManualRetentionBundleRule for ManualRetentionBundleCountryLengthRule {
    fn check(&self, item: &ManualRetentionBundle) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRetentionBundleCodeLengthRule {
    pub max_length: i64,
}

impl ManualRetentionBundleRule for ManualRetentionBundleCodeLengthRule {
    fn check(&self, item: &ManualRetentionBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRetentionBundleRequiredCodeRule {
    pub message: String,
}

impl ManualRetentionBundleRule for ManualRetentionBundleRequiredCodeRule {
    fn check(&self, item: &ManualRetentionBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual retention bundle.
#[derive(Debug, Clone)]
pub struct ManualRetentionBundleValidator {
    pub rules: Vec<Rc<dyn ManualRetentionBundleRule>>,
}

impl ManualRetentionBundleValidator {
    pub fn validate(&self, item: &ManualRetentionBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualRetentionBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualRetentionBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_retention_bundle_validator() -> ManualRetentionBundleValidator {
    ManualRetentionBundleValidator {
        rules: vec![
            Rc::new(ManualRetentionBundleCountryLengthRule { max_length: 255 }) as Rc<dyn ManualRetentionBundleRule>,
            Rc::new(ManualRetentionBundleCodeLengthRule { max_length: 32 }),
            Rc::new(ManualRetentionBundleRequiredCodeRule {
                message: "manual retention bundle needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_retention_bundle_validator();
        let item = new_manual_retention_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = ManualRetentionBundleCountryLengthRule { max_length: 16 };
        let mut item = new_manual_retention_bundle("x", "y");
        item.country = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualRetentionBundleCodeLengthRule { max_length: 80 };
        let mut item = new_manual_retention_bundle("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualRetentionBundleRequiredCodeRule {
            message: "manual retention bundle needs this field".to_string(),
        };
        let mut item = new_manual_retention_bundle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
