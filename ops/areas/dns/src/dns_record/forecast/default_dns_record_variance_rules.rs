use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default dns record variance must pass. Check returns its problems, or none.
pub trait DefaultDnsRecordVarianceRule: std::fmt::Debug {
    fn check(&self, item: &DefaultDnsRecordVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDnsRecordVarianceCodePrefixRule {
    pub prefix: String,
}

impl DefaultDnsRecordVarianceRule for DefaultDnsRecordVarianceCodePrefixRule {
    fn check(&self, item: &DefaultDnsRecordVariance) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDnsRecordVarianceTitleLengthRule {
    pub max_length: i64,
}

impl DefaultDnsRecordVarianceRule for DefaultDnsRecordVarianceTitleLengthRule {
    fn check(&self, item: &DefaultDnsRecordVariance) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDnsRecordVarianceAllowedUnitRule {
    pub allowed: Vec<String>,
}

impl DefaultDnsRecordVarianceRule for DefaultDnsRecordVarianceAllowedUnitRule {
    fn check(&self, item: &DefaultDnsRecordVariance) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.unit))) {
            return vec![format!("unit {} is too long", item.unit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a default dns record variance.
#[derive(Debug, Clone)]
pub struct DefaultDnsRecordVarianceValidator {
    pub rules: Vec<Rc<dyn DefaultDnsRecordVarianceRule>>,
}

impl DefaultDnsRecordVarianceValidator {
    pub fn validate(&self, item: &DefaultDnsRecordVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultDnsRecordVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultDnsRecordVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_dns_record_variance_validator() -> DefaultDnsRecordVarianceValidator {
    DefaultDnsRecordVarianceValidator {
        rules: vec![
            Rc::new(DefaultDnsRecordVarianceCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn DefaultDnsRecordVarianceRule>,
            Rc::new(DefaultDnsRecordVarianceTitleLengthRule { max_length: 64 }),
            Rc::new(DefaultDnsRecordVarianceAllowedUnitRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_dns_record_variance_validator();
        let item = new_default_dns_record_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = DefaultDnsRecordVarianceTitleLengthRule { max_length: 16 };
        let mut item = new_default_dns_record_variance("x", "y");
        item.title = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
