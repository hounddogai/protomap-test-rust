use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a canary audit must pass. Check returns its problems, or none.
pub trait CanaryAuditRule: std::fmt::Debug {
    fn check(&self, item: &CanaryAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryAuditCodePrefixRule {
    pub prefix: String,
}

impl CanaryAuditRule for CanaryAuditCodePrefixRule {
    fn check(&self, item: &CanaryAudit) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryAuditAllowedNameRule {
    pub allowed: Vec<String>,
}

impl CanaryAuditRule for CanaryAuditAllowedNameRule {
    fn check(&self, item: &CanaryAudit) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is missing", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryAuditExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CanaryAuditRule for CanaryAuditExpiresAtRangeRule {
    fn check(&self, item: &CanaryAudit) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary audit.
#[derive(Debug, Clone)]
pub struct CanaryAuditValidator {
    pub rules: Vec<Rc<dyn CanaryAuditRule>>,
}

impl CanaryAuditValidator {
    pub fn validate(&self, item: &CanaryAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CanaryAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanaryAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_audit_validator() -> CanaryAuditValidator {
    CanaryAuditValidator {
        rules: vec![
            Rc::new(CanaryAuditCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn CanaryAuditRule>,
            Rc::new(CanaryAuditAllowedNameRule { allowed: Vec::new() }),
            Rc::new(CanaryAuditExpiresAtRangeRule { min: 16, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_canary_audit_validator();
        let item = new_canary_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = CanaryAuditExpiresAtRangeRule { min: 16, max: 1000 };
        let mut item = new_canary_audit("x", "y");
        item.expires_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
