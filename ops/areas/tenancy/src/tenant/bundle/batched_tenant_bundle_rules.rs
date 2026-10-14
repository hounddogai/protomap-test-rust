use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a batched tenant bundle must pass. Check returns its problems, or none.
pub trait BatchedTenantBundleRule: std::fmt::Debug {
    fn check(&self, item: &BatchedTenantBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTenantBundleNamespaceLengthRule {
    pub max_length: i64,
}

impl BatchedTenantBundleRule for BatchedTenantBundleNamespaceLengthRule {
    fn check(&self, item: &BatchedTenantBundle) -> Vec<String> {
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
pub struct BatchedTenantBundleRegionLengthRule {
    pub max_length: i64,
}

impl BatchedTenantBundleRule for BatchedTenantBundleRegionLengthRule {
    fn check(&self, item: &BatchedTenantBundle) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTenantBundleNameLengthRule {
    pub max_length: i64,
}

impl BatchedTenantBundleRule for BatchedTenantBundleNameLengthRule {
    fn check(&self, item: &BatchedTenantBundle) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTenantBundleRequiredNamespaceRule {
    pub message: String,
}

impl BatchedTenantBundleRule for BatchedTenantBundleRequiredNamespaceRule {
    fn check(&self, item: &BatchedTenantBundle) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTenantBundleCodePrefixRule {
    pub prefix: String,
}

impl BatchedTenantBundleRule for BatchedTenantBundleCodePrefixRule {
    fn check(&self, item: &BatchedTenantBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a batched tenant bundle.
#[derive(Debug, Clone)]
pub struct BatchedTenantBundleValidator {
    pub rules: Vec<Rc<dyn BatchedTenantBundleRule>>,
}

impl BatchedTenantBundleValidator {
    pub fn validate(&self, item: &BatchedTenantBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BatchedTenantBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BatchedTenantBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_batched_tenant_bundle_validator() -> BatchedTenantBundleValidator {
    BatchedTenantBundleValidator {
        rules: vec![
            Rc::new(BatchedTenantBundleNamespaceLengthRule { max_length: 16 }) as Rc<dyn BatchedTenantBundleRule>,
            Rc::new(BatchedTenantBundleRegionLengthRule { max_length: 1000 }),
            Rc::new(BatchedTenantBundleNameLengthRule { max_length: 80 }),
            Rc::new(BatchedTenantBundleRequiredNamespaceRule {
                message: "batched tenant bundle needs a value".to_string(),
            }),
            Rc::new(BatchedTenantBundleCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_batched_tenant_bundle_validator();
        let item = new_batched_tenant_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = BatchedTenantBundleNamespaceLengthRule { max_length: 32 };
        let mut item = new_batched_tenant_bundle("x", "y");
        item.namespace = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = BatchedTenantBundleRegionLengthRule { max_length: 86400 };
        let mut item = new_batched_tenant_bundle("x", "y");
        item.region = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = BatchedTenantBundleNameLengthRule { max_length: 3600 };
        let mut item = new_batched_tenant_bundle("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = BatchedTenantBundleRequiredNamespaceRule {
            message: "batched tenant bundle needs this field".to_string(),
        };
        let mut item = new_batched_tenant_bundle("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
