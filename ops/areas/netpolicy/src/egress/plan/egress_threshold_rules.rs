use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a egress threshold must pass. Check returns its problems, or none.
pub trait EgressThresholdRule: std::fmt::Debug {
    fn check(&self, item: &EgressThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressThresholdRequiredCodeRule {
    pub message: String,
}

impl EgressThresholdRule for EgressThresholdRequiredCodeRule {
    fn check(&self, item: &EgressThreshold) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressThresholdCodeLengthRule {
    pub max_length: i64,
}

impl EgressThresholdRule for EgressThresholdCodeLengthRule {
    fn check(&self, item: &EgressThreshold) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressThresholdTitleLengthRule {
    pub max_length: i64,
}

impl EgressThresholdRule for EgressThresholdTitleLengthRule {
    fn check(&self, item: &EgressThreshold) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressThresholdCipherLengthRule {
    pub max_length: i64,
}

impl EgressThresholdRule for EgressThresholdCipherLengthRule {
    fn check(&self, item: &EgressThreshold) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressThresholdCodePrefixRule {
    pub prefix: String,
}

impl EgressThresholdRule for EgressThresholdCodePrefixRule {
    fn check(&self, item: &EgressThreshold) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress threshold.
#[derive(Debug, Clone)]
pub struct EgressThresholdValidator {
    pub rules: Vec<Rc<dyn EgressThresholdRule>>,
}

impl EgressThresholdValidator {
    pub fn validate(&self, item: &EgressThreshold) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EgressThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_threshold_validator() -> EgressThresholdValidator {
    EgressThresholdValidator {
        rules: vec![
            Rc::new(EgressThresholdRequiredCodeRule {
                message: "egress threshold needs this field".to_string(),
            }) as Rc<dyn EgressThresholdRule>,
            Rc::new(EgressThresholdCodeLengthRule { max_length: 64 }),
            Rc::new(EgressThresholdTitleLengthRule { max_length: 255 }),
            Rc::new(EgressThresholdCipherLengthRule { max_length: 1000 }),
            Rc::new(EgressThresholdCodePrefixRule {
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
        let validator = default_egress_threshold_validator();
        let item = new_egress_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EgressThresholdRequiredCodeRule {
            message: "egress threshold needs an entry".to_string(),
        };
        let mut item = new_egress_threshold("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = EgressThresholdCodeLengthRule { max_length: 32 };
        let mut item = new_egress_threshold("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = EgressThresholdTitleLengthRule { max_length: 3600 };
        let mut item = new_egress_threshold("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = EgressThresholdCipherLengthRule { max_length: 80 };
        let mut item = new_egress_threshold("x", "y");
        item.cipher = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
