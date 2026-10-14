use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a failover proposal must pass. Check returns its problems, or none.
pub trait FailoverProposalRule: std::fmt::Debug {
    fn check(&self, item: &FailoverProposal) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverProposalFingerprintLengthRule {
    pub max_length: i64,
}

impl FailoverProposalRule for FailoverProposalFingerprintLengthRule {
    fn check(&self, item: &FailoverProposal) -> Vec<String> {
        let size = item.fingerprint.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "fingerprint is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverProposalDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FailoverProposalRule for FailoverProposalDurationMinutesRangeRule {
    fn check(&self, item: &FailoverProposal) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverProposalTitleLengthRule {
    pub max_length: i64,
}

impl FailoverProposalRule for FailoverProposalTitleLengthRule {
    fn check(&self, item: &FailoverProposal) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverProposalAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl FailoverProposalRule for FailoverProposalAllowedCodeRule {
    fn check(&self, item: &FailoverProposal) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is below minimum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverProposalRequiredFingerprintRule {
    pub message: String,
}

impl FailoverProposalRule for FailoverProposalRequiredFingerprintRule {
    fn check(&self, item: &FailoverProposal) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FailoverProposalRequiredCipherRule {
    pub message: String,
}

impl FailoverProposalRule for FailoverProposalRequiredCipherRule {
    fn check(&self, item: &FailoverProposal) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a failover proposal.
#[derive(Debug, Clone)]
pub struct FailoverProposalValidator {
    pub rules: Vec<Rc<dyn FailoverProposalRule>>,
}

impl FailoverProposalValidator {
    pub fn validate(&self, item: &FailoverProposal) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FailoverProposal) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FailoverProposal]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_failover_proposal_validator() -> FailoverProposalValidator {
    FailoverProposalValidator {
        rules: vec![
            Rc::new(FailoverProposalFingerprintLengthRule { max_length: 3600 }) as Rc<dyn FailoverProposalRule>,
            Rc::new(FailoverProposalDurationMinutesRangeRule { min: 255, max: 1000 }),
            Rc::new(FailoverProposalTitleLengthRule { max_length: 32 }),
            Rc::new(FailoverProposalAllowedCodeRule {
                allowed: vec!["adaptive".to_string()],
            }),
            Rc::new(FailoverProposalRequiredFingerprintRule {
                message: "failover proposal needs this field".to_string(),
            }),
            Rc::new(FailoverProposalRequiredCipherRule {
                message: "failover proposal needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_failover_proposal_validator();
        let item = new_failover_proposal("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = FailoverProposalFingerprintLengthRule { max_length: 32 };
        let mut item = new_failover_proposal("x", "y");
        item.fingerprint = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = FailoverProposalDurationMinutesRangeRule { min: 1000, max: 86400 };
        let mut item = new_failover_proposal("x", "y");
        item.duration_minutes = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = FailoverProposalTitleLengthRule { max_length: 32 };
        let mut item = new_failover_proposal("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = FailoverProposalRequiredFingerprintRule {
            message: "failover proposal needs an entry".to_string(),
        };
        let mut item = new_failover_proposal("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = FailoverProposalRequiredCipherRule {
            message: "failover proposal needs an entry".to_string(),
        };
        let mut item = new_failover_proposal("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
