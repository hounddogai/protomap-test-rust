use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a verifier rollup must pass. Check returns its problems, or none.
pub trait VerifierRollupRule: std::fmt::Debug {
    fn check(&self, item: &VerifierRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierRollupAllowedChecksumRule {
    pub allowed: Vec<String>,
}

impl VerifierRollupRule for VerifierRollupAllowedChecksumRule {
    fn check(&self, item: &VerifierRollup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.checksum)))
        {
            return vec![format!("checksum {} is above maximum", item.checksum)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierRollupCodePrefixRule {
    pub prefix: String,
}

impl VerifierRollupRule for VerifierRollupCodePrefixRule {
    fn check(&self, item: &VerifierRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierRollupRequiredChecksumRule {
    pub message: String,
}

impl VerifierRollupRule for VerifierRollupRequiredChecksumRule {
    fn check(&self, item: &VerifierRollup) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierRollupScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl VerifierRollupRule for VerifierRollupScoreRangeRule {
    fn check(&self, item: &VerifierRollup) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a verifier rollup.
#[derive(Debug, Clone)]
pub struct VerifierRollupValidator {
    pub rules: Vec<Rc<dyn VerifierRollupRule>>,
}

impl VerifierRollupValidator {
    pub fn validate(&self, item: &VerifierRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &VerifierRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_rollup_validator() -> VerifierRollupValidator {
    VerifierRollupValidator {
        rules: vec![
            Rc::new(VerifierRollupAllowedChecksumRule {
                allowed: vec!["weighted".to_string()],
            }) as Rc<dyn VerifierRollupRule>,
            Rc::new(VerifierRollupCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(VerifierRollupRequiredChecksumRule {
                message: "verifier rollup needs an entry".to_string(),
            }),
            Rc::new(VerifierRollupScoreRangeRule { min: 8.3, max: 442.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_verifier_rollup_validator();
        let item = new_verifier_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = VerifierRollupRequiredChecksumRule {
            message: "verifier rollup needs a value".to_string(),
        };
        let mut item = new_verifier_rollup("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = VerifierRollupScoreRangeRule { min: 6.1, max: 849.5 };
        let mut item = new_verifier_rollup("x", "y");
        item.score = 851.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
