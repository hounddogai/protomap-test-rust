use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy allowlist adjustment must pass. Check returns its problems, or none.
pub trait LegacyAllowlistAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &LegacyAllowlistAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistAdjustmentRequiredScopeRule {
    pub message: String,
}

impl LegacyAllowlistAdjustmentRule for LegacyAllowlistAdjustmentRequiredScopeRule {
    fn check(&self, item: &LegacyAllowlistAdjustment) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistAdjustmentNameLengthRule {
    pub max_length: i64,
}

impl LegacyAllowlistAdjustmentRule for LegacyAllowlistAdjustmentNameLengthRule {
    fn check(&self, item: &LegacyAllowlistAdjustment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistAdjustmentCodePrefixRule {
    pub prefix: String,
}

impl LegacyAllowlistAdjustmentRule for LegacyAllowlistAdjustmentCodePrefixRule {
    fn check(&self, item: &LegacyAllowlistAdjustment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistAdjustmentAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl LegacyAllowlistAdjustmentRule for LegacyAllowlistAdjustmentAllowedCodeRule {
    fn check(&self, item: &LegacyAllowlistAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is too long", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistAdjustmentScopeLengthRule {
    pub max_length: i64,
}

impl LegacyAllowlistAdjustmentRule for LegacyAllowlistAdjustmentScopeLengthRule {
    fn check(&self, item: &LegacyAllowlistAdjustment) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy allowlist adjustment.
#[derive(Debug, Clone)]
pub struct LegacyAllowlistAdjustmentValidator {
    pub rules: Vec<Rc<dyn LegacyAllowlistAdjustmentRule>>,
}

impl LegacyAllowlistAdjustmentValidator {
    pub fn validate(&self, item: &LegacyAllowlistAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyAllowlistAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyAllowlistAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_allowlist_adjustment_validator() -> LegacyAllowlistAdjustmentValidator {
    LegacyAllowlistAdjustmentValidator {
        rules: vec![
            Rc::new(LegacyAllowlistAdjustmentRequiredScopeRule {
                message: "legacy allowlist adjustment needs a value".to_string(),
            }) as Rc<dyn LegacyAllowlistAdjustmentRule>,
            Rc::new(LegacyAllowlistAdjustmentNameLengthRule { max_length: 86400 }),
            Rc::new(LegacyAllowlistAdjustmentCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(LegacyAllowlistAdjustmentAllowedCodeRule {
                allowed: vec!["adaptive".to_string(), "pending".to_string(), "scheduled".to_string()],
            }),
            Rc::new(LegacyAllowlistAdjustmentScopeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_allowlist_adjustment_validator();
        let item = new_legacy_allowlist_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = LegacyAllowlistAdjustmentRequiredScopeRule {
            message: "legacy allowlist adjustment needs this field".to_string(),
        };
        let mut item = new_legacy_allowlist_adjustment("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = LegacyAllowlistAdjustmentNameLengthRule { max_length: 86400 };
        let mut item = new_legacy_allowlist_adjustment("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = LegacyAllowlistAdjustmentScopeLengthRule { max_length: 16 };
        let mut item = new_legacy_allowlist_adjustment("x", "y");
        item.scope = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
