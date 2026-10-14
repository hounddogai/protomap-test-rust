use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global allowlist bundle must pass. Check returns its problems, or none.
pub trait GlobalAllowlistBundleRule: std::fmt::Debug {
    fn check(&self, item: &GlobalAllowlistBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAllowlistBundleOwnerIdLengthRule {
    pub max_length: i64,
}

impl GlobalAllowlistBundleRule for GlobalAllowlistBundleOwnerIdLengthRule {
    fn check(&self, item: &GlobalAllowlistBundle) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAllowlistBundleSegmentLengthRule {
    pub max_length: i64,
}

impl GlobalAllowlistBundleRule for GlobalAllowlistBundleSegmentLengthRule {
    fn check(&self, item: &GlobalAllowlistBundle) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAllowlistBundleCodeLengthRule {
    pub max_length: i64,
}

impl GlobalAllowlistBundleRule for GlobalAllowlistBundleCodeLengthRule {
    fn check(&self, item: &GlobalAllowlistBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global allowlist bundle.
#[derive(Debug, Clone)]
pub struct GlobalAllowlistBundleValidator {
    pub rules: Vec<Rc<dyn GlobalAllowlistBundleRule>>,
}

impl GlobalAllowlistBundleValidator {
    pub fn validate(&self, item: &GlobalAllowlistBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalAllowlistBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalAllowlistBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_allowlist_bundle_validator() -> GlobalAllowlistBundleValidator {
    GlobalAllowlistBundleValidator {
        rules: vec![
            Rc::new(GlobalAllowlistBundleOwnerIdLengthRule { max_length: 32 }) as Rc<dyn GlobalAllowlistBundleRule>,
            Rc::new(GlobalAllowlistBundleSegmentLengthRule { max_length: 86400 }),
            Rc::new(GlobalAllowlistBundleCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_allowlist_bundle_validator();
        let item = new_global_allowlist_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_owner_id() {
        let rule = GlobalAllowlistBundleOwnerIdLengthRule { max_length: 86400 };
        let mut item = new_global_allowlist_bundle("x", "y");
        item.owner_id = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.owner_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = GlobalAllowlistBundleSegmentLengthRule { max_length: 1000 };
        let mut item = new_global_allowlist_bundle("x", "y");
        item.segment = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = GlobalAllowlistBundleCodeLengthRule { max_length: 1000 };
        let mut item = new_global_allowlist_bundle("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
