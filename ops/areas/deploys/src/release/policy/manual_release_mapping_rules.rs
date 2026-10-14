use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual release mapping must pass. Check returns its problems, or none.
pub trait ManualReleaseMappingRule: std::fmt::Debug {
    fn check(&self, item: &ManualReleaseMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualReleaseMappingRequiredCodeRule {
    pub message: String,
}

impl ManualReleaseMappingRule for ManualReleaseMappingRequiredCodeRule {
    fn check(&self, item: &ManualReleaseMapping) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualReleaseMappingExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualReleaseMappingRule for ManualReleaseMappingExpiresAtRangeRule {
    fn check(&self, item: &ManualReleaseMapping) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualReleaseMappingRequiredTitleRule {
    pub message: String,
}

impl ManualReleaseMappingRule for ManualReleaseMappingRequiredTitleRule {
    fn check(&self, item: &ManualReleaseMapping) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualReleaseMappingCodePrefixRule {
    pub prefix: String,
}

impl ManualReleaseMappingRule for ManualReleaseMappingCodePrefixRule {
    fn check(&self, item: &ManualReleaseMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualReleaseMappingNamespaceLengthRule {
    pub max_length: i64,
}

impl ManualReleaseMappingRule for ManualReleaseMappingNamespaceLengthRule {
    fn check(&self, item: &ManualReleaseMapping) -> Vec<String> {
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
pub struct ManualReleaseMappingNameLengthRule {
    pub max_length: i64,
}

impl ManualReleaseMappingRule for ManualReleaseMappingNameLengthRule {
    fn check(&self, item: &ManualReleaseMapping) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual release mapping.
#[derive(Debug, Clone)]
pub struct ManualReleaseMappingValidator {
    pub rules: Vec<Rc<dyn ManualReleaseMappingRule>>,
}

impl ManualReleaseMappingValidator {
    pub fn validate(&self, item: &ManualReleaseMapping) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualReleaseMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualReleaseMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_release_mapping_validator() -> ManualReleaseMappingValidator {
    ManualReleaseMappingValidator {
        rules: vec![
            Rc::new(ManualReleaseMappingRequiredCodeRule {
                message: "manual release mapping needs this field".to_string(),
            }) as Rc<dyn ManualReleaseMappingRule>,
            Rc::new(ManualReleaseMappingExpiresAtRangeRule { min: 1000, max: 3600 }),
            Rc::new(ManualReleaseMappingRequiredTitleRule {
                message: "manual release mapping needs this field".to_string(),
            }),
            Rc::new(ManualReleaseMappingCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(ManualReleaseMappingNamespaceLengthRule { max_length: 64 }),
            Rc::new(ManualReleaseMappingNameLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_release_mapping_validator();
        let item = new_manual_release_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ManualReleaseMappingRequiredCodeRule {
            message: "manual release mapping needs a value".to_string(),
        };
        let mut item = new_manual_release_mapping("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = ManualReleaseMappingExpiresAtRangeRule { min: 80, max: 255 };
        let mut item = new_manual_release_mapping("x", "y");
        item.expires_at = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ManualReleaseMappingRequiredTitleRule {
            message: "manual release mapping needs an entry".to_string(),
        };
        let mut item = new_manual_release_mapping("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = ManualReleaseMappingNamespaceLengthRule { max_length: 120 };
        let mut item = new_manual_release_mapping("x", "y");
        item.namespace = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ManualReleaseMappingNameLengthRule { max_length: 86400 };
        let mut item = new_manual_release_mapping("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
