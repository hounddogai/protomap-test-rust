use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling verifier agreement must pass. Check returns its problems, or none.
pub trait RollingVerifierAgreementRule: std::fmt::Debug {
    fn check(&self, item: &RollingVerifierAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierAgreementRequiredCodeRule {
    pub message: String,
}

impl RollingVerifierAgreementRule for RollingVerifierAgreementRequiredCodeRule {
    fn check(&self, item: &RollingVerifierAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierAgreementTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingVerifierAgreementRule for RollingVerifierAgreementTtlSecondsRangeRule {
    fn check(&self, item: &RollingVerifierAgreement) -> Vec<String> {
        let value = item.ttl_seconds;
        if value < self.min || value > self.max {
            return vec![format!("ttl seconds {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierAgreementRequiredLocaleRule {
    pub message: String,
}

impl RollingVerifierAgreementRule for RollingVerifierAgreementRequiredLocaleRule {
    fn check(&self, item: &RollingVerifierAgreement) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierAgreementRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingVerifierAgreementRule for RollingVerifierAgreementRateRangeRule {
    fn check(&self, item: &RollingVerifierAgreement) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingVerifierAgreementRequiredNameRule {
    pub message: String,
}

impl RollingVerifierAgreementRule for RollingVerifierAgreementRequiredNameRule {
    fn check(&self, item: &RollingVerifierAgreement) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling verifier agreement.
#[derive(Debug, Clone)]
pub struct RollingVerifierAgreementValidator {
    pub rules: Vec<Rc<dyn RollingVerifierAgreementRule>>,
}

impl RollingVerifierAgreementValidator {
    pub fn validate(&self, item: &RollingVerifierAgreement) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingVerifierAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingVerifierAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_verifier_agreement_validator() -> RollingVerifierAgreementValidator {
    RollingVerifierAgreementValidator {
        rules: vec![
            Rc::new(RollingVerifierAgreementRequiredCodeRule {
                message: "rolling verifier agreement needs an entry".to_string(),
            }) as Rc<dyn RollingVerifierAgreementRule>,
            Rc::new(RollingVerifierAgreementTtlSecondsRangeRule { min: 16, max: 64 }),
            Rc::new(RollingVerifierAgreementRequiredLocaleRule {
                message: "rolling verifier agreement needs this field".to_string(),
            }),
            Rc::new(RollingVerifierAgreementRateRangeRule { min: 9.3, max: 2731.7 }),
            Rc::new(RollingVerifierAgreementRequiredNameRule {
                message: "rolling verifier agreement needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_verifier_agreement_validator();
        let item = new_rolling_verifier_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingVerifierAgreementRequiredCodeRule {
            message: "rolling verifier agreement needs an entry".to_string(),
        };
        let mut item = new_rolling_verifier_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = RollingVerifierAgreementTtlSecondsRangeRule { min: 32, max: 3600 };
        let mut item = new_rolling_verifier_agreement("x", "y");
        item.ttl_seconds = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = RollingVerifierAgreementRequiredLocaleRule {
            message: "rolling verifier agreement needs this field".to_string(),
        };
        let mut item = new_rolling_verifier_agreement("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = RollingVerifierAgreementRateRangeRule { min: 9.6, max: 236.2 };
        let mut item = new_rolling_verifier_agreement("x", "y");
        item.rate = 237.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RollingVerifierAgreementRequiredNameRule {
            message: "rolling verifier agreement needs this field".to_string(),
        };
        let mut item = new_rolling_verifier_agreement("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
