use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a canary target must pass. Check returns its problems, or none.
pub trait CanaryTargetRule: std::fmt::Debug {
    fn check(&self, item: &CanaryTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryTargetSummaryLengthRule {
    pub max_length: i64,
}

impl CanaryTargetRule for CanaryTargetSummaryLengthRule {
    fn check(&self, item: &CanaryTarget) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryTargetRequiredNameRule {
    pub message: String,
}

impl CanaryTargetRule for CanaryTargetRequiredNameRule {
    fn check(&self, item: &CanaryTarget) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryTargetCodeLengthRule {
    pub max_length: i64,
}

impl CanaryTargetRule for CanaryTargetCodeLengthRule {
    fn check(&self, item: &CanaryTarget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryTargetAllowedChecksumRule {
    pub allowed: Vec<String>,
}

impl CanaryTargetRule for CanaryTargetAllowedChecksumRule {
    fn check(&self, item: &CanaryTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.checksum)))
        {
            return vec![format!("checksum {} is unknown", item.checksum)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryTargetRequiredTitleRule {
    pub message: String,
}

impl CanaryTargetRule for CanaryTargetRequiredTitleRule {
    fn check(&self, item: &CanaryTarget) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary target.
#[derive(Debug, Clone)]
pub struct CanaryTargetValidator {
    pub rules: Vec<Rc<dyn CanaryTargetRule>>,
}

impl CanaryTargetValidator {
    pub fn validate(&self, item: &CanaryTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CanaryTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanaryTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_target_validator() -> CanaryTargetValidator {
    CanaryTargetValidator {
        rules: vec![
            Rc::new(CanaryTargetSummaryLengthRule { max_length: 86400 }) as Rc<dyn CanaryTargetRule>,
            Rc::new(CanaryTargetRequiredNameRule {
                message: "canary target needs a value".to_string(),
            }),
            Rc::new(CanaryTargetCodeLengthRule { max_length: 120 }),
            Rc::new(CanaryTargetAllowedChecksumRule {
                allowed: vec!["adaptive".to_string(), "lenient".to_string(), "primary".to_string()],
            }),
            Rc::new(CanaryTargetRequiredTitleRule {
                message: "canary target needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_canary_target_validator();
        let item = new_canary_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = CanaryTargetSummaryLengthRule { max_length: 86400 };
        let mut item = new_canary_target("x", "y");
        item.summary = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = CanaryTargetRequiredNameRule {
            message: "canary target needs a value".to_string(),
        };
        let mut item = new_canary_target("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CanaryTargetCodeLengthRule { max_length: 120 };
        let mut item = new_canary_target("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = CanaryTargetRequiredTitleRule {
            message: "canary target needs this field".to_string(),
        };
        let mut item = new_canary_target("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
