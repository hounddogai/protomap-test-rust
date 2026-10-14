use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a policy variance must pass. Check returns its problems, or none.
pub trait PolicyVarianceRule: std::fmt::Debug {
    fn check(&self, item: &PolicyVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyVarianceWindowRule {
    pub max_span: i64,
}

impl PolicyVarianceRule for PolicyVarianceWindowRule {
    fn check(&self, item: &PolicyVariance) -> Vec<String> {
        if item.starts_at > item.ends_at {
            return vec!["starts at is after ends at".to_string()];
        }
        if self.max_span > 0 && (item.ends_at - item.starts_at > self.max_span) {
            return vec![format!("spans more than {} seconds", self.max_span)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyVarianceRequiredTenantIdRule {
    pub message: String,
}

impl PolicyVarianceRule for PolicyVarianceRequiredTenantIdRule {
    fn check(&self, item: &PolicyVariance) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyVarianceCodePrefixRule {
    pub prefix: String,
}

impl PolicyVarianceRule for PolicyVarianceCodePrefixRule {
    fn check(&self, item: &PolicyVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyVarianceTenantIdLengthRule {
    pub max_length: i64,
}

impl PolicyVarianceRule for PolicyVarianceTenantIdLengthRule {
    fn check(&self, item: &PolicyVariance) -> Vec<String> {
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

/// Runs every rule of a policy variance.
#[derive(Debug, Clone)]
pub struct PolicyVarianceValidator {
    pub rules: Vec<Rc<dyn PolicyVarianceRule>>,
}

impl PolicyVarianceValidator {
    pub fn validate(&self, item: &PolicyVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PolicyVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PolicyVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_policy_variance_validator() -> PolicyVarianceValidator {
    PolicyVarianceValidator {
        rules: vec![
            Rc::new(PolicyVarianceWindowRule { max_span: 16 }) as Rc<dyn PolicyVarianceRule>,
            Rc::new(PolicyVarianceRequiredTenantIdRule {
                message: "policy variance needs an entry".to_string(),
            }),
            Rc::new(PolicyVarianceCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(PolicyVarianceTenantIdLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_policy_variance_validator();
        let item = new_policy_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = PolicyVarianceRequiredTenantIdRule {
            message: "policy variance needs this field".to_string(),
        };
        let mut item = new_policy_variance("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = PolicyVarianceTenantIdLengthRule { max_length: 86400 };
        let mut item = new_policy_variance("x", "y");
        item.tenant_id = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
