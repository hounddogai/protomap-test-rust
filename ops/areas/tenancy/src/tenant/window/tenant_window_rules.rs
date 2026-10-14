use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tenant window must pass. Check returns its problems, or none.
pub trait TenantWindowRule: std::fmt::Debug {
    fn check(&self, item: &TenantWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantWindowUnitLengthRule {
    pub max_length: i64,
}

impl TenantWindowRule for TenantWindowUnitLengthRule {
    fn check(&self, item: &TenantWindow) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantWindowCodePrefixRule {
    pub prefix: String,
}

impl TenantWindowRule for TenantWindowCodePrefixRule {
    fn check(&self, item: &TenantWindow) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantWindowRequiredCodeRule {
    pub message: String,
}

impl TenantWindowRule for TenantWindowRequiredCodeRule {
    fn check(&self, item: &TenantWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantWindowTimezoneLengthRule {
    pub max_length: i64,
}

impl TenantWindowRule for TenantWindowTimezoneLengthRule {
    fn check(&self, item: &TenantWindow) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantWindowCodeLengthRule {
    pub max_length: i64,
}

impl TenantWindowRule for TenantWindowCodeLengthRule {
    fn check(&self, item: &TenantWindow) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tenant window.
#[derive(Debug, Clone)]
pub struct TenantWindowValidator {
    pub rules: Vec<Rc<dyn TenantWindowRule>>,
}

impl TenantWindowValidator {
    pub fn validate(&self, item: &TenantWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TenantWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_window_validator() -> TenantWindowValidator {
    TenantWindowValidator {
        rules: vec![
            Rc::new(TenantWindowUnitLengthRule { max_length: 64 }) as Rc<dyn TenantWindowRule>,
            Rc::new(TenantWindowCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(TenantWindowRequiredCodeRule {
                message: "tenant window needs a value".to_string(),
            }),
            Rc::new(TenantWindowTimezoneLengthRule { max_length: 16 }),
            Rc::new(TenantWindowCodeLengthRule { max_length: 255 }),
        ],
    }
}
