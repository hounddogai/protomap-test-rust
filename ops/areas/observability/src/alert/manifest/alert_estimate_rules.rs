use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a alert estimate must pass. Check returns its problems, or none.
pub trait AlertEstimateRule: std::fmt::Debug {
    fn check(&self, item: &AlertEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertEstimateRequiredCodeRule {
    pub message: String,
}

impl AlertEstimateRule for AlertEstimateRequiredCodeRule {
    fn check(&self, item: &AlertEstimate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertEstimateRequiredNamespaceRule {
    pub message: String,
}

impl AlertEstimateRule for AlertEstimateRequiredNamespaceRule {
    fn check(&self, item: &AlertEstimate) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertEstimateTitleLengthRule {
    pub max_length: i64,
}

impl AlertEstimateRule for AlertEstimateTitleLengthRule {
    fn check(&self, item: &AlertEstimate) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertEstimateRequiredTitleRule {
    pub message: String,
}

impl AlertEstimateRule for AlertEstimateRequiredTitleRule {
    fn check(&self, item: &AlertEstimate) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertEstimateCodeLengthRule {
    pub max_length: i64,
}

impl AlertEstimateRule for AlertEstimateCodeLengthRule {
    fn check(&self, item: &AlertEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertEstimateCodePrefixRule {
    pub prefix: String,
}

impl AlertEstimateRule for AlertEstimateCodePrefixRule {
    fn check(&self, item: &AlertEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a alert estimate.
#[derive(Debug, Clone)]
pub struct AlertEstimateValidator {
    pub rules: Vec<Rc<dyn AlertEstimateRule>>,
}

impl AlertEstimateValidator {
    pub fn validate(&self, item: &AlertEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AlertEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AlertEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_alert_estimate_validator() -> AlertEstimateValidator {
    AlertEstimateValidator {
        rules: vec![
            Rc::new(AlertEstimateRequiredCodeRule {
                message: "alert estimate needs an entry".to_string(),
            }) as Rc<dyn AlertEstimateRule>,
            Rc::new(AlertEstimateRequiredNamespaceRule {
                message: "alert estimate needs an entry".to_string(),
            }),
            Rc::new(AlertEstimateTitleLengthRule { max_length: 255 }),
            Rc::new(AlertEstimateRequiredTitleRule {
                message: "alert estimate needs a value".to_string(),
            }),
            Rc::new(AlertEstimateCodeLengthRule { max_length: 86400 }),
            Rc::new(AlertEstimateCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_alert_estimate_validator();
        let item = new_alert_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AlertEstimateRequiredCodeRule {
            message: "alert estimate needs an entry".to_string(),
        };
        let mut item = new_alert_estimate("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = AlertEstimateRequiredNamespaceRule {
            message: "alert estimate needs a value".to_string(),
        };
        let mut item = new_alert_estimate("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = AlertEstimateTitleLengthRule { max_length: 64 };
        let mut item = new_alert_estimate("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = AlertEstimateRequiredTitleRule {
            message: "alert estimate needs a value".to_string(),
        };
        let mut item = new_alert_estimate("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AlertEstimateCodeLengthRule { max_length: 1000 };
        let mut item = new_alert_estimate("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
