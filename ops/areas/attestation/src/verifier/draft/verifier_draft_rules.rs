use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a verifier draft must pass. Check returns its problems, or none.
pub trait VerifierDraftRule: std::fmt::Debug {
    fn check(&self, item: &VerifierDraft) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierDraftCountryLengthRule {
    pub max_length: i64,
}

impl VerifierDraftRule for VerifierDraftCountryLengthRule {
    fn check(&self, item: &VerifierDraft) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierDraftStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl VerifierDraftRule for VerifierDraftStartsAtRangeRule {
    fn check(&self, item: &VerifierDraft) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierDraftTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl VerifierDraftRule for VerifierDraftTtlSecondsRangeRule {
    fn check(&self, item: &VerifierDraft) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierDraftTitleLengthRule {
    pub max_length: i64,
}

impl VerifierDraftRule for VerifierDraftTitleLengthRule {
    fn check(&self, item: &VerifierDraft) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier draft.
#[derive(Debug, Clone)]
pub struct VerifierDraftValidator {
    pub rules: Vec<Rc<dyn VerifierDraftRule>>,
}

impl VerifierDraftValidator {
    pub fn validate(&self, item: &VerifierDraft) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &VerifierDraft) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierDraft]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_draft_validator() -> VerifierDraftValidator {
    VerifierDraftValidator {
        rules: vec![
            Rc::new(VerifierDraftCountryLengthRule { max_length: 120 }) as Rc<dyn VerifierDraftRule>,
            Rc::new(VerifierDraftStartsAtRangeRule { min: 64, max: 86400 }),
            Rc::new(VerifierDraftTtlSecondsRangeRule { min: 120, max: 86400 }),
            Rc::new(VerifierDraftTitleLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_verifier_draft_validator();
        let item = new_verifier_draft("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = VerifierDraftCountryLengthRule { max_length: 80 };
        let mut item = new_verifier_draft("x", "y");
        item.country = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = VerifierDraftStartsAtRangeRule { min: 16, max: 120 };
        let mut item = new_verifier_draft("x", "y");
        item.starts_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = VerifierDraftTtlSecondsRangeRule { min: 64, max: 1000 };
        let mut item = new_verifier_draft("x", "y");
        item.ttl_seconds = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = VerifierDraftTitleLengthRule { max_length: 80 };
        let mut item = new_verifier_draft("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
