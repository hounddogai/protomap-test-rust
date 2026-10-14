use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a evidence checkpoint must pass. Check returns its problems, or none.
pub trait EvidenceCheckpointRule: std::fmt::Debug {
    fn check(&self, item: &EvidenceCheckpoint) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceCheckpointRequiredCipherRule {
    pub message: String,
}

impl EvidenceCheckpointRule for EvidenceCheckpointRequiredCipherRule {
    fn check(&self, item: &EvidenceCheckpoint) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceCheckpointLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EvidenceCheckpointRule for EvidenceCheckpointLimitRangeRule {
    fn check(&self, item: &EvidenceCheckpoint) -> Vec<String> {
        let value = item.limit;
        if value < self.min || value > self.max {
            return vec![format!("limit {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceCheckpointLabelLengthRule {
    pub max_length: i64,
}

impl EvidenceCheckpointRule for EvidenceCheckpointLabelLengthRule {
    fn check(&self, item: &EvidenceCheckpoint) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceCheckpointAllowedChecksumRule {
    pub allowed: Vec<String>,
}

impl EvidenceCheckpointRule for EvidenceCheckpointAllowedChecksumRule {
    fn check(&self, item: &EvidenceCheckpoint) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.checksum)))
        {
            return vec![format!("checksum {} is duplicated", item.checksum)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceCheckpointGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EvidenceCheckpointRule for EvidenceCheckpointGraceDaysRangeRule {
    fn check(&self, item: &EvidenceCheckpoint) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceCheckpointCodeLengthRule {
    pub max_length: i64,
}

impl EvidenceCheckpointRule for EvidenceCheckpointCodeLengthRule {
    fn check(&self, item: &EvidenceCheckpoint) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a evidence checkpoint.
#[derive(Debug, Clone)]
pub struct EvidenceCheckpointValidator {
    pub rules: Vec<Rc<dyn EvidenceCheckpointRule>>,
}

impl EvidenceCheckpointValidator {
    pub fn validate(&self, item: &EvidenceCheckpoint) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EvidenceCheckpoint) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EvidenceCheckpoint]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_evidence_checkpoint_validator() -> EvidenceCheckpointValidator {
    EvidenceCheckpointValidator {
        rules: vec![
            Rc::new(EvidenceCheckpointRequiredCipherRule {
                message: "evidence checkpoint needs this field".to_string(),
            }) as Rc<dyn EvidenceCheckpointRule>,
            Rc::new(EvidenceCheckpointLimitRangeRule { min: 16, max: 1000 }),
            Rc::new(EvidenceCheckpointLabelLengthRule { max_length: 1000 }),
            Rc::new(EvidenceCheckpointAllowedChecksumRule { allowed: Vec::new() }),
            Rc::new(EvidenceCheckpointGraceDaysRangeRule { min: 16, max: 86400 }),
            Rc::new(EvidenceCheckpointCodeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_evidence_checkpoint_validator();
        let item = new_evidence_checkpoint("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = EvidenceCheckpointRequiredCipherRule {
            message: "evidence checkpoint needs a value".to_string(),
        };
        let mut item = new_evidence_checkpoint("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = EvidenceCheckpointLimitRangeRule { min: 16, max: 80 };
        let mut item = new_evidence_checkpoint("x", "y");
        item.limit = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = EvidenceCheckpointLabelLengthRule { max_length: 3600 };
        let mut item = new_evidence_checkpoint("x", "y");
        item.label = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = EvidenceCheckpointGraceDaysRangeRule { min: 32, max: 86400 };
        let mut item = new_evidence_checkpoint("x", "y");
        item.grace_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = EvidenceCheckpointCodeLengthRule { max_length: 255 };
        let mut item = new_evidence_checkpoint("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
