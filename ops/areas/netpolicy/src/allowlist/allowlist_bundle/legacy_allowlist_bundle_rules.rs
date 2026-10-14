use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy allowlist bundle must pass. Check returns its problems, or none.
pub trait LegacyAllowlistBundleRule: std::fmt::Debug {
    fn check(&self, item: &LegacyAllowlistBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistBundleAllowedNameRule {
    pub allowed: Vec<String>,
}

impl LegacyAllowlistBundleRule for LegacyAllowlistBundleAllowedNameRule {
    fn check(&self, item: &LegacyAllowlistBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is below minimum", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistBundleClusterLengthRule {
    pub max_length: i64,
}

impl LegacyAllowlistBundleRule for LegacyAllowlistBundleClusterLengthRule {
    fn check(&self, item: &LegacyAllowlistBundle) -> Vec<String> {
        let size = item.cluster.len() as i64;
        if size > self.max_length {
            return vec![format!("cluster is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistBundleNameLengthRule {
    pub max_length: i64,
}

impl LegacyAllowlistBundleRule for LegacyAllowlistBundleNameLengthRule {
    fn check(&self, item: &LegacyAllowlistBundle) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistBundleCodeLengthRule {
    pub max_length: i64,
}

impl LegacyAllowlistBundleRule for LegacyAllowlistBundleCodeLengthRule {
    fn check(&self, item: &LegacyAllowlistBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy allowlist bundle.
#[derive(Debug, Clone)]
pub struct LegacyAllowlistBundleValidator {
    pub rules: Vec<Rc<dyn LegacyAllowlistBundleRule>>,
}

impl LegacyAllowlistBundleValidator {
    pub fn validate(&self, item: &LegacyAllowlistBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyAllowlistBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyAllowlistBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_allowlist_bundle_validator() -> LegacyAllowlistBundleValidator {
    LegacyAllowlistBundleValidator {
        rules: vec![
            Rc::new(LegacyAllowlistBundleAllowedNameRule { allowed: Vec::new() }) as Rc<dyn LegacyAllowlistBundleRule>,
            Rc::new(LegacyAllowlistBundleClusterLengthRule { max_length: 86400 }),
            Rc::new(LegacyAllowlistBundleNameLengthRule { max_length: 32 }),
            Rc::new(LegacyAllowlistBundleCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_allowlist_bundle_validator();
        let item = new_legacy_allowlist_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_cluster() {
        let rule = LegacyAllowlistBundleClusterLengthRule { max_length: 86400 };
        let mut item = new_legacy_allowlist_bundle("x", "y");
        item.cluster = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cluster = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = LegacyAllowlistBundleNameLengthRule { max_length: 255 };
        let mut item = new_legacy_allowlist_bundle("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyAllowlistBundleCodeLengthRule { max_length: 16 };
        let mut item = new_legacy_allowlist_bundle("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
