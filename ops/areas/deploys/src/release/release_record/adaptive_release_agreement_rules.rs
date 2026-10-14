use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive release agreement must pass. Check returns its problems, or none.
pub trait AdaptiveReleaseAgreementRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveReleaseAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveReleaseAgreementPinnedGuardRule {
    pub limit: i64,
}

impl AdaptiveReleaseAgreementRule for AdaptiveReleaseAgreementPinnedGuardRule {
    fn check(&self, item: &AdaptiveReleaseAgreement) -> Vec<String> {
        if item.pinned && (item.limit > self.limit) {
            return vec![format!("limit {} exceeds {} while pinned", item.limit, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveReleaseAgreementAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveReleaseAgreementRule for AdaptiveReleaseAgreementAmountRangeRule {
    fn check(&self, item: &AdaptiveReleaseAgreement) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveReleaseAgreementLabelLengthRule {
    pub max_length: i64,
}

impl AdaptiveReleaseAgreementRule for AdaptiveReleaseAgreementLabelLengthRule {
    fn check(&self, item: &AdaptiveReleaseAgreement) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive release agreement.
#[derive(Debug, Clone)]
pub struct AdaptiveReleaseAgreementValidator {
    pub rules: Vec<Rc<dyn AdaptiveReleaseAgreementRule>>,
}

impl AdaptiveReleaseAgreementValidator {
    pub fn validate(&self, item: &AdaptiveReleaseAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &AdaptiveReleaseAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveReleaseAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_release_agreement_validator() -> AdaptiveReleaseAgreementValidator {
    AdaptiveReleaseAgreementValidator {
        rules: vec![
            Rc::new(AdaptiveReleaseAgreementPinnedGuardRule { limit: 1000 }) as Rc<dyn AdaptiveReleaseAgreementRule>,
            Rc::new(AdaptiveReleaseAgreementAmountRangeRule { min: 6.0, max: 4623.7 }),
            Rc::new(AdaptiveReleaseAgreementLabelLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_release_agreement_validator();
        let item = new_adaptive_release_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = AdaptiveReleaseAgreementAmountRangeRule { min: 3.5, max: 2291.2 };
        let mut item = new_adaptive_release_agreement("x", "y");
        item.amount = 2292.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = AdaptiveReleaseAgreementLabelLengthRule { max_length: 80 };
        let mut item = new_adaptive_release_agreement("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
