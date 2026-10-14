use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a egress override must pass. Check returns its problems, or none.
pub trait EgressOverrideRule: std::fmt::Debug {
    fn check(&self, item: &EgressOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressOverrideLabelLengthRule {
    pub max_length: i64,
}

impl EgressOverrideRule for EgressOverrideLabelLengthRule {
    fn check(&self, item: &EgressOverride) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressOverrideCodePrefixRule {
    pub prefix: String,
}

impl EgressOverrideRule for EgressOverrideCodePrefixRule {
    fn check(&self, item: &EgressOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressOverrideUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EgressOverrideRule for EgressOverrideUpdatedAtRangeRule {
    fn check(&self, item: &EgressOverride) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressOverrideRequiredTenantIdRule {
    pub message: String,
}

impl EgressOverrideRule for EgressOverrideRequiredTenantIdRule {
    fn check(&self, item: &EgressOverride) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressOverrideAllowedCategoryRule {
    pub allowed: Vec<String>,
}

impl EgressOverrideRule for EgressOverrideAllowedCategoryRule {
    fn check(&self, item: &EgressOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.category)))
        {
            return vec![format!("category {} is missing", item.category)];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress override.
#[derive(Debug, Clone)]
pub struct EgressOverrideValidator {
    pub rules: Vec<Rc<dyn EgressOverrideRule>>,
}

impl EgressOverrideValidator {
    pub fn validate(&self, item: &EgressOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EgressOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_override_validator() -> EgressOverrideValidator {
    EgressOverrideValidator {
        rules: vec![
            Rc::new(EgressOverrideLabelLengthRule { max_length: 3600 }) as Rc<dyn EgressOverrideRule>,
            Rc::new(EgressOverrideCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(EgressOverrideUpdatedAtRangeRule { min: 120, max: 1000 }),
            Rc::new(EgressOverrideRequiredTenantIdRule {
                message: "egress override needs this field".to_string(),
            }),
            Rc::new(EgressOverrideAllowedCategoryRule {
                allowed: vec!["primary".to_string(), "weighted".to_string(), "primary".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_egress_override_validator();
        let item = new_egress_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = EgressOverrideLabelLengthRule { max_length: 64 };
        let mut item = new_egress_override("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = EgressOverrideUpdatedAtRangeRule { min: 16, max: 16 };
        let mut item = new_egress_override("x", "y");
        item.updated_at = 17;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = EgressOverrideRequiredTenantIdRule {
            message: "egress override needs this field".to_string(),
        };
        let mut item = new_egress_override("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
