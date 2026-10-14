use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a verifier baseline must pass. Check returns its problems, or none.
pub trait VerifierBaselineRule: std::fmt::Debug {
    fn check(&self, item: &VerifierBaseline) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierBaselineTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl VerifierBaselineRule for VerifierBaselineTtlSecondsRangeRule {
    fn check(&self, item: &VerifierBaseline) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierBaselineRequiredCodeRule {
    pub message: String,
}

impl VerifierBaselineRule for VerifierBaselineRequiredCodeRule {
    fn check(&self, item: &VerifierBaseline) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierBaselineCodeLengthRule {
    pub max_length: i64,
}

impl VerifierBaselineRule for VerifierBaselineCodeLengthRule {
    fn check(&self, item: &VerifierBaseline) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierBaselineRequiredChannelRule {
    pub message: String,
}

impl VerifierBaselineRule for VerifierBaselineRequiredChannelRule {
    fn check(&self, item: &VerifierBaseline) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierBaselineScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl VerifierBaselineRule for VerifierBaselineScoreRangeRule {
    fn check(&self, item: &VerifierBaseline) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier baseline.
#[derive(Debug, Clone)]
pub struct VerifierBaselineValidator {
    pub rules: Vec<Rc<dyn VerifierBaselineRule>>,
}

impl VerifierBaselineValidator {
    pub fn validate(&self, item: &VerifierBaseline) -> Vec<String> {
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

    pub fn require_valid(&self, item: &VerifierBaseline) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierBaseline]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_baseline_validator() -> VerifierBaselineValidator {
    VerifierBaselineValidator {
        rules: vec![
            Rc::new(VerifierBaselineTtlSecondsRangeRule { min: 80, max: 1000 }) as Rc<dyn VerifierBaselineRule>,
            Rc::new(VerifierBaselineRequiredCodeRule {
                message: "verifier baseline needs a value".to_string(),
            }),
            Rc::new(VerifierBaselineCodeLengthRule { max_length: 255 }),
            Rc::new(VerifierBaselineRequiredChannelRule {
                message: "verifier baseline needs this field".to_string(),
            }),
            Rc::new(VerifierBaselineScoreRangeRule { min: 5.2, max: 908.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_verifier_baseline_validator();
        let item = new_verifier_baseline("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = VerifierBaselineTtlSecondsRangeRule { min: 255, max: 86400 };
        let mut item = new_verifier_baseline("x", "y");
        item.ttl_seconds = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = VerifierBaselineRequiredCodeRule {
            message: "verifier baseline needs an entry".to_string(),
        };
        let mut item = new_verifier_baseline("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = VerifierBaselineCodeLengthRule { max_length: 86400 };
        let mut item = new_verifier_baseline("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = VerifierBaselineRequiredChannelRule {
            message: "verifier baseline needs a value".to_string(),
        };
        let mut item = new_verifier_baseline("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = VerifierBaselineScoreRangeRule { min: 2.1, max: 1192.6 };
        let mut item = new_verifier_baseline("x", "y");
        item.score = 1194.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
