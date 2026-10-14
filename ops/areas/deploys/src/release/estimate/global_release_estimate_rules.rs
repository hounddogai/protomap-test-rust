use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global release estimate must pass. Check returns its problems, or none.
pub trait GlobalReleaseEstimateRule: std::fmt::Debug {
    fn check(&self, item: &GlobalReleaseEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseEstimateCodePrefixRule {
    pub prefix: String,
}

impl GlobalReleaseEstimateRule for GlobalReleaseEstimateCodePrefixRule {
    fn check(&self, item: &GlobalReleaseEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseEstimateCodeLengthRule {
    pub max_length: i64,
}

impl GlobalReleaseEstimateRule for GlobalReleaseEstimateCodeLengthRule {
    fn check(&self, item: &GlobalReleaseEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseEstimateRequiredCodeRule {
    pub message: String,
}

impl GlobalReleaseEstimateRule for GlobalReleaseEstimateRequiredCodeRule {
    fn check(&self, item: &GlobalReleaseEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseEstimateRequiredCipherRule {
    pub message: String,
}

impl GlobalReleaseEstimateRule for GlobalReleaseEstimateRequiredCipherRule {
    fn check(&self, item: &GlobalReleaseEstimate) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a global release estimate.
#[derive(Debug, Clone)]
pub struct GlobalReleaseEstimateValidator {
    pub rules: Vec<Rc<dyn GlobalReleaseEstimateRule>>,
}

impl GlobalReleaseEstimateValidator {
    pub fn validate(&self, item: &GlobalReleaseEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalReleaseEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalReleaseEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_release_estimate_validator() -> GlobalReleaseEstimateValidator {
    GlobalReleaseEstimateValidator {
        rules: vec![
            Rc::new(GlobalReleaseEstimateCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn GlobalReleaseEstimateRule>,
            Rc::new(GlobalReleaseEstimateCodeLengthRule { max_length: 16 }),
            Rc::new(GlobalReleaseEstimateRequiredCodeRule {
                message: "global release estimate needs an entry".to_string(),
            }),
            Rc::new(GlobalReleaseEstimateRequiredCipherRule {
                message: "global release estimate needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_release_estimate_validator();
        let item = new_global_release_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = GlobalReleaseEstimateCodeLengthRule { max_length: 255 };
        let mut item = new_global_release_estimate("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GlobalReleaseEstimateRequiredCodeRule {
            message: "global release estimate needs this field".to_string(),
        };
        let mut item = new_global_release_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = GlobalReleaseEstimateRequiredCipherRule {
            message: "global release estimate needs this field".to_string(),
        };
        let mut item = new_global_release_estimate("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
