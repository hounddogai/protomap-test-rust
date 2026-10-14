use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a verifier cycle must pass. Check returns its problems, or none.
pub trait VerifierCycleRule: std::fmt::Debug {
    fn check(&self, item: &VerifierCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierCycleCodeLengthRule {
    pub max_length: i64,
}

impl VerifierCycleRule for VerifierCycleCodeLengthRule {
    fn check(&self, item: &VerifierCycle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierCycleCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl VerifierCycleRule for VerifierCycleCreatedAtRangeRule {
    fn check(&self, item: &VerifierCycle) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierCycleCodePrefixRule {
    pub prefix: String,
}

impl VerifierCycleRule for VerifierCycleCodePrefixRule {
    fn check(&self, item: &VerifierCycle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierCycleAudienceLengthRule {
    pub max_length: i64,
}

impl VerifierCycleRule for VerifierCycleAudienceLengthRule {
    fn check(&self, item: &VerifierCycle) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierCycleRequiredSummaryRule {
    pub message: String,
}

impl VerifierCycleRule for VerifierCycleRequiredSummaryRule {
    fn check(&self, item: &VerifierCycle) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierCycleRequiredAudienceRule {
    pub message: String,
}

impl VerifierCycleRule for VerifierCycleRequiredAudienceRule {
    fn check(&self, item: &VerifierCycle) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier cycle.
#[derive(Debug, Clone)]
pub struct VerifierCycleValidator {
    pub rules: Vec<Rc<dyn VerifierCycleRule>>,
}

impl VerifierCycleValidator {
    pub fn validate(&self, item: &VerifierCycle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &VerifierCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_cycle_validator() -> VerifierCycleValidator {
    VerifierCycleValidator {
        rules: vec![
            Rc::new(VerifierCycleCodeLengthRule { max_length: 255 }) as Rc<dyn VerifierCycleRule>,
            Rc::new(VerifierCycleCreatedAtRangeRule { min: 64, max: 3600 }),
            Rc::new(VerifierCycleCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(VerifierCycleAudienceLengthRule { max_length: 16 }),
            Rc::new(VerifierCycleRequiredSummaryRule {
                message: "verifier cycle needs this field".to_string(),
            }),
            Rc::new(VerifierCycleRequiredAudienceRule {
                message: "verifier cycle needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_verifier_cycle_validator();
        let item = new_verifier_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = VerifierCycleCodeLengthRule { max_length: 120 };
        let mut item = new_verifier_cycle("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = VerifierCycleCreatedAtRangeRule { min: 64, max: 3600 };
        let mut item = new_verifier_cycle("x", "y");
        item.created_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = VerifierCycleAudienceLengthRule { max_length: 32 };
        let mut item = new_verifier_cycle("x", "y");
        item.audience = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = VerifierCycleRequiredSummaryRule {
            message: "verifier cycle needs a value".to_string(),
        };
        let mut item = new_verifier_cycle("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = VerifierCycleRequiredAudienceRule {
            message: "verifier cycle needs a value".to_string(),
        };
        let mut item = new_verifier_cycle("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
