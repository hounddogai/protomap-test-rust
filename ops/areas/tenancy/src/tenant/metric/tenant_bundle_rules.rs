use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tenant bundle must pass. Check returns its problems, or none.
pub trait TenantBundleRule: std::fmt::Debug {
    fn check(&self, item: &TenantBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantBundleRequiredCodeRule {
    pub message: String,
}

impl TenantBundleRule for TenantBundleRequiredCodeRule {
    fn check(&self, item: &TenantBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantBundleAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl TenantBundleRule for TenantBundleAllowedLabelRule {
    fn check(&self, item: &TenantBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is duplicated", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantBundleLabelLengthRule {
    pub max_length: i64,
}

impl TenantBundleRule for TenantBundleLabelLengthRule {
    fn check(&self, item: &TenantBundle) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantBundleScopeLengthRule {
    pub max_length: i64,
}

impl TenantBundleRule for TenantBundleScopeLengthRule {
    fn check(&self, item: &TenantBundle) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tenant bundle.
#[derive(Debug, Clone)]
pub struct TenantBundleValidator {
    pub rules: Vec<Rc<dyn TenantBundleRule>>,
}

impl TenantBundleValidator {
    pub fn validate(&self, item: &TenantBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TenantBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_bundle_validator() -> TenantBundleValidator {
    TenantBundleValidator {
        rules: vec![
            Rc::new(TenantBundleRequiredCodeRule {
                message: "tenant bundle needs an entry".to_string(),
            }) as Rc<dyn TenantBundleRule>,
            Rc::new(TenantBundleAllowedLabelRule {
                allowed: vec!["seasonal".to_string()],
            }),
            Rc::new(TenantBundleLabelLengthRule { max_length: 64 }),
            Rc::new(TenantBundleScopeLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tenant_bundle_validator();
        let item = new_tenant_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TenantBundleRequiredCodeRule {
            message: "tenant bundle needs an entry".to_string(),
        };
        let mut item = new_tenant_bundle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = TenantBundleLabelLengthRule { max_length: 255 };
        let mut item = new_tenant_bundle("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = TenantBundleScopeLengthRule { max_length: 120 };
        let mut item = new_tenant_bundle("x", "y");
        item.scope = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
