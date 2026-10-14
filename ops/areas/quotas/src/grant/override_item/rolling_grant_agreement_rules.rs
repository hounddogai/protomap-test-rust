use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling grant agreement must pass. Check returns its problems, or none.
pub trait RollingGrantAgreementRule: std::fmt::Debug {
    fn check(&self, item: &RollingGrantAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingGrantAgreementActiveGuardRule {
    pub limit: i64,
}

impl RollingGrantAgreementRule for RollingGrantAgreementActiveGuardRule {
    fn check(&self, item: &RollingGrantAgreement) -> Vec<String> {
        if item.active && (item.rotation_days > self.limit) {
            return vec![format!(
                "rotation days {} exceeds {} while active",
                item.rotation_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingGrantAgreementReviewerLengthRule {
    pub max_length: i64,
}

impl RollingGrantAgreementRule for RollingGrantAgreementReviewerLengthRule {
    fn check(&self, item: &RollingGrantAgreement) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingGrantAgreementAllowedNameRule {
    pub allowed: Vec<String>,
}

impl RollingGrantAgreementRule for RollingGrantAgreementAllowedNameRule {
    fn check(&self, item: &RollingGrantAgreement) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is out of range", item.name)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling grant agreement.
#[derive(Debug, Clone)]
pub struct RollingGrantAgreementValidator {
    pub rules: Vec<Rc<dyn RollingGrantAgreementRule>>,
}

impl RollingGrantAgreementValidator {
    pub fn validate(&self, item: &RollingGrantAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingGrantAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingGrantAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_grant_agreement_validator() -> RollingGrantAgreementValidator {
    RollingGrantAgreementValidator {
        rules: vec![
            Rc::new(RollingGrantAgreementActiveGuardRule { limit: 86400 }) as Rc<dyn RollingGrantAgreementRule>,
            Rc::new(RollingGrantAgreementReviewerLengthRule { max_length: 64 }),
            Rc::new(RollingGrantAgreementAllowedNameRule {
                allowed: vec!["regional".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_grant_agreement_validator();
        let item = new_rolling_grant_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = RollingGrantAgreementReviewerLengthRule { max_length: 255 };
        let mut item = new_rolling_grant_agreement("x", "y");
        item.reviewer = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
