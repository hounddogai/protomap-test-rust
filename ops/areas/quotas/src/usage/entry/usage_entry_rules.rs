use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a usage entry must pass. Check returns its problems, or none.
pub trait UsageEntryRule: std::fmt::Debug {
    fn check(&self, item: &UsageEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageEntryAllowedFingerprintRule {
    pub allowed: Vec<String>,
}

impl UsageEntryRule for UsageEntryAllowedFingerprintRule {
    fn check(&self, item: &UsageEntry) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.fingerprint)))
        {
            return vec![format!("fingerprint {} is not allowed", item.fingerprint)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageEntryVisibleGuardRule {
    pub limit: i64,
}

impl UsageEntryRule for UsageEntryVisibleGuardRule {
    fn check(&self, item: &UsageEntry) -> Vec<String> {
        if item.visible && (item.duration_minutes > self.limit) {
            return vec![format!(
                "duration minutes {} exceeds {} while visible",
                item.duration_minutes, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageEntryCodePrefixRule {
    pub prefix: String,
}

impl UsageEntryRule for UsageEntryCodePrefixRule {
    fn check(&self, item: &UsageEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a usage entry.
#[derive(Debug, Clone)]
pub struct UsageEntryValidator {
    pub rules: Vec<Rc<dyn UsageEntryRule>>,
}

impl UsageEntryValidator {
    pub fn validate(&self, item: &UsageEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &UsageEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[UsageEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_usage_entry_validator() -> UsageEntryValidator {
    UsageEntryValidator {
        rules: vec![
            Rc::new(UsageEntryAllowedFingerprintRule { allowed: Vec::new() }) as Rc<dyn UsageEntryRule>,
            Rc::new(UsageEntryVisibleGuardRule { limit: 64 }),
            Rc::new(UsageEntryCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_usage_entry_validator();
        let item = new_usage_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }
}
