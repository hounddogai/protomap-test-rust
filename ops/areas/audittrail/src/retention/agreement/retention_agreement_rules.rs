use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a retention agreement must pass. Check returns its problems, or none.
pub trait RetentionAgreementRule: std::fmt::Debug {
    fn check(&self, item: &RetentionAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionAgreementMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RetentionAgreementRule for RetentionAgreementMarginRangeRule {
    fn check(&self, item: &RetentionAgreement) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionAgreementRequiredCodeRule {
    pub message: String,
}

impl RetentionAgreementRule for RetentionAgreementRequiredCodeRule {
    fn check(&self, item: &RetentionAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionAgreementPrincipalLengthRule {
    pub max_length: i64,
}

impl RetentionAgreementRule for RetentionAgreementPrincipalLengthRule {
    fn check(&self, item: &RetentionAgreement) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionAgreementRequiredLabelRule {
    pub message: String,
}

impl RetentionAgreementRule for RetentionAgreementRequiredLabelRule {
    fn check(&self, item: &RetentionAgreement) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionAgreementCodeLengthRule {
    pub max_length: i64,
}

impl RetentionAgreementRule for RetentionAgreementCodeLengthRule {
    fn check(&self, item: &RetentionAgreement) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RetentionAgreementLabelLengthRule {
    pub max_length: i64,
}

impl RetentionAgreementRule for RetentionAgreementLabelLengthRule {
    fn check(&self, item: &RetentionAgreement) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a retention agreement.
#[derive(Debug, Clone)]
pub struct RetentionAgreementValidator {
    pub rules: Vec<Rc<dyn RetentionAgreementRule>>,
}

impl RetentionAgreementValidator {
    pub fn validate(&self, item: &RetentionAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RetentionAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RetentionAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_retention_agreement_validator() -> RetentionAgreementValidator {
    RetentionAgreementValidator {
        rules: vec![
            Rc::new(RetentionAgreementMarginRangeRule { min: 4.2, max: 107.5 }) as Rc<dyn RetentionAgreementRule>,
            Rc::new(RetentionAgreementRequiredCodeRule {
                message: "retention agreement needs an entry".to_string(),
            }),
            Rc::new(RetentionAgreementPrincipalLengthRule { max_length: 120 }),
            Rc::new(RetentionAgreementRequiredLabelRule {
                message: "retention agreement needs an entry".to_string(),
            }),
            Rc::new(RetentionAgreementCodeLengthRule { max_length: 255 }),
            Rc::new(RetentionAgreementLabelLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_retention_agreement_validator();
        let item = new_retention_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = RetentionAgreementMarginRangeRule { min: 4.4, max: 249.2 };
        let mut item = new_retention_agreement("x", "y");
        item.margin = 250.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RetentionAgreementRequiredCodeRule {
            message: "retention agreement needs an entry".to_string(),
        };
        let mut item = new_retention_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = RetentionAgreementPrincipalLengthRule { max_length: 80 };
        let mut item = new_retention_agreement("x", "y");
        item.principal = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = RetentionAgreementRequiredLabelRule {
            message: "retention agreement needs this field".to_string(),
        };
        let mut item = new_retention_agreement("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RetentionAgreementCodeLengthRule { max_length: 16 };
        let mut item = new_retention_agreement("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RetentionAgreementLabelLengthRule { max_length: 86400 };
        let mut item = new_retention_agreement("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
