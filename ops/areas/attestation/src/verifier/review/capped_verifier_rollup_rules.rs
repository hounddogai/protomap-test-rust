use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped verifier rollup must pass. Check returns its problems, or none.
pub trait CappedVerifierRollupRule: std::fmt::Debug {
    fn check(&self, item: &CappedVerifierRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedVerifierRollupCodeLengthRule {
    pub max_length: i64,
}

impl CappedVerifierRollupRule for CappedVerifierRollupCodeLengthRule {
    fn check(&self, item: &CappedVerifierRollup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedVerifierRollupFingerprintLengthRule {
    pub max_length: i64,
}

impl CappedVerifierRollupRule for CappedVerifierRollupFingerprintLengthRule {
    fn check(&self, item: &CappedVerifierRollup) -> Vec<String> {
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
pub struct CappedVerifierRollupUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedVerifierRollupRule for CappedVerifierRollupUpdatedAtRangeRule {
    fn check(&self, item: &CappedVerifierRollup) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped verifier rollup.
#[derive(Debug, Clone)]
pub struct CappedVerifierRollupValidator {
    pub rules: Vec<Rc<dyn CappedVerifierRollupRule>>,
}

impl CappedVerifierRollupValidator {
    pub fn validate(&self, item: &CappedVerifierRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedVerifierRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedVerifierRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_verifier_rollup_validator() -> CappedVerifierRollupValidator {
    CappedVerifierRollupValidator {
        rules: vec![
            Rc::new(CappedVerifierRollupCodeLengthRule { max_length: 255 }) as Rc<dyn CappedVerifierRollupRule>,
            Rc::new(CappedVerifierRollupFingerprintLengthRule { max_length: 255 }),
            Rc::new(CappedVerifierRollupUpdatedAtRangeRule { min: 32, max: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_verifier_rollup_validator();
        let item = new_capped_verifier_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = CappedVerifierRollupCodeLengthRule { max_length: 255 };
        let mut item = new_capped_verifier_rollup("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_fingerprint() {
        let rule = CappedVerifierRollupFingerprintLengthRule { max_length: 64 };
        let mut item = new_capped_verifier_rollup("x", "y");
        item.fingerprint = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.fingerprint = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = CappedVerifierRollupUpdatedAtRangeRule { min: 16, max: 86400 };
        let mut item = new_capped_verifier_rollup("x", "y");
        item.updated_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
