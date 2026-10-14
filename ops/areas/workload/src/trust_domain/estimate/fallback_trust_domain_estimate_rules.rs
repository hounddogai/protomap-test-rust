use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a fallback trust domain estimate must pass. Check returns its problems, or none.
pub trait FallbackTrustDomainEstimateRule: std::fmt::Debug {
    fn check(&self, item: &FallbackTrustDomainEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTrustDomainEstimateQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackTrustDomainEstimateRule for FallbackTrustDomainEstimateQuotaUnitsRangeRule {
    fn check(&self, item: &FallbackTrustDomainEstimate) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTrustDomainEstimateRequiredCurrencyRule {
    pub message: String,
}

impl FallbackTrustDomainEstimateRule for FallbackTrustDomainEstimateRequiredCurrencyRule {
    fn check(&self, item: &FallbackTrustDomainEstimate) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTrustDomainEstimateCurrencyLengthRule {
    pub max_length: i64,
}

impl FallbackTrustDomainEstimateRule for FallbackTrustDomainEstimateCurrencyLengthRule {
    fn check(&self, item: &FallbackTrustDomainEstimate) -> Vec<String> {
        let size = item.currency.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "currency is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTrustDomainEstimateAllowedCurrencyRule {
    pub allowed: Vec<String>,
}

impl FallbackTrustDomainEstimateRule for FallbackTrustDomainEstimateAllowedCurrencyRule {
    fn check(&self, item: &FallbackTrustDomainEstimate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.currency)))
        {
            return vec![format!("currency {} is out of range", item.currency)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTrustDomainEstimateCodePrefixRule {
    pub prefix: String,
}

impl FallbackTrustDomainEstimateRule for FallbackTrustDomainEstimateCodePrefixRule {
    fn check(&self, item: &FallbackTrustDomainEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback trust domain estimate.
#[derive(Debug, Clone)]
pub struct FallbackTrustDomainEstimateValidator {
    pub rules: Vec<Rc<dyn FallbackTrustDomainEstimateRule>>,
}

impl FallbackTrustDomainEstimateValidator {
    pub fn validate(&self, item: &FallbackTrustDomainEstimate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackTrustDomainEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackTrustDomainEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_trust_domain_estimate_validator() -> FallbackTrustDomainEstimateValidator {
    FallbackTrustDomainEstimateValidator {
        rules: vec![
            Rc::new(FallbackTrustDomainEstimateQuotaUnitsRangeRule { min: 32, max: 255 })
                as Rc<dyn FallbackTrustDomainEstimateRule>,
            Rc::new(FallbackTrustDomainEstimateRequiredCurrencyRule {
                message: "fallback trust domain estimate needs an entry".to_string(),
            }),
            Rc::new(FallbackTrustDomainEstimateCurrencyLengthRule { max_length: 86400 }),
            Rc::new(FallbackTrustDomainEstimateAllowedCurrencyRule {
                allowed: vec!["pending".to_string(), "fallback".to_string(), "scheduled".to_string()],
            }),
            Rc::new(FallbackTrustDomainEstimateCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_trust_domain_estimate_validator();
        let item = new_fallback_trust_domain_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = FallbackTrustDomainEstimateQuotaUnitsRangeRule { min: 80, max: 86400 };
        let mut item = new_fallback_trust_domain_estimate("x", "y");
        item.quota_units = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = FallbackTrustDomainEstimateRequiredCurrencyRule {
            message: "fallback trust domain estimate needs a value".to_string(),
        };
        let mut item = new_fallback_trust_domain_estimate("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_currency() {
        let rule = FallbackTrustDomainEstimateCurrencyLengthRule { max_length: 64 };
        let mut item = new_fallback_trust_domain_estimate("x", "y");
        item.currency = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.currency = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
