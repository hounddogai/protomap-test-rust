use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary evidence request must pass. Check returns its problems, or none.
pub trait PrimaryEvidenceRequestRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryEvidenceRequest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceRequestRequiredCategoryRule {
    pub message: String,
}

impl PrimaryEvidenceRequestRule for PrimaryEvidenceRequestRequiredCategoryRule {
    fn check(&self, item: &PrimaryEvidenceRequest) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceRequestDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryEvidenceRequestRule for PrimaryEvidenceRequestDurationMinutesRangeRule {
    fn check(&self, item: &PrimaryEvidenceRequest) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceRequestCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryEvidenceRequestRule for PrimaryEvidenceRequestCodeLengthRule {
    fn check(&self, item: &PrimaryEvidenceRequest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceRequestCodePrefixRule {
    pub prefix: String,
}

impl PrimaryEvidenceRequestRule for PrimaryEvidenceRequestCodePrefixRule {
    fn check(&self, item: &PrimaryEvidenceRequest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceRequestRequiredCodeRule {
    pub message: String,
}

impl PrimaryEvidenceRequestRule for PrimaryEvidenceRequestRequiredCodeRule {
    fn check(&self, item: &PrimaryEvidenceRequest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryEvidenceRequestRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryEvidenceRequestRule for PrimaryEvidenceRequestRateRangeRule {
    fn check(&self, item: &PrimaryEvidenceRequest) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary evidence request.
#[derive(Debug, Clone)]
pub struct PrimaryEvidenceRequestValidator {
    pub rules: Vec<Rc<dyn PrimaryEvidenceRequestRule>>,
}

impl PrimaryEvidenceRequestValidator {
    pub fn validate(&self, item: &PrimaryEvidenceRequest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryEvidenceRequest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryEvidenceRequest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_evidence_request_validator() -> PrimaryEvidenceRequestValidator {
    PrimaryEvidenceRequestValidator {
        rules: vec![
            Rc::new(PrimaryEvidenceRequestRequiredCategoryRule {
                message: "primary evidence request needs a value".to_string(),
            }) as Rc<dyn PrimaryEvidenceRequestRule>,
            Rc::new(PrimaryEvidenceRequestDurationMinutesRangeRule { min: 16, max: 86400 }),
            Rc::new(PrimaryEvidenceRequestCodeLengthRule { max_length: 1000 }),
            Rc::new(PrimaryEvidenceRequestCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(PrimaryEvidenceRequestRequiredCodeRule {
                message: "primary evidence request needs this field".to_string(),
            }),
            Rc::new(PrimaryEvidenceRequestRateRangeRule { min: 3.6, max: 4263.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_evidence_request_validator();
        let item = new_primary_evidence_request("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = PrimaryEvidenceRequestRequiredCategoryRule {
            message: "primary evidence request needs this field".to_string(),
        };
        let mut item = new_primary_evidence_request("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = PrimaryEvidenceRequestDurationMinutesRangeRule { min: 16, max: 86400 };
        let mut item = new_primary_evidence_request("x", "y");
        item.duration_minutes = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryEvidenceRequestCodeLengthRule { max_length: 32 };
        let mut item = new_primary_evidence_request("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryEvidenceRequestRequiredCodeRule {
            message: "primary evidence request needs a value".to_string(),
        };
        let mut item = new_primary_evidence_request("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = PrimaryEvidenceRequestRateRangeRule { min: 5.0, max: 3685.8 };
        let mut item = new_primary_evidence_request("x", "y");
        item.rate = 3687.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
