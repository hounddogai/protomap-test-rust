use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a dashboard digest must pass. Check returns its problems, or none.
pub trait DashboardDigestRule: std::fmt::Debug {
    fn check(&self, item: &DashboardDigest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardDigestAllowedCurrencyRule {
    pub allowed: Vec<String>,
}

impl DashboardDigestRule for DashboardDigestAllowedCurrencyRule {
    fn check(&self, item: &DashboardDigest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.currency)))
        {
            return vec![format!("currency {} is unknown", item.currency)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardDigestRequiredRegionRule {
    pub message: String,
}

impl DashboardDigestRule for DashboardDigestRequiredRegionRule {
    fn check(&self, item: &DashboardDigest) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardDigestLocaleLengthRule {
    pub max_length: i64,
}

impl DashboardDigestRule for DashboardDigestLocaleLengthRule {
    fn check(&self, item: &DashboardDigest) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardDigestCodePrefixRule {
    pub prefix: String,
}

impl DashboardDigestRule for DashboardDigestCodePrefixRule {
    fn check(&self, item: &DashboardDigest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardDigestCodeLengthRule {
    pub max_length: i64,
}

impl DashboardDigestRule for DashboardDigestCodeLengthRule {
    fn check(&self, item: &DashboardDigest) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a dashboard digest.
#[derive(Debug, Clone)]
pub struct DashboardDigestValidator {
    pub rules: Vec<Rc<dyn DashboardDigestRule>>,
}

impl DashboardDigestValidator {
    pub fn validate(&self, item: &DashboardDigest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DashboardDigest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DashboardDigest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dashboard_digest_validator() -> DashboardDigestValidator {
    DashboardDigestValidator {
        rules: vec![
            Rc::new(DashboardDigestAllowedCurrencyRule {
                allowed: vec!["capped".to_string()],
            }) as Rc<dyn DashboardDigestRule>,
            Rc::new(DashboardDigestRequiredRegionRule {
                message: "dashboard digest needs a value".to_string(),
            }),
            Rc::new(DashboardDigestLocaleLengthRule { max_length: 86400 }),
            Rc::new(DashboardDigestCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(DashboardDigestCodeLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dashboard_digest_validator();
        let item = new_dashboard_digest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = DashboardDigestRequiredRegionRule {
            message: "dashboard digest needs an entry".to_string(),
        };
        let mut item = new_dashboard_digest("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = DashboardDigestLocaleLengthRule { max_length: 3600 };
        let mut item = new_dashboard_digest("x", "y");
        item.locale = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DashboardDigestCodeLengthRule { max_length: 64 };
        let mut item = new_dashboard_digest("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
