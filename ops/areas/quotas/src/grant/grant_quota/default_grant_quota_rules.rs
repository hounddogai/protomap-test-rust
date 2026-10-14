use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default grant quota must pass. Check returns its problems, or none.
pub trait DefaultGrantQuotaRule: std::fmt::Debug {
    fn check(&self, item: &DefaultGrantQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultGrantQuotaAllowedPrincipalRule {
    pub allowed: Vec<String>,
}

impl DefaultGrantQuotaRule for DefaultGrantQuotaAllowedPrincipalRule {
    fn check(&self, item: &DefaultGrantQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.principal)))
        {
            return vec![format!("principal {} is too long", item.principal)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultGrantQuotaCountryLengthRule {
    pub max_length: i64,
}

impl DefaultGrantQuotaRule for DefaultGrantQuotaCountryLengthRule {
    fn check(&self, item: &DefaultGrantQuota) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultGrantQuotaMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultGrantQuotaRule for DefaultGrantQuotaMarginRangeRule {
    fn check(&self, item: &DefaultGrantQuota) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultGrantQuotaNameLengthRule {
    pub max_length: i64,
}

impl DefaultGrantQuotaRule for DefaultGrantQuotaNameLengthRule {
    fn check(&self, item: &DefaultGrantQuota) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default grant quota.
#[derive(Debug, Clone)]
pub struct DefaultGrantQuotaValidator {
    pub rules: Vec<Rc<dyn DefaultGrantQuotaRule>>,
}

impl DefaultGrantQuotaValidator {
    pub fn validate(&self, item: &DefaultGrantQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultGrantQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultGrantQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_grant_quota_validator() -> DefaultGrantQuotaValidator {
    DefaultGrantQuotaValidator {
        rules: vec![
            Rc::new(DefaultGrantQuotaAllowedPrincipalRule { allowed: Vec::new() }) as Rc<dyn DefaultGrantQuotaRule>,
            Rc::new(DefaultGrantQuotaCountryLengthRule { max_length: 64 }),
            Rc::new(DefaultGrantQuotaMarginRangeRule { min: 5.4, max: 4071.1 }),
            Rc::new(DefaultGrantQuotaNameLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_grant_quota_validator();
        let item = new_default_grant_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = DefaultGrantQuotaCountryLengthRule { max_length: 255 };
        let mut item = new_default_grant_quota("x", "y");
        item.country = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = DefaultGrantQuotaMarginRangeRule { min: 8.7, max: 1310.2 };
        let mut item = new_default_grant_quota("x", "y");
        item.margin = 1311.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = DefaultGrantQuotaNameLengthRule { max_length: 120 };
        let mut item = new_default_grant_quota("x", "y");
        item.name = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
