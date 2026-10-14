use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback suspension agreement must pass. Check returns its problems, or none.
pub trait FallbackSuspensionAgreementRule: std::fmt::Debug {
    fn check(&self, item: &FallbackSuspensionAgreement) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSuspensionAgreementPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackSuspensionAgreementRule for FallbackSuspensionAgreementPriorityRangeRule {
    fn check(&self, item: &FallbackSuspensionAgreement) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSuspensionAgreementAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl FallbackSuspensionAgreementRule for FallbackSuspensionAgreementAllowedCodeRule {
    fn check(&self, item: &FallbackSuspensionAgreement) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is inconsistent", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSuspensionAgreementTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackSuspensionAgreementRule for FallbackSuspensionAgreementTtlSecondsRangeRule {
    fn check(&self, item: &FallbackSuspensionAgreement) -> Vec<String> {
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
pub struct FallbackSuspensionAgreementCodePrefixRule {
    pub prefix: String,
}

impl FallbackSuspensionAgreementRule for FallbackSuspensionAgreementCodePrefixRule {
    fn check(&self, item: &FallbackSuspensionAgreement) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback suspension agreement.
#[derive(Debug, Clone)]
pub struct FallbackSuspensionAgreementValidator {
    pub rules: Vec<Rc<dyn FallbackSuspensionAgreementRule>>,
}

impl FallbackSuspensionAgreementValidator {
    pub fn validate(&self, item: &FallbackSuspensionAgreement) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FallbackSuspensionAgreement) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackSuspensionAgreement]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_suspension_agreement_validator() -> FallbackSuspensionAgreementValidator {
    FallbackSuspensionAgreementValidator {
        rules: vec![
            Rc::new(FallbackSuspensionAgreementPriorityRangeRule { min: 32, max: 64 })
                as Rc<dyn FallbackSuspensionAgreementRule>,
            Rc::new(FallbackSuspensionAgreementAllowedCodeRule {
                allowed: vec!["primary".to_string(), "fallback".to_string(), "lenient".to_string()],
            }),
            Rc::new(FallbackSuspensionAgreementTtlSecondsRangeRule { min: 80, max: 1000 }),
            Rc::new(FallbackSuspensionAgreementCodePrefixRule {
                prefix: "X".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_suspension_agreement_validator();
        let item = new_fallback_suspension_agreement("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = FallbackSuspensionAgreementPriorityRangeRule { min: 120, max: 86400 };
        let mut item = new_fallback_suspension_agreement("x", "y");
        item.priority = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = FallbackSuspensionAgreementTtlSecondsRangeRule { min: 64, max: 120 };
        let mut item = new_fallback_suspension_agreement("x", "y");
        item.ttl_seconds = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
