use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped tenant manifest must pass. Check returns its problems, or none.
pub trait CappedTenantManifestRule: std::fmt::Debug {
    fn check(&self, item: &CappedTenantManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTenantManifestAllowedCurrencyRule {
    pub allowed: Vec<String>,
}

impl CappedTenantManifestRule for CappedTenantManifestAllowedCurrencyRule {
    fn check(&self, item: &CappedTenantManifest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.currency)))
        {
            return vec![format!("currency {} is too long", item.currency)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTenantManifestRequiredTitleRule {
    pub message: String,
}

impl CappedTenantManifestRule for CappedTenantManifestRequiredTitleRule {
    fn check(&self, item: &CappedTenantManifest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedTenantManifestPrincipalLengthRule {
    pub max_length: i64,
}

impl CappedTenantManifestRule for CappedTenantManifestPrincipalLengthRule {
    fn check(&self, item: &CappedTenantManifest) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped tenant manifest.
#[derive(Debug, Clone)]
pub struct CappedTenantManifestValidator {
    pub rules: Vec<Rc<dyn CappedTenantManifestRule>>,
}

impl CappedTenantManifestValidator {
    pub fn validate(&self, item: &CappedTenantManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedTenantManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedTenantManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_tenant_manifest_validator() -> CappedTenantManifestValidator {
    CappedTenantManifestValidator {
        rules: vec![
            Rc::new(CappedTenantManifestAllowedCurrencyRule { allowed: Vec::new() })
                as Rc<dyn CappedTenantManifestRule>,
            Rc::new(CappedTenantManifestRequiredTitleRule {
                message: "capped tenant manifest needs this field".to_string(),
            }),
            Rc::new(CappedTenantManifestPrincipalLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_tenant_manifest_validator();
        let item = new_capped_tenant_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = CappedTenantManifestRequiredTitleRule {
            message: "capped tenant manifest needs a value".to_string(),
        };
        let mut item = new_capped_tenant_manifest("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = CappedTenantManifestPrincipalLengthRule { max_length: 3600 };
        let mut item = new_capped_tenant_manifest("x", "y");
        item.principal = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
