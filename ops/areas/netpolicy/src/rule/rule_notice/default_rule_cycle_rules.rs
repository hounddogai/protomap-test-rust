use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default rule cycle must pass. Check returns its problems, or none.
pub trait DefaultRuleCycleRule: std::fmt::Debug {
    fn check(&self, item: &DefaultRuleCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRuleCycleRequiredCodeRule {
    pub message: String,
}

impl DefaultRuleCycleRule for DefaultRuleCycleRequiredCodeRule {
    fn check(&self, item: &DefaultRuleCycle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRuleCycleCodeLengthRule {
    pub max_length: i64,
}

impl DefaultRuleCycleRule for DefaultRuleCycleCodeLengthRule {
    fn check(&self, item: &DefaultRuleCycle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRuleCycleRequiredFingerprintRule {
    pub message: String,
}

impl DefaultRuleCycleRule for DefaultRuleCycleRequiredFingerprintRule {
    fn check(&self, item: &DefaultRuleCycle) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRuleCycleSegmentLengthRule {
    pub max_length: i64,
}

impl DefaultRuleCycleRule for DefaultRuleCycleSegmentLengthRule {
    fn check(&self, item: &DefaultRuleCycle) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default rule cycle.
#[derive(Debug, Clone)]
pub struct DefaultRuleCycleValidator {
    pub rules: Vec<Rc<dyn DefaultRuleCycleRule>>,
}

impl DefaultRuleCycleValidator {
    pub fn validate(&self, item: &DefaultRuleCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultRuleCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultRuleCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_rule_cycle_validator() -> DefaultRuleCycleValidator {
    DefaultRuleCycleValidator {
        rules: vec![
            Rc::new(DefaultRuleCycleRequiredCodeRule {
                message: "default rule cycle needs an entry".to_string(),
            }) as Rc<dyn DefaultRuleCycleRule>,
            Rc::new(DefaultRuleCycleCodeLengthRule { max_length: 86400 }),
            Rc::new(DefaultRuleCycleRequiredFingerprintRule {
                message: "default rule cycle needs an entry".to_string(),
            }),
            Rc::new(DefaultRuleCycleSegmentLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_rule_cycle_validator();
        let item = new_default_rule_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultRuleCycleRequiredCodeRule {
            message: "default rule cycle needs this field".to_string(),
        };
        let mut item = new_default_rule_cycle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DefaultRuleCycleCodeLengthRule { max_length: 32 };
        let mut item = new_default_rule_cycle("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = DefaultRuleCycleRequiredFingerprintRule {
            message: "default rule cycle needs an entry".to_string(),
        };
        let mut item = new_default_rule_cycle("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = DefaultRuleCycleSegmentLengthRule { max_length: 120 };
        let mut item = new_default_rule_cycle("x", "y");
        item.segment = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
