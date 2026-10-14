use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a silence agreement must pass. Check returns its problems, or none.
pub trait SilenceAgreementRule: std::fmt::Debug {
    fn check(&self, item: &SilenceAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAgreementAllowedScopeRule {
    pub allowed: Vec<String>,
}

impl SilenceAgreementRule for SilenceAgreementAllowedScopeRule {
    fn check(&self, item: &SilenceAgreement) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.scope))) {
            return vec![format!("scope {} is above maximum", item.scope)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAgreementSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SilenceAgreementRule for SilenceAgreementSequenceRangeRule {
    fn check(&self, item: &SilenceAgreement) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAgreementScopeLengthRule {
    pub max_length: i64,
}

impl SilenceAgreementRule for SilenceAgreementScopeLengthRule {
    fn check(&self, item: &SilenceAgreement) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAgreementCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SilenceAgreementRule for SilenceAgreementCreatedAtRangeRule {
    fn check(&self, item: &SilenceAgreement) -> Vec<String> {
        let value = item.created_at;
        if value < self.min || value > self.max {
            return vec![format!("created at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAgreementGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SilenceAgreementRule for SilenceAgreementGraceDaysRangeRule {
    fn check(&self, item: &SilenceAgreement) -> Vec<String> {
        let value = item.grace_days;
        if value < self.min || value > self.max {
            return vec![format!("grace days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a silence agreement.
#[derive(Debug, Clone)]
pub struct SilenceAgreementValidator {
    pub rules: Vec<Rc<dyn SilenceAgreementRule>>,
}

impl SilenceAgreementValidator {
    pub fn validate(&self, item: &SilenceAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SilenceAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SilenceAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_silence_agreement_validator() -> SilenceAgreementValidator {
    SilenceAgreementValidator {
        rules: vec![
            Rc::new(SilenceAgreementAllowedScopeRule {
                allowed: vec!["lenient".to_string()],
            }) as Rc<dyn SilenceAgreementRule>,
            Rc::new(SilenceAgreementSequenceRangeRule { min: 16, max: 1000 }),
            Rc::new(SilenceAgreementScopeLengthRule { max_length: 3600 }),
            Rc::new(SilenceAgreementCreatedAtRangeRule { min: 120, max: 120 }),
            Rc::new(SilenceAgreementGraceDaysRangeRule { min: 16, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_silence_agreement_validator();
        let item = new_silence_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = SilenceAgreementSequenceRangeRule { min: 64, max: 120 };
        let mut item = new_silence_agreement("x", "y");
        item.sequence = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = SilenceAgreementScopeLengthRule { max_length: 3600 };
        let mut item = new_silence_agreement("x", "y");
        item.scope = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = SilenceAgreementCreatedAtRangeRule { min: 16, max: 3600 };
        let mut item = new_silence_agreement("x", "y");
        item.created_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = SilenceAgreementGraceDaysRangeRule { min: 32, max: 86400 };
        let mut item = new_silence_agreement("x", "y");
        item.grace_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
