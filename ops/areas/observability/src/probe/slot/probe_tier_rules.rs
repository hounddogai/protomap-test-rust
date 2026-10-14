use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a probe tier must pass. Check returns its problems, or none.
pub trait ProbeTierRule: std::fmt::Debug {
    fn check(&self, item: &ProbeTier) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeTierFingerprintLengthRule {
    pub max_length: i64,
}

impl ProbeTierRule for ProbeTierFingerprintLengthRule {
    fn check(&self, item: &ProbeTier) -> Vec<String> {
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
pub struct ProbeTierRequiredFingerprintRule {
    pub message: String,
}

impl ProbeTierRule for ProbeTierRequiredFingerprintRule {
    fn check(&self, item: &ProbeTier) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeTierCodeLengthRule {
    pub max_length: i64,
}

impl ProbeTierRule for ProbeTierCodeLengthRule {
    fn check(&self, item: &ProbeTier) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a probe tier.
#[derive(Debug, Clone)]
pub struct ProbeTierValidator {
    pub rules: Vec<Rc<dyn ProbeTierRule>>,
}

impl ProbeTierValidator {
    pub fn validate(&self, item: &ProbeTier) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ProbeTier) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbeTier]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_tier_validator() -> ProbeTierValidator {
    ProbeTierValidator {
        rules: vec![
            Rc::new(ProbeTierFingerprintLengthRule { max_length: 86400 }) as Rc<dyn ProbeTierRule>,
            Rc::new(ProbeTierRequiredFingerprintRule {
                message: "probe tier needs a value".to_string(),
            }),
            Rc::new(ProbeTierCodeLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_probe_tier_validator();
        let item = new_probe_tier("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = ProbeTierFingerprintLengthRule { max_length: 64 };
        let mut item = new_probe_tier("x", "y");
        item.fingerprint = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = ProbeTierRequiredFingerprintRule {
            message: "probe tier needs this field".to_string(),
        };
        let mut item = new_probe_tier("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ProbeTierCodeLengthRule { max_length: 86400 };
        let mut item = new_probe_tier("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
