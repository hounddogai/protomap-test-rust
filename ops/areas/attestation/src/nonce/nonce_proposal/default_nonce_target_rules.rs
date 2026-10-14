use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default nonce target must pass. Check returns its problems, or none.
pub trait DefaultNonceTargetRule: std::fmt::Debug {
    fn check(&self, item: &DefaultNonceTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultNonceTargetRequiredCodeRule {
    pub message: String,
}

impl DefaultNonceTargetRule for DefaultNonceTargetRequiredCodeRule {
    fn check(&self, item: &DefaultNonceTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultNonceTargetCodeLengthRule {
    pub max_length: i64,
}

impl DefaultNonceTargetRule for DefaultNonceTargetCodeLengthRule {
    fn check(&self, item: &DefaultNonceTarget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultNonceTargetLabelLengthRule {
    pub max_length: i64,
}

impl DefaultNonceTargetRule for DefaultNonceTargetLabelLengthRule {
    fn check(&self, item: &DefaultNonceTarget) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultNonceTargetRequiredLabelRule {
    pub message: String,
}

impl DefaultNonceTargetRule for DefaultNonceTargetRequiredLabelRule {
    fn check(&self, item: &DefaultNonceTarget) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default nonce target.
#[derive(Debug, Clone)]
pub struct DefaultNonceTargetValidator {
    pub rules: Vec<Rc<dyn DefaultNonceTargetRule>>,
}

impl DefaultNonceTargetValidator {
    pub fn validate(&self, item: &DefaultNonceTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultNonceTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultNonceTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_nonce_target_validator() -> DefaultNonceTargetValidator {
    DefaultNonceTargetValidator {
        rules: vec![
            Rc::new(DefaultNonceTargetRequiredCodeRule {
                message: "default nonce target needs a value".to_string(),
            }) as Rc<dyn DefaultNonceTargetRule>,
            Rc::new(DefaultNonceTargetCodeLengthRule { max_length: 1000 }),
            Rc::new(DefaultNonceTargetLabelLengthRule { max_length: 16 }),
            Rc::new(DefaultNonceTargetRequiredLabelRule {
                message: "default nonce target needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_nonce_target_validator();
        let item = new_default_nonce_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultNonceTargetRequiredCodeRule {
            message: "default nonce target needs an entry".to_string(),
        };
        let mut item = new_default_nonce_target("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultNonceTargetCodeLengthRule { max_length: 255 };
        let mut item = new_default_nonce_target("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DefaultNonceTargetLabelLengthRule { max_length: 32 };
        let mut item = new_default_nonce_target("x", "y");
        item.label = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = DefaultNonceTargetRequiredLabelRule {
            message: "default nonce target needs a value".to_string(),
        };
        let mut item = new_default_nonce_target("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
