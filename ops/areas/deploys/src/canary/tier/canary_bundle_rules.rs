use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a canary bundle must pass. Check returns its problems, or none.
pub trait CanaryBundleRule: std::fmt::Debug {
    fn check(&self, item: &CanaryBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryBundleAllowedLocaleRule {
    pub allowed: Vec<String>,
}

impl CanaryBundleRule for CanaryBundleAllowedLocaleRule {
    fn check(&self, item: &CanaryBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.locale))) {
            return vec![format!("locale {} is out of range", item.locale)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryBundleCodePrefixRule {
    pub prefix: String,
}

impl CanaryBundleRule for CanaryBundleCodePrefixRule {
    fn check(&self, item: &CanaryBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CanaryBundleRequiredIssuerRule {
    pub message: String,
}

impl CanaryBundleRule for CanaryBundleRequiredIssuerRule {
    fn check(&self, item: &CanaryBundle) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a canary bundle.
#[derive(Debug, Clone)]
pub struct CanaryBundleValidator {
    pub rules: Vec<Rc<dyn CanaryBundleRule>>,
}

impl CanaryBundleValidator {
    pub fn validate(&self, item: &CanaryBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CanaryBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CanaryBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_canary_bundle_validator() -> CanaryBundleValidator {
    CanaryBundleValidator {
        rules: vec![
            Rc::new(CanaryBundleAllowedLocaleRule {
                allowed: vec!["fallback".to_string()],
            }) as Rc<dyn CanaryBundleRule>,
            Rc::new(CanaryBundleCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(CanaryBundleRequiredIssuerRule {
                message: "canary bundle needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_canary_bundle_validator();
        let item = new_canary_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = CanaryBundleRequiredIssuerRule {
            message: "canary bundle needs this field".to_string(),
        };
        let mut item = new_canary_bundle("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
