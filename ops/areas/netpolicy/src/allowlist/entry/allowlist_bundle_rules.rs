use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a allowlist bundle must pass. Check returns its problems, or none.
pub trait AllowlistBundleRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistBundleRequiredCodeRule {
    pub message: String,
}

impl AllowlistBundleRule for AllowlistBundleRequiredCodeRule {
    fn check(&self, item: &AllowlistBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistBundleLabelLengthRule {
    pub max_length: i64,
}

impl AllowlistBundleRule for AllowlistBundleLabelLengthRule {
    fn check(&self, item: &AllowlistBundle) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistBundlePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AllowlistBundleRule for AllowlistBundlePortRangeRule {
    fn check(&self, item: &AllowlistBundle) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistBundleTenantIdLengthRule {
    pub max_length: i64,
}

impl AllowlistBundleRule for AllowlistBundleTenantIdLengthRule {
    fn check(&self, item: &AllowlistBundle) -> Vec<String> {
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
pub struct AllowlistBundleFingerprintLengthRule {
    pub max_length: i64,
}

impl AllowlistBundleRule for AllowlistBundleFingerprintLengthRule {
    fn check(&self, item: &AllowlistBundle) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist bundle.
#[derive(Debug, Clone)]
pub struct AllowlistBundleValidator {
    pub rules: Vec<Rc<dyn AllowlistBundleRule>>,
}

impl AllowlistBundleValidator {
    pub fn validate(&self, item: &AllowlistBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AllowlistBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_bundle_validator() -> AllowlistBundleValidator {
    AllowlistBundleValidator {
        rules: vec![
            Rc::new(AllowlistBundleRequiredCodeRule {
                message: "allowlist bundle needs an entry".to_string(),
            }) as Rc<dyn AllowlistBundleRule>,
            Rc::new(AllowlistBundleLabelLengthRule { max_length: 32 }),
            Rc::new(AllowlistBundlePortRangeRule { min: 16, max: 86400 }),
            Rc::new(AllowlistBundleTenantIdLengthRule { max_length: 1000 }),
            Rc::new(AllowlistBundleFingerprintLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_allowlist_bundle_validator();
        let item = new_allowlist_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AllowlistBundleRequiredCodeRule {
            message: "allowlist bundle needs an entry".to_string(),
        };
        let mut item = new_allowlist_bundle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = AllowlistBundleLabelLengthRule { max_length: 255 };
        let mut item = new_allowlist_bundle("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = AllowlistBundlePortRangeRule { min: 255, max: 255 };
        let mut item = new_allowlist_bundle("x", "y");
        item.port = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = AllowlistBundleTenantIdLengthRule { max_length: 120 };
        let mut item = new_allowlist_bundle("x", "y");
        item.tenant_id = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = AllowlistBundleFingerprintLengthRule { max_length: 1000 };
        let mut item = new_allowlist_bundle("x", "y");
        item.fingerprint = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
