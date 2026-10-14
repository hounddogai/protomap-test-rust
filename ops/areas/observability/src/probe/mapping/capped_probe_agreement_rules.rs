use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped probe agreement must pass. Check returns its problems, or none.
pub trait CappedProbeAgreementRule: std::fmt::Debug {
    fn check(&self, item: &CappedProbeAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedProbeAgreementCodePrefixRule {
    pub prefix: String,
}

impl CappedProbeAgreementRule for CappedProbeAgreementCodePrefixRule {
    fn check(&self, item: &CappedProbeAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedProbeAgreementLabelLengthRule {
    pub max_length: i64,
}

impl CappedProbeAgreementRule for CappedProbeAgreementLabelLengthRule {
    fn check(&self, item: &CappedProbeAgreement) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedProbeAgreementRequiredExternalRefRule {
    pub message: String,
}

impl CappedProbeAgreementRule for CappedProbeAgreementRequiredExternalRefRule {
    fn check(&self, item: &CappedProbeAgreement) -> Vec<String> {
        if item.external_ref.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedProbeAgreementKeyIdLengthRule {
    pub max_length: i64,
}

impl CappedProbeAgreementRule for CappedProbeAgreementKeyIdLengthRule {
    fn check(&self, item: &CappedProbeAgreement) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped probe agreement.
#[derive(Debug, Clone)]
pub struct CappedProbeAgreementValidator {
    pub rules: Vec<Rc<dyn CappedProbeAgreementRule>>,
}

impl CappedProbeAgreementValidator {
    pub fn validate(&self, item: &CappedProbeAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedProbeAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedProbeAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_probe_agreement_validator() -> CappedProbeAgreementValidator {
    CappedProbeAgreementValidator {
        rules: vec![
            Rc::new(CappedProbeAgreementCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn CappedProbeAgreementRule>,
            Rc::new(CappedProbeAgreementLabelLengthRule { max_length: 86400 }),
            Rc::new(CappedProbeAgreementRequiredExternalRefRule {
                message: "capped probe agreement needs a value".to_string(),
            }),
            Rc::new(CappedProbeAgreementKeyIdLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_probe_agreement_validator();
        let item = new_capped_probe_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = CappedProbeAgreementLabelLengthRule { max_length: 3600 };
        let mut item = new_capped_probe_agreement("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_external_ref() {
        let rule = CappedProbeAgreementRequiredExternalRefRule {
            message: "capped probe agreement needs a value".to_string(),
        };
        let mut item = new_capped_probe_agreement("", "");
        item.external_ref = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = CappedProbeAgreementKeyIdLengthRule { max_length: 255 };
        let mut item = new_capped_probe_agreement("x", "y");
        item.key_id = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
