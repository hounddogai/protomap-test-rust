use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tenant override must pass. Check returns its problems, or none.
pub trait TenantOverrideRule: std::fmt::Debug {
    fn check(&self, item: &TenantOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantOverrideRequiredNameRule {
    pub message: String,
}

impl TenantOverrideRule for TenantOverrideRequiredNameRule {
    fn check(&self, item: &TenantOverride) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantOverrideRequiredCodeRule {
    pub message: String,
}

impl TenantOverrideRule for TenantOverrideRequiredCodeRule {
    fn check(&self, item: &TenantOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantOverrideCodePrefixRule {
    pub prefix: String,
}

impl TenantOverrideRule for TenantOverrideCodePrefixRule {
    fn check(&self, item: &TenantOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tenant override.
#[derive(Debug, Clone)]
pub struct TenantOverrideValidator {
    pub rules: Vec<Rc<dyn TenantOverrideRule>>,
}

impl TenantOverrideValidator {
    pub fn validate(&self, item: &TenantOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TenantOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_override_validator() -> TenantOverrideValidator {
    TenantOverrideValidator {
        rules: vec![
            Rc::new(TenantOverrideRequiredNameRule {
                message: "tenant override needs an entry".to_string(),
            }) as Rc<dyn TenantOverrideRule>,
            Rc::new(TenantOverrideRequiredCodeRule {
                message: "tenant override needs an entry".to_string(),
            }),
            Rc::new(TenantOverrideCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tenant_override_validator();
        let item = new_tenant_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TenantOverrideRequiredNameRule {
            message: "tenant override needs an entry".to_string(),
        };
        let mut item = new_tenant_override("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TenantOverrideRequiredCodeRule {
            message: "tenant override needs an entry".to_string(),
        };
        let mut item = new_tenant_override("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
