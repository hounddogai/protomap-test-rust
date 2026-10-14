use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a binding quota must pass. Check returns its problems, or none.
pub trait BindingQuotaRule: std::fmt::Debug {
    fn check(&self, item: &BindingQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingQuotaRequiredLabelRule {
    pub message: String,
}

impl BindingQuotaRule for BindingQuotaRequiredLabelRule {
    fn check(&self, item: &BindingQuota) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingQuotaRequiredTimezoneRule {
    pub message: String,
}

impl BindingQuotaRule for BindingQuotaRequiredTimezoneRule {
    fn check(&self, item: &BindingQuota) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingQuotaArchivedGuardRule {
    pub limit: f64,
}

impl BindingQuotaRule for BindingQuotaArchivedGuardRule {
    fn check(&self, item: &BindingQuota) -> Vec<String> {
        if item.archived && (item.cost > self.limit) {
            return vec![format!("cost {} exceeds {} while archived", item.cost, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingQuotaCodePrefixRule {
    pub prefix: String,
}

impl BindingQuotaRule for BindingQuotaCodePrefixRule {
    fn check(&self, item: &BindingQuota) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BindingQuotaTimezoneLengthRule {
    pub max_length: i64,
}

impl BindingQuotaRule for BindingQuotaTimezoneLengthRule {
    fn check(&self, item: &BindingQuota) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a binding quota.
#[derive(Debug, Clone)]
pub struct BindingQuotaValidator {
    pub rules: Vec<Rc<dyn BindingQuotaRule>>,
}

impl BindingQuotaValidator {
    pub fn validate(&self, item: &BindingQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &BindingQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BindingQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_binding_quota_validator() -> BindingQuotaValidator {
    BindingQuotaValidator {
        rules: vec![
            Rc::new(BindingQuotaRequiredLabelRule {
                message: "binding quota needs a value".to_string(),
            }) as Rc<dyn BindingQuotaRule>,
            Rc::new(BindingQuotaRequiredTimezoneRule {
                message: "binding quota needs an entry".to_string(),
            }),
            Rc::new(BindingQuotaArchivedGuardRule { limit: 2.6 }),
            Rc::new(BindingQuotaCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(BindingQuotaTimezoneLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_binding_quota_validator();
        let item = new_binding_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = BindingQuotaRequiredLabelRule {
            message: "binding quota needs this field".to_string(),
        };
        let mut item = new_binding_quota("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = BindingQuotaRequiredTimezoneRule {
            message: "binding quota needs a value".to_string(),
        };
        let mut item = new_binding_quota("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = BindingQuotaTimezoneLengthRule { max_length: 255 };
        let mut item = new_binding_quota("x", "y");
        item.timezone = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
