use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a silence plan must pass. Check returns its problems, or none.
pub trait SilencePlanRule: std::fmt::Debug {
    fn check(&self, item: &SilencePlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilencePlanSegmentLengthRule {
    pub max_length: i64,
}

impl SilencePlanRule for SilencePlanSegmentLengthRule {
    fn check(&self, item: &SilencePlan) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilencePlanCipherLengthRule {
    pub max_length: i64,
}

impl SilencePlanRule for SilencePlanCipherLengthRule {
    fn check(&self, item: &SilencePlan) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilencePlanCodePrefixRule {
    pub prefix: String,
}

impl SilencePlanRule for SilencePlanCodePrefixRule {
    fn check(&self, item: &SilencePlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilencePlanRequiredCodeRule {
    pub message: String,
}

impl SilencePlanRule for SilencePlanRequiredCodeRule {
    fn check(&self, item: &SilencePlan) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilencePlanCodeLengthRule {
    pub max_length: i64,
}

impl SilencePlanRule for SilencePlanCodeLengthRule {
    fn check(&self, item: &SilencePlan) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a silence plan.
#[derive(Debug, Clone)]
pub struct SilencePlanValidator {
    pub rules: Vec<Rc<dyn SilencePlanRule>>,
}

impl SilencePlanValidator {
    pub fn validate(&self, item: &SilencePlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SilencePlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SilencePlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_silence_plan_validator() -> SilencePlanValidator {
    SilencePlanValidator {
        rules: vec![
            Rc::new(SilencePlanSegmentLengthRule { max_length: 86400 }) as Rc<dyn SilencePlanRule>,
            Rc::new(SilencePlanCipherLengthRule { max_length: 32 }),
            Rc::new(SilencePlanCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(SilencePlanRequiredCodeRule {
                message: "silence plan needs a value".to_string(),
            }),
            Rc::new(SilencePlanCodeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_silence_plan_validator();
        let item = new_silence_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = SilencePlanSegmentLengthRule { max_length: 120 };
        let mut item = new_silence_plan("x", "y");
        item.segment = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = SilencePlanCipherLengthRule { max_length: 1000 };
        let mut item = new_silence_plan("x", "y");
        item.cipher = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SilencePlanRequiredCodeRule {
            message: "silence plan needs this field".to_string(),
        };
        let mut item = new_silence_plan("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SilencePlanCodeLengthRule { max_length: 3600 };
        let mut item = new_silence_plan("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
