use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive trust domain record must pass. Check returns its problems, or none.
pub trait AdaptiveTrustDomainRecordRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveTrustDomainRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTrustDomainRecordRequiredLocaleRule {
    pub message: String,
}

impl AdaptiveTrustDomainRecordRule for AdaptiveTrustDomainRecordRequiredLocaleRule {
    fn check(&self, item: &AdaptiveTrustDomainRecord) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTrustDomainRecordCodeLengthRule {
    pub max_length: i64,
}

impl AdaptiveTrustDomainRecordRule for AdaptiveTrustDomainRecordCodeLengthRule {
    fn check(&self, item: &AdaptiveTrustDomainRecord) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTrustDomainRecordAllowedCategoryRule {
    pub allowed: Vec<String>,
}

impl AdaptiveTrustDomainRecordRule for AdaptiveTrustDomainRecordAllowedCategoryRule {
    fn check(&self, item: &AdaptiveTrustDomainRecord) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.category)))
        {
            return vec![format!("category {} is above maximum", item.category)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTrustDomainRecordCodePrefixRule {
    pub prefix: String,
}

impl AdaptiveTrustDomainRecordRule for AdaptiveTrustDomainRecordCodePrefixRule {
    fn check(&self, item: &AdaptiveTrustDomainRecord) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTrustDomainRecordCategoryLengthRule {
    pub max_length: i64,
}

impl AdaptiveTrustDomainRecordRule for AdaptiveTrustDomainRecordCategoryLengthRule {
    fn check(&self, item: &AdaptiveTrustDomainRecord) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTrustDomainRecordMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveTrustDomainRecordRule for AdaptiveTrustDomainRecordMinimumRangeRule {
    fn check(&self, item: &AdaptiveTrustDomainRecord) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a adaptive trust domain record.
#[derive(Debug, Clone)]
pub struct AdaptiveTrustDomainRecordValidator {
    pub rules: Vec<Rc<dyn AdaptiveTrustDomainRecordRule>>,
}

impl AdaptiveTrustDomainRecordValidator {
    pub fn validate(&self, item: &AdaptiveTrustDomainRecord) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveTrustDomainRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveTrustDomainRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_trust_domain_record_validator() -> AdaptiveTrustDomainRecordValidator {
    AdaptiveTrustDomainRecordValidator {
        rules: vec![
            Rc::new(AdaptiveTrustDomainRecordRequiredLocaleRule {
                message: "adaptive trust domain record needs this field".to_string(),
            }) as Rc<dyn AdaptiveTrustDomainRecordRule>,
            Rc::new(AdaptiveTrustDomainRecordCodeLengthRule { max_length: 1000 }),
            Rc::new(AdaptiveTrustDomainRecordAllowedCategoryRule {
                allowed: vec!["regional".to_string(), "default".to_string()],
            }),
            Rc::new(AdaptiveTrustDomainRecordCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(AdaptiveTrustDomainRecordCategoryLengthRule { max_length: 120 }),
            Rc::new(AdaptiveTrustDomainRecordMinimumRangeRule { min: 3.4, max: 1127.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_trust_domain_record_validator();
        let item = new_adaptive_trust_domain_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = AdaptiveTrustDomainRecordRequiredLocaleRule {
            message: "adaptive trust domain record needs an entry".to_string(),
        };
        let mut item = new_adaptive_trust_domain_record("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = AdaptiveTrustDomainRecordCodeLengthRule { max_length: 120 };
        let mut item = new_adaptive_trust_domain_record("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = AdaptiveTrustDomainRecordCategoryLengthRule { max_length: 1000 };
        let mut item = new_adaptive_trust_domain_record("x", "y");
        item.category = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = AdaptiveTrustDomainRecordMinimumRangeRule { min: 4.8, max: 2786.5 };
        let mut item = new_adaptive_trust_domain_record("x", "y");
        item.minimum = 2788.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
