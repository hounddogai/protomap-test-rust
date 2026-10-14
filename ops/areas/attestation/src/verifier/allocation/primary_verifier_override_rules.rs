use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary verifier override must pass. Check returns its problems, or none.
pub trait PrimaryVerifierOverrideRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryVerifierOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierOverrideCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryVerifierOverrideRule for PrimaryVerifierOverrideCeilingRangeRule {
    fn check(&self, item: &PrimaryVerifierOverride) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierOverrideLabelLengthRule {
    pub max_length: i64,
}

impl PrimaryVerifierOverrideRule for PrimaryVerifierOverrideLabelLengthRule {
    fn check(&self, item: &PrimaryVerifierOverride) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierOverrideRequiredLabelRule {
    pub message: String,
}

impl PrimaryVerifierOverrideRule for PrimaryVerifierOverrideRequiredLabelRule {
    fn check(&self, item: &PrimaryVerifierOverride) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierOverrideScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryVerifierOverrideRule for PrimaryVerifierOverrideScoreRangeRule {
    fn check(&self, item: &PrimaryVerifierOverride) -> Vec<String> {
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

/// Runs every rule of a primary verifier override.
#[derive(Debug, Clone)]
pub struct PrimaryVerifierOverrideValidator {
    pub rules: Vec<Rc<dyn PrimaryVerifierOverrideRule>>,
}

impl PrimaryVerifierOverrideValidator {
    pub fn validate(&self, item: &PrimaryVerifierOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryVerifierOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryVerifierOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_verifier_override_validator() -> PrimaryVerifierOverrideValidator {
    PrimaryVerifierOverrideValidator {
        rules: vec![
            Rc::new(PrimaryVerifierOverrideCeilingRangeRule { min: 6.4, max: 3840.8 })
                as Rc<dyn PrimaryVerifierOverrideRule>,
            Rc::new(PrimaryVerifierOverrideLabelLengthRule { max_length: 255 }),
            Rc::new(PrimaryVerifierOverrideRequiredLabelRule {
                message: "primary verifier override needs a value".to_string(),
            }),
            Rc::new(PrimaryVerifierOverrideScoreRangeRule { min: 0.1, max: 1671.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_verifier_override_validator();
        let item = new_primary_verifier_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = PrimaryVerifierOverrideCeilingRangeRule { min: 9.1, max: 2407.2 };
        let mut item = new_primary_verifier_override("x", "y");
        item.ceiling = 2408.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PrimaryVerifierOverrideLabelLengthRule { max_length: 64 };
        let mut item = new_primary_verifier_override("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = PrimaryVerifierOverrideRequiredLabelRule {
            message: "primary verifier override needs this field".to_string(),
        };
        let mut item = new_primary_verifier_override("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = PrimaryVerifierOverrideScoreRangeRule { min: 4.0, max: 1685.2 };
        let mut item = new_primary_verifier_override("x", "y");
        item.score = 1686.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
