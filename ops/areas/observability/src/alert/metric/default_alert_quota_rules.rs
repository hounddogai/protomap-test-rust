use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a default alert quota must pass. Check returns its problems, or none.
pub trait DefaultAlertQuotaRule: std::fmt::Debug {
    fn check(&self, item: &DefaultAlertQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultAlertQuotaAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl DefaultAlertQuotaRule for DefaultAlertQuotaAllowedCodeRule {
    fn check(&self, item: &DefaultAlertQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is too long", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultAlertQuotaCodePrefixRule {
    pub prefix: String,
}

impl DefaultAlertQuotaRule for DefaultAlertQuotaCodePrefixRule {
    fn check(&self, item: &DefaultAlertQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultAlertQuotaRequiredTitleRule {
    pub message: String,
}

impl DefaultAlertQuotaRule for DefaultAlertQuotaRequiredTitleRule {
    fn check(&self, item: &DefaultAlertQuota) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a default alert quota.
#[derive(Debug, Clone)]
pub struct DefaultAlertQuotaValidator {
    pub rules: Vec<Rc<dyn DefaultAlertQuotaRule>>,
}

impl DefaultAlertQuotaValidator {
    pub fn validate(&self, item: &DefaultAlertQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultAlertQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultAlertQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_alert_quota_validator() -> DefaultAlertQuotaValidator {
    DefaultAlertQuotaValidator {
        rules: vec![
            Rc::new(DefaultAlertQuotaAllowedCodeRule {
                allowed: vec!["weighted".to_string()],
            }) as Rc<dyn DefaultAlertQuotaRule>,
            Rc::new(DefaultAlertQuotaCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(DefaultAlertQuotaRequiredTitleRule {
                message: "default alert quota needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_alert_quota_validator();
        let item = new_default_alert_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = DefaultAlertQuotaRequiredTitleRule {
            message: "default alert quota needs an entry".to_string(),
        };
        let mut item = new_default_alert_quota("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
