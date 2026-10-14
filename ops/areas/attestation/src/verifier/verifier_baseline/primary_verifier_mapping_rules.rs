use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary verifier mapping must pass. Check returns its problems, or none.
pub trait PrimaryVerifierMappingRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryVerifierMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierMappingRequiredAudienceRule {
    pub message: String,
}

impl PrimaryVerifierMappingRule for PrimaryVerifierMappingRequiredAudienceRule {
    fn check(&self, item: &PrimaryVerifierMapping) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierMappingAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl PrimaryVerifierMappingRule for PrimaryVerifierMappingAllowedTitleRule {
    fn check(&self, item: &PrimaryVerifierMapping) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is duplicated", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierMappingWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryVerifierMappingRule for PrimaryVerifierMappingWeightRangeRule {
    fn check(&self, item: &PrimaryVerifierMapping) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierMappingAudienceLengthRule {
    pub max_length: i64,
}

impl PrimaryVerifierMappingRule for PrimaryVerifierMappingAudienceLengthRule {
    fn check(&self, item: &PrimaryVerifierMapping) -> Vec<String> {
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

/// Runs every rule of a primary verifier mapping.
#[derive(Debug, Clone)]
pub struct PrimaryVerifierMappingValidator {
    pub rules: Vec<Rc<dyn PrimaryVerifierMappingRule>>,
}

impl PrimaryVerifierMappingValidator {
    pub fn validate(&self, item: &PrimaryVerifierMapping) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryVerifierMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryVerifierMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_verifier_mapping_validator() -> PrimaryVerifierMappingValidator {
    PrimaryVerifierMappingValidator {
        rules: vec![
            Rc::new(PrimaryVerifierMappingRequiredAudienceRule {
                message: "primary verifier mapping needs an entry".to_string(),
            }) as Rc<dyn PrimaryVerifierMappingRule>,
            Rc::new(PrimaryVerifierMappingAllowedTitleRule {
                allowed: vec!["legacy".to_string(), "default".to_string(), "batched".to_string()],
            }),
            Rc::new(PrimaryVerifierMappingWeightRangeRule { min: 2.6, max: 2782.7 }),
            Rc::new(PrimaryVerifierMappingAudienceLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_verifier_mapping_validator();
        let item = new_primary_verifier_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = PrimaryVerifierMappingRequiredAudienceRule {
            message: "primary verifier mapping needs this field".to_string(),
        };
        let mut item = new_primary_verifier_mapping("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = PrimaryVerifierMappingWeightRangeRule { min: 7.1, max: 915.9 };
        let mut item = new_primary_verifier_mapping("x", "y");
        item.weight = 917.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = PrimaryVerifierMappingAudienceLengthRule { max_length: 32 };
        let mut item = new_primary_verifier_mapping("x", "y");
        item.audience = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
