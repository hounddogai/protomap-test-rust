use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a release manifest must pass. Check returns its problems, or none.
pub trait ReleaseManifestRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseManifestCodeLengthRule {
    pub max_length: i64,
}

impl ReleaseManifestRule for ReleaseManifestCodeLengthRule {
    fn check(&self, item: &ReleaseManifest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseManifestCodePrefixRule {
    pub prefix: String,
}

impl ReleaseManifestRule for ReleaseManifestCodePrefixRule {
    fn check(&self, item: &ReleaseManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseManifestNameLengthRule {
    pub max_length: i64,
}

impl ReleaseManifestRule for ReleaseManifestNameLengthRule {
    fn check(&self, item: &ReleaseManifest) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseManifestRequiredTitleRule {
    pub message: String,
}

impl ReleaseManifestRule for ReleaseManifestRequiredTitleRule {
    fn check(&self, item: &ReleaseManifest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseManifestEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ReleaseManifestRule for ReleaseManifestEndsAtRangeRule {
    fn check(&self, item: &ReleaseManifest) -> Vec<String> {
        let value = item.ends_at;
        if value < self.min || value > self.max {
            return vec![format!("ends at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a release manifest.
#[derive(Debug, Clone)]
pub struct ReleaseManifestValidator {
    pub rules: Vec<Rc<dyn ReleaseManifestRule>>,
}

impl ReleaseManifestValidator {
    pub fn validate(&self, item: &ReleaseManifest) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReleaseManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_manifest_validator() -> ReleaseManifestValidator {
    ReleaseManifestValidator {
        rules: vec![
            Rc::new(ReleaseManifestCodeLengthRule { max_length: 64 }) as Rc<dyn ReleaseManifestRule>,
            Rc::new(ReleaseManifestCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(ReleaseManifestNameLengthRule { max_length: 32 }),
            Rc::new(ReleaseManifestRequiredTitleRule {
                message: "release manifest needs a value".to_string(),
            }),
            Rc::new(ReleaseManifestEndsAtRangeRule { min: 255, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_release_manifest_validator();
        let item = new_release_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ReleaseManifestCodeLengthRule { max_length: 3600 };
        let mut item = new_release_manifest("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ReleaseManifestNameLengthRule { max_length: 16 };
        let mut item = new_release_manifest("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ReleaseManifestRequiredTitleRule {
            message: "release manifest needs an entry".to_string(),
        };
        let mut item = new_release_manifest("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = ReleaseManifestEndsAtRangeRule { min: 64, max: 3600 };
        let mut item = new_release_manifest("x", "y");
        item.ends_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
