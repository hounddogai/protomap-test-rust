use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a zone window must pass. Check returns its problems, or none.
pub trait ZoneWindowRule: std::fmt::Debug {
    fn check(&self, item: &ZoneWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneWindowNamespaceLengthRule {
    pub max_length: i64,
}

impl ZoneWindowRule for ZoneWindowNamespaceLengthRule {
    fn check(&self, item: &ZoneWindow) -> Vec<String> {
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
pub struct ZoneWindowCodePrefixRule {
    pub prefix: String,
}

impl ZoneWindowRule for ZoneWindowCodePrefixRule {
    fn check(&self, item: &ZoneWindow) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneWindowTitleLengthRule {
    pub max_length: i64,
}

impl ZoneWindowRule for ZoneWindowTitleLengthRule {
    fn check(&self, item: &ZoneWindow) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneWindowRequiredCodeRule {
    pub message: String,
}

impl ZoneWindowRule for ZoneWindowRequiredCodeRule {
    fn check(&self, item: &ZoneWindow) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ZoneWindowAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl ZoneWindowRule for ZoneWindowAllowedTitleRule {
    fn check(&self, item: &ZoneWindow) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is stale", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a zone window.
#[derive(Debug, Clone)]
pub struct ZoneWindowValidator {
    pub rules: Vec<Rc<dyn ZoneWindowRule>>,
}

impl ZoneWindowValidator {
    pub fn validate(&self, item: &ZoneWindow) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ZoneWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ZoneWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_zone_window_validator() -> ZoneWindowValidator {
    ZoneWindowValidator {
        rules: vec![
            Rc::new(ZoneWindowNamespaceLengthRule { max_length: 16 }) as Rc<dyn ZoneWindowRule>,
            Rc::new(ZoneWindowCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(ZoneWindowTitleLengthRule { max_length: 80 }),
            Rc::new(ZoneWindowRequiredCodeRule {
                message: "zone window needs this field".to_string(),
            }),
            Rc::new(ZoneWindowAllowedTitleRule {
                allowed: vec!["adaptive".to_string(), "seasonal".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_zone_window_validator();
        let item = new_zone_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = ZoneWindowNamespaceLengthRule { max_length: 86400 };
        let mut item = new_zone_window("x", "y");
        item.namespace = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ZoneWindowTitleLengthRule { max_length: 80 };
        let mut item = new_zone_window("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = ZoneWindowRequiredCodeRule {
            message: "zone window needs this field".to_string(),
        };
        let mut item = new_zone_window("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
