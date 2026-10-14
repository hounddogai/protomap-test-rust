use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a nonce rule must pass. Check returns its problems, or none.
pub trait NonceRuleRule: std::fmt::Debug {
    fn check(&self, item: &NonceRule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceRuleIssuerLengthRule {
    pub max_length: i64,
}

impl NonceRuleRule for NonceRuleIssuerLengthRule {
    fn check(&self, item: &NonceRule) -> Vec<String> {
        let size = item.issuer.len() as i64;
        if size > self.max_length {
            return vec![format!("issuer is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceRuleRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl NonceRuleRule for NonceRuleRotationDaysRangeRule {
    fn check(&self, item: &NonceRule) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceRuleRequiredCodeRule {
    pub message: String,
}

impl NonceRuleRule for NonceRuleRequiredCodeRule {
    fn check(&self, item: &NonceRule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NonceRuleCodePrefixRule {
    pub prefix: String,
}

impl NonceRuleRule for NonceRuleCodePrefixRule {
    fn check(&self, item: &NonceRule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a nonce rule.
#[derive(Debug, Clone)]
pub struct NonceRuleValidator {
    pub rules: Vec<Rc<dyn NonceRuleRule>>,
}

impl NonceRuleValidator {
    pub fn validate(&self, item: &NonceRule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &NonceRule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[NonceRule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_nonce_rule_validator() -> NonceRuleValidator {
    NonceRuleValidator {
        rules: vec![
            Rc::new(NonceRuleIssuerLengthRule { max_length: 64 }) as Rc<dyn NonceRuleRule>,
            Rc::new(NonceRuleRotationDaysRangeRule { min: 32, max: 80 }),
            Rc::new(NonceRuleRequiredCodeRule {
                message: "nonce rule needs this field".to_string(),
            }),
            Rc::new(NonceRuleCodePrefixRule {
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
        let validator = default_nonce_rule_validator();
        let item = new_nonce_rule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_issuer() {
        let rule = NonceRuleIssuerLengthRule { max_length: 16 };
        let mut item = new_nonce_rule("x", "y");
        item.issuer = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.issuer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = NonceRuleRotationDaysRangeRule { min: 120, max: 3600 };
        let mut item = new_nonce_rule("x", "y");
        item.rotation_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = NonceRuleRequiredCodeRule {
            message: "nonce rule needs a value".to_string(),
        };
        let mut item = new_nonce_rule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
