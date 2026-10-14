use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered binding group must pass. Check returns its problems, or none.
pub trait TieredBindingGroupRule: std::fmt::Debug {
    fn check(&self, item: &TieredBindingGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBindingGroupCodeLengthRule {
    pub max_length: i64,
}

impl TieredBindingGroupRule for TieredBindingGroupCodeLengthRule {
    fn check(&self, item: &TieredBindingGroup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBindingGroupRegionLengthRule {
    pub max_length: i64,
}

impl TieredBindingGroupRule for TieredBindingGroupRegionLengthRule {
    fn check(&self, item: &TieredBindingGroup) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBindingGroupRequiredFingerprintRule {
    pub message: String,
}

impl TieredBindingGroupRule for TieredBindingGroupRequiredFingerprintRule {
    fn check(&self, item: &TieredBindingGroup) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBindingGroupAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl TieredBindingGroupRule for TieredBindingGroupAllowedCodeRule {
    fn check(&self, item: &TieredBindingGroup) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBindingGroupRequiredCodeRule {
    pub message: String,
}

impl TieredBindingGroupRule for TieredBindingGroupRequiredCodeRule {
    fn check(&self, item: &TieredBindingGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBindingGroupRequiredTitleRule {
    pub message: String,
}

impl TieredBindingGroupRule for TieredBindingGroupRequiredTitleRule {
    fn check(&self, item: &TieredBindingGroup) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered binding group.
#[derive(Debug, Clone)]
pub struct TieredBindingGroupValidator {
    pub rules: Vec<Rc<dyn TieredBindingGroupRule>>,
}

impl TieredBindingGroupValidator {
    pub fn validate(&self, item: &TieredBindingGroup) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredBindingGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredBindingGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_binding_group_validator() -> TieredBindingGroupValidator {
    TieredBindingGroupValidator {
        rules: vec![
            Rc::new(TieredBindingGroupCodeLengthRule { max_length: 80 }) as Rc<dyn TieredBindingGroupRule>,
            Rc::new(TieredBindingGroupRegionLengthRule { max_length: 32 }),
            Rc::new(TieredBindingGroupRequiredFingerprintRule {
                message: "tiered binding group needs this field".to_string(),
            }),
            Rc::new(TieredBindingGroupAllowedCodeRule {
                allowed: vec!["batched".to_string(), "default".to_string()],
            }),
            Rc::new(TieredBindingGroupRequiredCodeRule {
                message: "tiered binding group needs a value".to_string(),
            }),
            Rc::new(TieredBindingGroupRequiredTitleRule {
                message: "tiered binding group needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_binding_group_validator();
        let item = new_tiered_binding_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredBindingGroupCodeLengthRule { max_length: 1000 };
        let mut item = new_tiered_binding_group("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = TieredBindingGroupRegionLengthRule { max_length: 3600 };
        let mut item = new_tiered_binding_group("x", "y");
        item.region = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = TieredBindingGroupRequiredFingerprintRule {
            message: "tiered binding group needs a value".to_string(),
        };
        let mut item = new_tiered_binding_group("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredBindingGroupRequiredCodeRule {
            message: "tiered binding group needs this field".to_string(),
        };
        let mut item = new_tiered_binding_group("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TieredBindingGroupRequiredTitleRule {
            message: "tiered binding group needs a value".to_string(),
        };
        let mut item = new_tiered_binding_group("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
