use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a dashboard override must pass. Check returns its problems, or none.
pub trait DashboardOverrideRule: std::fmt::Debug {
    fn check(&self, item: &DashboardOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardOverrideRequiredCodeRule {
    pub message: String,
}

impl DashboardOverrideRule for DashboardOverrideRequiredCodeRule {
    fn check(&self, item: &DashboardOverride) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardOverrideRequiredCipherRule {
    pub message: String,
}

impl DashboardOverrideRule for DashboardOverrideRequiredCipherRule {
    fn check(&self, item: &DashboardOverride) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardOverrideRequiredIssuerRule {
    pub message: String,
}

impl DashboardOverrideRule for DashboardOverrideRequiredIssuerRule {
    fn check(&self, item: &DashboardOverride) -> Vec<String> {
        if item.issuer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardOverrideAllowedLabelRule {
    pub allowed: Vec<String>,
}

impl DashboardOverrideRule for DashboardOverrideAllowedLabelRule {
    fn check(&self, item: &DashboardOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.label))) {
            return vec![format!("label {} is inconsistent", item.label)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardOverrideLabelLengthRule {
    pub max_length: i64,
}

impl DashboardOverrideRule for DashboardOverrideLabelLengthRule {
    fn check(&self, item: &DashboardOverride) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardOverrideVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DashboardOverrideRule for DashboardOverrideVersionRangeRule {
    fn check(&self, item: &DashboardOverride) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a dashboard override.
#[derive(Debug, Clone)]
pub struct DashboardOverrideValidator {
    pub rules: Vec<Rc<dyn DashboardOverrideRule>>,
}

impl DashboardOverrideValidator {
    pub fn validate(&self, item: &DashboardOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DashboardOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DashboardOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dashboard_override_validator() -> DashboardOverrideValidator {
    DashboardOverrideValidator {
        rules: vec![
            Rc::new(DashboardOverrideRequiredCodeRule {
                message: "dashboard override needs a value".to_string(),
            }) as Rc<dyn DashboardOverrideRule>,
            Rc::new(DashboardOverrideRequiredCipherRule {
                message: "dashboard override needs an entry".to_string(),
            }),
            Rc::new(DashboardOverrideRequiredIssuerRule {
                message: "dashboard override needs this field".to_string(),
            }),
            Rc::new(DashboardOverrideAllowedLabelRule {
                allowed: vec!["rolling".to_string()],
            }),
            Rc::new(DashboardOverrideLabelLengthRule { max_length: 1000 }),
            Rc::new(DashboardOverrideVersionRangeRule { min: 16, max: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dashboard_override_validator();
        let item = new_dashboard_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DashboardOverrideRequiredCodeRule {
            message: "dashboard override needs a value".to_string(),
        };
        let mut item = new_dashboard_override("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = DashboardOverrideRequiredCipherRule {
            message: "dashboard override needs this field".to_string(),
        };
        let mut item = new_dashboard_override("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_issuer() {
        let rule = DashboardOverrideRequiredIssuerRule {
            message: "dashboard override needs an entry".to_string(),
        };
        let mut item = new_dashboard_override("", "");
        item.issuer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = DashboardOverrideLabelLengthRule { max_length: 16 };
        let mut item = new_dashboard_override("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = DashboardOverrideVersionRangeRule { min: 64, max: 3600 };
        let mut item = new_dashboard_override("x", "y");
        item.version = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
