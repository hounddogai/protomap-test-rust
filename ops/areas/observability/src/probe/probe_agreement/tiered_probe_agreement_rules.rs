use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered probe agreement must pass. Check returns its problems, or none.
pub trait TieredProbeAgreementRule: std::fmt::Debug {
    fn check(&self, item: &TieredProbeAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredProbeAgreementRequiredNameRule {
    pub message: String,
}

impl TieredProbeAgreementRule for TieredProbeAgreementRequiredNameRule {
    fn check(&self, item: &TieredProbeAgreement) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredProbeAgreementNameLengthRule {
    pub max_length: i64,
}

impl TieredProbeAgreementRule for TieredProbeAgreementNameLengthRule {
    fn check(&self, item: &TieredProbeAgreement) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredProbeAgreementNotesLengthRule {
    pub max_length: i64,
}

impl TieredProbeAgreementRule for TieredProbeAgreementNotesLengthRule {
    fn check(&self, item: &TieredProbeAgreement) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered probe agreement.
#[derive(Debug, Clone)]
pub struct TieredProbeAgreementValidator {
    pub rules: Vec<Rc<dyn TieredProbeAgreementRule>>,
}

impl TieredProbeAgreementValidator {
    pub fn validate(&self, item: &TieredProbeAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredProbeAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredProbeAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_probe_agreement_validator() -> TieredProbeAgreementValidator {
    TieredProbeAgreementValidator {
        rules: vec![
            Rc::new(TieredProbeAgreementRequiredNameRule {
                message: "tiered probe agreement needs a value".to_string(),
            }) as Rc<dyn TieredProbeAgreementRule>,
            Rc::new(TieredProbeAgreementNameLengthRule { max_length: 32 }),
            Rc::new(TieredProbeAgreementNotesLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_probe_agreement_validator();
        let item = new_tiered_probe_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TieredProbeAgreementRequiredNameRule {
            message: "tiered probe agreement needs this field".to_string(),
        };
        let mut item = new_tiered_probe_agreement("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TieredProbeAgreementNameLengthRule { max_length: 3600 };
        let mut item = new_tiered_probe_agreement("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = TieredProbeAgreementNotesLengthRule { max_length: 80 };
        let mut item = new_tiered_probe_agreement("x", "y");
        item.notes = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
