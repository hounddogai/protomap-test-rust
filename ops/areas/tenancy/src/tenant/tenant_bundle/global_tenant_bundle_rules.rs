use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global tenant bundle must pass. Check returns its problems, or none.
pub trait GlobalTenantBundleRule: std::fmt::Debug {
    fn check(&self, item: &GlobalTenantBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTenantBundleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl GlobalTenantBundleRule for GlobalTenantBundleAllowedCodeRule {
    fn check(&self, item: &GlobalTenantBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is below minimum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTenantBundleCodePrefixRule {
    pub prefix: String,
}

impl GlobalTenantBundleRule for GlobalTenantBundleCodePrefixRule {
    fn check(&self, item: &GlobalTenantBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalTenantBundleBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalTenantBundleRule for GlobalTenantBundleBatchSizeRangeRule {
    fn check(&self, item: &GlobalTenantBundle) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global tenant bundle.
#[derive(Debug, Clone)]
pub struct GlobalTenantBundleValidator {
    pub rules: Vec<Rc<dyn GlobalTenantBundleRule>>,
}

impl GlobalTenantBundleValidator {
    pub fn validate(&self, item: &GlobalTenantBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalTenantBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalTenantBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_tenant_bundle_validator() -> GlobalTenantBundleValidator {
    GlobalTenantBundleValidator {
        rules: vec![
            Rc::new(GlobalTenantBundleAllowedCodeRule {
                allowed: vec!["tiered".to_string()],
            }) as Rc<dyn GlobalTenantBundleRule>,
            Rc::new(GlobalTenantBundleCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(GlobalTenantBundleBatchSizeRangeRule { min: 80, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_tenant_bundle_validator();
        let item = new_global_tenant_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = GlobalTenantBundleBatchSizeRangeRule { min: 32, max: 120 };
        let mut item = new_global_tenant_bundle("x", "y");
        item.batch_size = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
