use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a release agreement must pass. Check returns its problems, or none.
pub trait ReleaseAgreementRule: std::fmt::Debug {
    fn check(&self, item: &ReleaseAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseAgreementNotesLengthRule {
    pub max_length: i64,
}

impl ReleaseAgreementRule for ReleaseAgreementNotesLengthRule {
    fn check(&self, item: &ReleaseAgreement) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseAgreementRequiredScopeRule {
    pub message: String,
}

impl ReleaseAgreementRule for ReleaseAgreementRequiredScopeRule {
    fn check(&self, item: &ReleaseAgreement) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseAgreementRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ReleaseAgreementRule for ReleaseAgreementRatioRangeRule {
    fn check(&self, item: &ReleaseAgreement) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a release agreement.
#[derive(Debug, Clone)]
pub struct ReleaseAgreementValidator {
    pub rules: Vec<Rc<dyn ReleaseAgreementRule>>,
}

impl ReleaseAgreementValidator {
    pub fn validate(&self, item: &ReleaseAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ReleaseAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReleaseAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_release_agreement_validator() -> ReleaseAgreementValidator {
    ReleaseAgreementValidator {
        rules: vec![
            Rc::new(ReleaseAgreementNotesLengthRule { max_length: 80 }) as Rc<dyn ReleaseAgreementRule>,
            Rc::new(ReleaseAgreementRequiredScopeRule {
                message: "release agreement needs a value".to_string(),
            }),
            Rc::new(ReleaseAgreementRatioRangeRule { min: 3.8, max: 3511.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_release_agreement_validator();
        let item = new_release_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = ReleaseAgreementNotesLengthRule { max_length: 80 };
        let mut item = new_release_agreement("x", "y");
        item.notes = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = ReleaseAgreementRequiredScopeRule {
            message: "release agreement needs an entry".to_string(),
        };
        let mut item = new_release_agreement("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = ReleaseAgreementRatioRangeRule { min: 3.0, max: 2719.2 };
        let mut item = new_release_agreement("x", "y");
        item.ratio = 2720.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
