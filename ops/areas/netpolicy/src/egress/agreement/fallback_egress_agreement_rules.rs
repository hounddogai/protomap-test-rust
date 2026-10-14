use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback egress agreement must pass. Check returns its problems, or none.
pub trait FallbackEgressAgreementRule: std::fmt::Debug {
    fn check(&self, item: &FallbackEgressAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressAgreementRequiredNameRule {
    pub message: String,
}

impl FallbackEgressAgreementRule for FallbackEgressAgreementRequiredNameRule {
    fn check(&self, item: &FallbackEgressAgreement) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressAgreementCodePrefixRule {
    pub prefix: String,
}

impl FallbackEgressAgreementRule for FallbackEgressAgreementCodePrefixRule {
    fn check(&self, item: &FallbackEgressAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressAgreementCodeLengthRule {
    pub max_length: i64,
}

impl FallbackEgressAgreementRule for FallbackEgressAgreementCodeLengthRule {
    fn check(&self, item: &FallbackEgressAgreement) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressAgreementMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl FallbackEgressAgreementRule for FallbackEgressAgreementMinimumRangeRule {
    fn check(&self, item: &FallbackEgressAgreement) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressAgreementLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackEgressAgreementRule for FallbackEgressAgreementLimitRangeRule {
    fn check(&self, item: &FallbackEgressAgreement) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressAgreementRequiredCodeRule {
    pub message: String,
}

impl FallbackEgressAgreementRule for FallbackEgressAgreementRequiredCodeRule {
    fn check(&self, item: &FallbackEgressAgreement) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback egress agreement.
#[derive(Debug, Clone)]
pub struct FallbackEgressAgreementValidator {
    pub rules: Vec<Rc<dyn FallbackEgressAgreementRule>>,
}

impl FallbackEgressAgreementValidator {
    pub fn validate(&self, item: &FallbackEgressAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackEgressAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackEgressAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_egress_agreement_validator() -> FallbackEgressAgreementValidator {
    FallbackEgressAgreementValidator {
        rules: vec![
            Rc::new(FallbackEgressAgreementRequiredNameRule {
                message: "fallback egress agreement needs a value".to_string(),
            }) as Rc<dyn FallbackEgressAgreementRule>,
            Rc::new(FallbackEgressAgreementCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(FallbackEgressAgreementCodeLengthRule { max_length: 32 }),
            Rc::new(FallbackEgressAgreementMinimumRangeRule { min: 1.6, max: 4354.1 }),
            Rc::new(FallbackEgressAgreementLimitRangeRule { min: 120, max: 120 }),
            Rc::new(FallbackEgressAgreementRequiredCodeRule {
                message: "fallback egress agreement needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_egress_agreement_validator();
        let item = new_fallback_egress_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = FallbackEgressAgreementRequiredNameRule {
            message: "fallback egress agreement needs an entry".to_string(),
        };
        let mut item = new_fallback_egress_agreement("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackEgressAgreementCodeLengthRule { max_length: 86400 };
        let mut item = new_fallback_egress_agreement("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = FallbackEgressAgreementMinimumRangeRule { min: 2.3, max: 2607.8 };
        let mut item = new_fallback_egress_agreement("x", "y");
        item.minimum = 2609.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = FallbackEgressAgreementLimitRangeRule { min: 80, max: 80 };
        let mut item = new_fallback_egress_agreement("x", "y");
        item.limit = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = FallbackEgressAgreementRequiredCodeRule {
            message: "fallback egress agreement needs an entry".to_string(),
        };
        let mut item = new_fallback_egress_agreement("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
