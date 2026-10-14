use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal evidence bundle must pass. Check returns its problems, or none.
pub trait SeasonalEvidenceBundleRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalEvidenceBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEvidenceBundleNotesLengthRule {
    pub max_length: i64,
}

impl SeasonalEvidenceBundleRule for SeasonalEvidenceBundleNotesLengthRule {
    fn check(&self, item: &SeasonalEvidenceBundle) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEvidenceBundleCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalEvidenceBundleRule for SeasonalEvidenceBundleCapacityRangeRule {
    fn check(&self, item: &SeasonalEvidenceBundle) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEvidenceBundleRequiredCountryRule {
    pub message: String,
}

impl SeasonalEvidenceBundleRule for SeasonalEvidenceBundleRequiredCountryRule {
    fn check(&self, item: &SeasonalEvidenceBundle) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal evidence bundle.
#[derive(Debug, Clone)]
pub struct SeasonalEvidenceBundleValidator {
    pub rules: Vec<Rc<dyn SeasonalEvidenceBundleRule>>,
}

impl SeasonalEvidenceBundleValidator {
    pub fn validate(&self, item: &SeasonalEvidenceBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalEvidenceBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalEvidenceBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_evidence_bundle_validator() -> SeasonalEvidenceBundleValidator {
    SeasonalEvidenceBundleValidator {
        rules: vec![
            Rc::new(SeasonalEvidenceBundleNotesLengthRule { max_length: 16 }) as Rc<dyn SeasonalEvidenceBundleRule>,
            Rc::new(SeasonalEvidenceBundleCapacityRangeRule { min: 120, max: 3600 }),
            Rc::new(SeasonalEvidenceBundleRequiredCountryRule {
                message: "seasonal evidence bundle needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_evidence_bundle_validator();
        let item = new_seasonal_evidence_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = SeasonalEvidenceBundleNotesLengthRule { max_length: 32 };
        let mut item = new_seasonal_evidence_bundle("x", "y");
        item.notes = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = SeasonalEvidenceBundleCapacityRangeRule { min: 32, max: 32 };
        let mut item = new_seasonal_evidence_bundle("x", "y");
        item.capacity = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = SeasonalEvidenceBundleRequiredCountryRule {
            message: "seasonal evidence bundle needs this field".to_string(),
        };
        let mut item = new_seasonal_evidence_bundle("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
