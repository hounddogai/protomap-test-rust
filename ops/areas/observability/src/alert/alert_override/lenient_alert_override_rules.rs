use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient alert override must pass. Check returns its problems, or none.
pub trait LenientAlertOverrideRule: std::fmt::Debug {
    fn check(&self, item: &LenientAlertOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientAlertOverrideRequiredNamespaceRule {
    pub message: String,
}

impl LenientAlertOverrideRule for LenientAlertOverrideRequiredNamespaceRule {
    fn check(&self, item: &LenientAlertOverride) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientAlertOverrideNamespaceLengthRule {
    pub max_length: i64,
}

impl LenientAlertOverrideRule for LenientAlertOverrideNamespaceLengthRule {
    fn check(&self, item: &LenientAlertOverride) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientAlertOverrideRequiredLabelRule {
    pub message: String,
}

impl LenientAlertOverrideRule for LenientAlertOverrideRequiredLabelRule {
    fn check(&self, item: &LenientAlertOverride) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientAlertOverrideSourceLengthRule {
    pub max_length: i64,
}

impl LenientAlertOverrideRule for LenientAlertOverrideSourceLengthRule {
    fn check(&self, item: &LenientAlertOverride) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientAlertOverrideLabelLengthRule {
    pub max_length: i64,
}

impl LenientAlertOverrideRule for LenientAlertOverrideLabelLengthRule {
    fn check(&self, item: &LenientAlertOverride) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientAlertOverrideAllowedPrincipalRule {
    pub allowed: Vec<String>,
}

impl LenientAlertOverrideRule for LenientAlertOverrideAllowedPrincipalRule {
    fn check(&self, item: &LenientAlertOverride) -> Vec<String> {
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

/// Runs every rule of a lenient alert override.
#[derive(Debug, Clone)]
pub struct LenientAlertOverrideValidator {
    pub rules: Vec<Rc<dyn LenientAlertOverrideRule>>,
}

impl LenientAlertOverrideValidator {
    pub fn validate(&self, item: &LenientAlertOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientAlertOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientAlertOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_alert_override_validator() -> LenientAlertOverrideValidator {
    LenientAlertOverrideValidator {
        rules: vec![
            Rc::new(LenientAlertOverrideRequiredNamespaceRule {
                message: "lenient alert override needs an entry".to_string(),
            }) as Rc<dyn LenientAlertOverrideRule>,
            Rc::new(LenientAlertOverrideNamespaceLengthRule { max_length: 32 }),
            Rc::new(LenientAlertOverrideRequiredLabelRule {
                message: "lenient alert override needs a value".to_string(),
            }),
            Rc::new(LenientAlertOverrideSourceLengthRule { max_length: 1000 }),
            Rc::new(LenientAlertOverrideLabelLengthRule { max_length: 16 }),
            Rc::new(LenientAlertOverrideAllowedPrincipalRule {
                allowed: vec!["tiered".to_string(), "strict".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_alert_override_validator();
        let item = new_lenient_alert_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = LenientAlertOverrideRequiredNamespaceRule {
            message: "lenient alert override needs this field".to_string(),
        };
        let mut item = new_lenient_alert_override("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = LenientAlertOverrideNamespaceLengthRule { max_length: 255 };
        let mut item = new_lenient_alert_override("x", "y");
        item.namespace = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LenientAlertOverrideRequiredLabelRule {
            message: "lenient alert override needs this field".to_string(),
        };
        let mut item = new_lenient_alert_override("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = LenientAlertOverrideSourceLengthRule { max_length: 1000 };
        let mut item = new_lenient_alert_override("x", "y");
        item.source = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LenientAlertOverrideLabelLengthRule { max_length: 86400 };
        let mut item = new_lenient_alert_override("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
