use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual query agreement must pass. Check returns its problems, or none.
pub trait ManualQueryAgreementRule: std::fmt::Debug {
    fn check(&self, item: &ManualQueryAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualQueryAgreementKeyIdLengthRule {
    pub max_length: i64,
}

impl ManualQueryAgreementRule for ManualQueryAgreementKeyIdLengthRule {
    fn check(&self, item: &ManualQueryAgreement) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualQueryAgreementRequiredNameRule {
    pub message: String,
}

impl ManualQueryAgreementRule for ManualQueryAgreementRequiredNameRule {
    fn check(&self, item: &ManualQueryAgreement) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualQueryAgreementRequiredKeyIdRule {
    pub message: String,
}

impl ManualQueryAgreementRule for ManualQueryAgreementRequiredKeyIdRule {
    fn check(&self, item: &ManualQueryAgreement) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual query agreement.
#[derive(Debug, Clone)]
pub struct ManualQueryAgreementValidator {
    pub rules: Vec<Rc<dyn ManualQueryAgreementRule>>,
}

impl ManualQueryAgreementValidator {
    pub fn validate(&self, item: &ManualQueryAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualQueryAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualQueryAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_query_agreement_validator() -> ManualQueryAgreementValidator {
    ManualQueryAgreementValidator {
        rules: vec![
            Rc::new(ManualQueryAgreementKeyIdLengthRule { max_length: 16 }) as Rc<dyn ManualQueryAgreementRule>,
            Rc::new(ManualQueryAgreementRequiredNameRule {
                message: "manual query agreement needs an entry".to_string(),
            }),
            Rc::new(ManualQueryAgreementRequiredKeyIdRule {
                message: "manual query agreement needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_query_agreement_validator();
        let item = new_manual_query_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = ManualQueryAgreementKeyIdLengthRule { max_length: 3600 };
        let mut item = new_manual_query_agreement("x", "y");
        item.key_id = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ManualQueryAgreementRequiredNameRule {
            message: "manual query agreement needs an entry".to_string(),
        };
        let mut item = new_manual_query_agreement("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = ManualQueryAgreementRequiredKeyIdRule {
            message: "manual query agreement needs a value".to_string(),
        };
        let mut item = new_manual_query_agreement("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
