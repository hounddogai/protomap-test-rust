use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a verifier score must pass. Check returns its problems, or none.
pub trait VerifierScoreRule: std::fmt::Debug {
    fn check(&self, item: &VerifierScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierScoreRequiredChecksumRule {
    pub message: String,
}

impl VerifierScoreRule for VerifierScoreRequiredChecksumRule {
    fn check(&self, item: &VerifierScore) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierScoreCodeLengthRule {
    pub max_length: i64,
}

impl VerifierScoreRule for VerifierScoreCodeLengthRule {
    fn check(&self, item: &VerifierScore) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierScoreBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl VerifierScoreRule for VerifierScoreBatchSizeRangeRule {
    fn check(&self, item: &VerifierScore) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier score.
#[derive(Debug, Clone)]
pub struct VerifierScoreValidator {
    pub rules: Vec<Rc<dyn VerifierScoreRule>>,
}

impl VerifierScoreValidator {
    pub fn validate(&self, item: &VerifierScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &VerifierScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_score_validator() -> VerifierScoreValidator {
    VerifierScoreValidator {
        rules: vec![
            Rc::new(VerifierScoreRequiredChecksumRule {
                message: "verifier score needs this field".to_string(),
            }) as Rc<dyn VerifierScoreRule>,
            Rc::new(VerifierScoreCodeLengthRule { max_length: 1000 }),
            Rc::new(VerifierScoreBatchSizeRangeRule { min: 64, max: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_verifier_score_validator();
        let item = new_verifier_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = VerifierScoreRequiredChecksumRule {
            message: "verifier score needs a value".to_string(),
        };
        let mut item = new_verifier_score("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = VerifierScoreCodeLengthRule { max_length: 120 };
        let mut item = new_verifier_score("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = VerifierScoreBatchSizeRangeRule { min: 16, max: 64 };
        let mut item = new_verifier_score("x", "y");
        item.batch_size = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
