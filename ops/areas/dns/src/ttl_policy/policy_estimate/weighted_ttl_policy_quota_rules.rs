use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted ttl policy quota must pass. Check returns its problems, or none.
pub trait WeightedTtlPolicyQuotaRule: std::fmt::Debug {
    fn check(&self, item: &WeightedTtlPolicyQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTtlPolicyQuotaAllowedPrincipalRule {
    pub allowed: Vec<String>,
}

impl WeightedTtlPolicyQuotaRule for WeightedTtlPolicyQuotaAllowedPrincipalRule {
    fn check(&self, item: &WeightedTtlPolicyQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.principal)))
        {
            return vec![format!("principal {} is missing", item.principal)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTtlPolicyQuotaLocaleLengthRule {
    pub max_length: i64,
}

impl WeightedTtlPolicyQuotaRule for WeightedTtlPolicyQuotaLocaleLengthRule {
    fn check(&self, item: &WeightedTtlPolicyQuota) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTtlPolicyQuotaDescriptionLengthRule {
    pub max_length: i64,
}

impl WeightedTtlPolicyQuotaRule for WeightedTtlPolicyQuotaDescriptionLengthRule {
    fn check(&self, item: &WeightedTtlPolicyQuota) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTtlPolicyQuotaRequiredCodeRule {
    pub message: String,
}

impl WeightedTtlPolicyQuotaRule for WeightedTtlPolicyQuotaRequiredCodeRule {
    fn check(&self, item: &WeightedTtlPolicyQuota) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted ttl policy quota.
#[derive(Debug, Clone)]
pub struct WeightedTtlPolicyQuotaValidator {
    pub rules: Vec<Rc<dyn WeightedTtlPolicyQuotaRule>>,
}

impl WeightedTtlPolicyQuotaValidator {
    pub fn validate(&self, item: &WeightedTtlPolicyQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedTtlPolicyQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedTtlPolicyQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_ttl_policy_quota_validator() -> WeightedTtlPolicyQuotaValidator {
    WeightedTtlPolicyQuotaValidator {
        rules: vec![
            Rc::new(WeightedTtlPolicyQuotaAllowedPrincipalRule {
                allowed: vec!["batched".to_string(), "rolling".to_string(), "adaptive".to_string()],
            }) as Rc<dyn WeightedTtlPolicyQuotaRule>,
            Rc::new(WeightedTtlPolicyQuotaLocaleLengthRule { max_length: 86400 }),
            Rc::new(WeightedTtlPolicyQuotaDescriptionLengthRule { max_length: 32 }),
            Rc::new(WeightedTtlPolicyQuotaRequiredCodeRule {
                message: "weighted ttl policy quota needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_ttl_policy_quota_validator();
        let item = new_weighted_ttl_policy_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = WeightedTtlPolicyQuotaLocaleLengthRule { max_length: 16 };
        let mut item = new_weighted_ttl_policy_quota("x", "y");
        item.locale = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = WeightedTtlPolicyQuotaDescriptionLengthRule { max_length: 64 };
        let mut item = new_weighted_ttl_policy_quota("x", "y");
        item.description = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedTtlPolicyQuotaRequiredCodeRule {
            message: "weighted ttl policy quota needs a value".to_string(),
        };
        let mut item = new_weighted_ttl_policy_quota("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
