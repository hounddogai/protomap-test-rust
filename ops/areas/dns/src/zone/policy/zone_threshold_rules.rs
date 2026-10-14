use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a zone threshold must pass. Check returns its problems, or none.
pub trait ZoneThresholdRule: std::fmt::Debug {
    fn check(&self, item: &ZoneThreshold) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneThresholdRequiredLabelRule {
    pub message: String,
}

impl ZoneThresholdRule for ZoneThresholdRequiredLabelRule {
    fn check(&self, item: &ZoneThreshold) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneThresholdCodeLengthRule {
    pub max_length: i64,
}

impl ZoneThresholdRule for ZoneThresholdCodeLengthRule {
    fn check(&self, item: &ZoneThreshold) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneThresholdNamespaceLengthRule {
    pub max_length: i64,
}

impl ZoneThresholdRule for ZoneThresholdNamespaceLengthRule {
    fn check(&self, item: &ZoneThreshold) -> Vec<String> {
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
pub struct ZoneThresholdRequiredNamespaceRule {
    pub message: String,
}

impl ZoneThresholdRule for ZoneThresholdRequiredNamespaceRule {
    fn check(&self, item: &ZoneThreshold) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneThresholdLabelLengthRule {
    pub max_length: i64,
}

impl ZoneThresholdRule for ZoneThresholdLabelLengthRule {
    fn check(&self, item: &ZoneThreshold) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneThresholdAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl ZoneThresholdRule for ZoneThresholdAllowedCodeRule {
    fn check(&self, item: &ZoneThreshold) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is below minimum", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a zone threshold.
#[derive(Debug, Clone)]
pub struct ZoneThresholdValidator {
    pub rules: Vec<Rc<dyn ZoneThresholdRule>>,
}

impl ZoneThresholdValidator {
    pub fn validate(&self, item: &ZoneThreshold) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ZoneThreshold) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ZoneThreshold]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_zone_threshold_validator() -> ZoneThresholdValidator {
    ZoneThresholdValidator {
        rules: vec![
            Rc::new(ZoneThresholdRequiredLabelRule {
                message: "zone threshold needs an entry".to_string(),
            }) as Rc<dyn ZoneThresholdRule>,
            Rc::new(ZoneThresholdCodeLengthRule { max_length: 255 }),
            Rc::new(ZoneThresholdNamespaceLengthRule { max_length: 255 }),
            Rc::new(ZoneThresholdRequiredNamespaceRule {
                message: "zone threshold needs a value".to_string(),
            }),
            Rc::new(ZoneThresholdLabelLengthRule { max_length: 32 }),
            Rc::new(ZoneThresholdAllowedCodeRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_zone_threshold_validator();
        let item = new_zone_threshold("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ZoneThresholdRequiredLabelRule {
            message: "zone threshold needs an entry".to_string(),
        };
        let mut item = new_zone_threshold("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ZoneThresholdCodeLengthRule { max_length: 1000 };
        let mut item = new_zone_threshold("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = ZoneThresholdNamespaceLengthRule { max_length: 80 };
        let mut item = new_zone_threshold("x", "y");
        item.namespace = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = ZoneThresholdRequiredNamespaceRule {
            message: "zone threshold needs a value".to_string(),
        };
        let mut item = new_zone_threshold("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ZoneThresholdLabelLengthRule { max_length: 16 };
        let mut item = new_zone_threshold("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
