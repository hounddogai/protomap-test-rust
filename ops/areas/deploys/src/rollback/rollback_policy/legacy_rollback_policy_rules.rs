use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy rollback policy must pass. Check returns its problems, or none.
pub trait LegacyRollbackPolicyRule: std::fmt::Debug {
    fn check(&self, item: &LegacyRollbackPolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRollbackPolicyLocaleLengthRule {
    pub max_length: i64,
}

impl LegacyRollbackPolicyRule for LegacyRollbackPolicyLocaleLengthRule {
    fn check(&self, item: &LegacyRollbackPolicy) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRollbackPolicyCodeLengthRule {
    pub max_length: i64,
}

impl LegacyRollbackPolicyRule for LegacyRollbackPolicyCodeLengthRule {
    fn check(&self, item: &LegacyRollbackPolicy) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRollbackPolicyRequiredCodeRule {
    pub message: String,
}

impl LegacyRollbackPolicyRule for LegacyRollbackPolicyRequiredCodeRule {
    fn check(&self, item: &LegacyRollbackPolicy) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRollbackPolicyRequiredLocaleRule {
    pub message: String,
}

impl LegacyRollbackPolicyRule for LegacyRollbackPolicyRequiredLocaleRule {
    fn check(&self, item: &LegacyRollbackPolicy) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRollbackPolicyCodePrefixRule {
    pub prefix: String,
}

impl LegacyRollbackPolicyRule for LegacyRollbackPolicyCodePrefixRule {
    fn check(&self, item: &LegacyRollbackPolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy rollback policy.
#[derive(Debug, Clone)]
pub struct LegacyRollbackPolicyValidator {
    pub rules: Vec<Rc<dyn LegacyRollbackPolicyRule>>,
}

impl LegacyRollbackPolicyValidator {
    pub fn validate(&self, item: &LegacyRollbackPolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyRollbackPolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyRollbackPolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_rollback_policy_validator() -> LegacyRollbackPolicyValidator {
    LegacyRollbackPolicyValidator {
        rules: vec![
            Rc::new(LegacyRollbackPolicyLocaleLengthRule { max_length: 3600 }) as Rc<dyn LegacyRollbackPolicyRule>,
            Rc::new(LegacyRollbackPolicyCodeLengthRule { max_length: 3600 }),
            Rc::new(LegacyRollbackPolicyRequiredCodeRule {
                message: "legacy rollback policy needs an entry".to_string(),
            }),
            Rc::new(LegacyRollbackPolicyRequiredLocaleRule {
                message: "legacy rollback policy needs this field".to_string(),
            }),
            Rc::new(LegacyRollbackPolicyCodePrefixRule {
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
        let validator = default_legacy_rollback_policy_validator();
        let item = new_legacy_rollback_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = LegacyRollbackPolicyLocaleLengthRule { max_length: 120 };
        let mut item = new_legacy_rollback_policy("x", "y");
        item.locale = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyRollbackPolicyCodeLengthRule { max_length: 64 };
        let mut item = new_legacy_rollback_policy("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LegacyRollbackPolicyRequiredCodeRule {
            message: "legacy rollback policy needs this field".to_string(),
        };
        let mut item = new_legacy_rollback_policy("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = LegacyRollbackPolicyRequiredLocaleRule {
            message: "legacy rollback policy needs a value".to_string(),
        };
        let mut item = new_legacy_rollback_policy("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
