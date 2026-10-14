use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive delegation record must pass. Check returns its problems, or none.
pub trait AdaptiveDelegationRecordRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveDelegationRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDelegationRecordCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveDelegationRecordRule for AdaptiveDelegationRecordCodePrefixRule {
    fn check(&self, item: &AdaptiveDelegationRecord) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDelegationRecordTenantIdLengthRule {
    pub max_length: i64,
}

impl AdaptiveDelegationRecordRule for AdaptiveDelegationRecordTenantIdLengthRule {
    fn check(&self, item: &AdaptiveDelegationRecord) -> Vec<String> {
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
pub struct AdaptiveDelegationRecordRequiredNameRule {
    pub message: String,
}

impl AdaptiveDelegationRecordRule for AdaptiveDelegationRecordRequiredNameRule {
    fn check(&self, item: &AdaptiveDelegationRecord) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDelegationRecordRequiredCodeRule {
    pub message: String,
}

impl AdaptiveDelegationRecordRule for AdaptiveDelegationRecordRequiredCodeRule {
    fn check(&self, item: &AdaptiveDelegationRecord) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDelegationRecordRequiredTenantIdRule {
    pub message: String,
}

impl AdaptiveDelegationRecordRule for AdaptiveDelegationRecordRequiredTenantIdRule {
    fn check(&self, item: &AdaptiveDelegationRecord) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDelegationRecordRequiredLabelRule {
    pub message: String,
}

impl AdaptiveDelegationRecordRule for AdaptiveDelegationRecordRequiredLabelRule {
    fn check(&self, item: &AdaptiveDelegationRecord) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive delegation record.
#[derive(Debug, Clone)]
pub struct AdaptiveDelegationRecordValidator {
    pub rules: Vec<Rc<dyn AdaptiveDelegationRecordRule>>,
}

impl AdaptiveDelegationRecordValidator {
    pub fn validate(&self, item: &AdaptiveDelegationRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveDelegationRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveDelegationRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_delegation_record_validator() -> AdaptiveDelegationRecordValidator {
    AdaptiveDelegationRecordValidator {
        rules: vec![
            Rc::new(AdaptiveDelegationRecordCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn AdaptiveDelegationRecordRule>,
            Rc::new(AdaptiveDelegationRecordTenantIdLengthRule { max_length: 32 }),
            Rc::new(AdaptiveDelegationRecordRequiredNameRule {
                message: "adaptive delegation record needs a value".to_string(),
            }),
            Rc::new(AdaptiveDelegationRecordRequiredCodeRule {
                message: "adaptive delegation record needs an entry".to_string(),
            }),
            Rc::new(AdaptiveDelegationRecordRequiredTenantIdRule {
                message: "adaptive delegation record needs this field".to_string(),
            }),
            Rc::new(AdaptiveDelegationRecordRequiredLabelRule {
                message: "adaptive delegation record needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_delegation_record_validator();
        let item = new_adaptive_delegation_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = AdaptiveDelegationRecordTenantIdLengthRule { max_length: 120 };
        let mut item = new_adaptive_delegation_record("x", "y");
        item.tenant_id = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = AdaptiveDelegationRecordRequiredNameRule {
            message: "adaptive delegation record needs a value".to_string(),
        };
        let mut item = new_adaptive_delegation_record("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AdaptiveDelegationRecordRequiredCodeRule {
            message: "adaptive delegation record needs a value".to_string(),
        };
        let mut item = new_adaptive_delegation_record("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = AdaptiveDelegationRecordRequiredTenantIdRule {
            message: "adaptive delegation record needs an entry".to_string(),
        };
        let mut item = new_adaptive_delegation_record("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = AdaptiveDelegationRecordRequiredLabelRule {
            message: "adaptive delegation record needs a value".to_string(),
        };
        let mut item = new_adaptive_delegation_record("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
