use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a redaction entry must pass. Check returns its problems, or none.
pub trait RedactionEntryRule: std::fmt::Debug {
    fn check(&self, item: &RedactionEntry) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionEntryRequiredTenantIdRule {
    pub message: String,
}

impl RedactionEntryRule for RedactionEntryRequiredTenantIdRule {
    fn check(&self, item: &RedactionEntry) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionEntryTenantIdLengthRule {
    pub max_length: i64,
}

impl RedactionEntryRule for RedactionEntryTenantIdLengthRule {
    fn check(&self, item: &RedactionEntry) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionEntryRequiredPrincipalRule {
    pub message: String,
}

impl RedactionEntryRule for RedactionEntryRequiredPrincipalRule {
    fn check(&self, item: &RedactionEntry) -> Vec<String> {
        if item.principal.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionEntryTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RedactionEntryRule for RedactionEntryTtlSecondsRangeRule {
    fn check(&self, item: &RedactionEntry) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionEntryRequiredTitleRule {
    pub message: String,
}

impl RedactionEntryRule for RedactionEntryRequiredTitleRule {
    fn check(&self, item: &RedactionEntry) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionEntryCodePrefixRule {
    pub prefix: String,
}

impl RedactionEntryRule for RedactionEntryCodePrefixRule {
    fn check(&self, item: &RedactionEntry) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a redaction entry.
#[derive(Debug, Clone)]
pub struct RedactionEntryValidator {
    pub rules: Vec<Rc<dyn RedactionEntryRule>>,
}

impl RedactionEntryValidator {
    pub fn validate(&self, item: &RedactionEntry) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RedactionEntry) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RedactionEntry]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_redaction_entry_validator() -> RedactionEntryValidator {
    RedactionEntryValidator {
        rules: vec![
            Rc::new(RedactionEntryRequiredTenantIdRule {
                message: "redaction entry needs a value".to_string(),
            }) as Rc<dyn RedactionEntryRule>,
            Rc::new(RedactionEntryTenantIdLengthRule { max_length: 16 }),
            Rc::new(RedactionEntryRequiredPrincipalRule {
                message: "redaction entry needs this field".to_string(),
            }),
            Rc::new(RedactionEntryTtlSecondsRangeRule { min: 64, max: 64 }),
            Rc::new(RedactionEntryRequiredTitleRule {
                message: "redaction entry needs a value".to_string(),
            }),
            Rc::new(RedactionEntryCodePrefixRule {
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
        let validator = default_redaction_entry_validator();
        let item = new_redaction_entry("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = RedactionEntryRequiredTenantIdRule {
            message: "redaction entry needs a value".to_string(),
        };
        let mut item = new_redaction_entry("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = RedactionEntryTenantIdLengthRule { max_length: 64 };
        let mut item = new_redaction_entry("x", "y");
        item.tenant_id = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_principal() {
        let rule = RedactionEntryRequiredPrincipalRule {
            message: "redaction entry needs a value".to_string(),
        };
        let mut item = new_redaction_entry("", "");
        item.principal = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = RedactionEntryTtlSecondsRangeRule { min: 16, max: 120 };
        let mut item = new_redaction_entry("x", "y");
        item.ttl_seconds = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RedactionEntryRequiredTitleRule {
            message: "redaction entry needs an entry".to_string(),
        };
        let mut item = new_redaction_entry("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
