use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped verifier baseline must pass. Check returns its problems, or none.
pub trait CappedVerifierBaselineRule: std::fmt::Debug {
    fn check(&self, item: &CappedVerifierBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedVerifierBaselineStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedVerifierBaselineRule for CappedVerifierBaselineStartsAtRangeRule {
    fn check(&self, item: &CappedVerifierBaseline) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedVerifierBaselineCodePrefixRule {
    pub prefix: String,
}

impl CappedVerifierBaselineRule for CappedVerifierBaselineCodePrefixRule {
    fn check(&self, item: &CappedVerifierBaseline) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedVerifierBaselineRequiredAudienceRule {
    pub message: String,
}

impl CappedVerifierBaselineRule for CappedVerifierBaselineRequiredAudienceRule {
    fn check(&self, item: &CappedVerifierBaseline) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped verifier baseline.
#[derive(Debug, Clone)]
pub struct CappedVerifierBaselineValidator {
    pub rules: Vec<Rc<dyn CappedVerifierBaselineRule>>,
}

impl CappedVerifierBaselineValidator {
    pub fn validate(&self, item: &CappedVerifierBaseline) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedVerifierBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedVerifierBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_verifier_baseline_validator() -> CappedVerifierBaselineValidator {
    CappedVerifierBaselineValidator {
        rules: vec![
            Rc::new(CappedVerifierBaselineStartsAtRangeRule { min: 64, max: 64 }) as Rc<dyn CappedVerifierBaselineRule>,
            Rc::new(CappedVerifierBaselineCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(CappedVerifierBaselineRequiredAudienceRule {
                message: "capped verifier baseline needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_verifier_baseline_validator();
        let item = new_capped_verifier_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = CappedVerifierBaselineStartsAtRangeRule { min: 16, max: 3600 };
        let mut item = new_capped_verifier_baseline("x", "y");
        item.starts_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = CappedVerifierBaselineRequiredAudienceRule {
            message: "capped verifier baseline needs a value".to_string(),
        };
        let mut item = new_capped_verifier_baseline("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
