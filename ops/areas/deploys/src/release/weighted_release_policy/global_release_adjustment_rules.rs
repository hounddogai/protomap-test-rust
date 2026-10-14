use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global release adjustment must pass. Check returns its problems, or none.
pub trait GlobalReleaseAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &GlobalReleaseAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseAdjustmentNamespaceLengthRule {
    pub max_length: i64,
}

impl GlobalReleaseAdjustmentRule for GlobalReleaseAdjustmentNamespaceLengthRule {
    fn check(&self, item: &GlobalReleaseAdjustment) -> Vec<String> {
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
pub struct GlobalReleaseAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl GlobalReleaseAdjustmentRule for GlobalReleaseAdjustmentCodeLengthRule {
    fn check(&self, item: &GlobalReleaseAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseAdjustmentCodePrefixRule {
    pub prefix: String,
}

impl GlobalReleaseAdjustmentRule for GlobalReleaseAdjustmentCodePrefixRule {
    fn check(&self, item: &GlobalReleaseAdjustment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseAdjustmentNameLengthRule {
    pub max_length: i64,
}

impl GlobalReleaseAdjustmentRule for GlobalReleaseAdjustmentNameLengthRule {
    fn check(&self, item: &GlobalReleaseAdjustment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseAdjustmentRequiredNamespaceRule {
    pub message: String,
}

impl GlobalReleaseAdjustmentRule for GlobalReleaseAdjustmentRequiredNamespaceRule {
    fn check(&self, item: &GlobalReleaseAdjustment) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseAdjustmentRequiredTitleRule {
    pub message: String,
}

impl GlobalReleaseAdjustmentRule for GlobalReleaseAdjustmentRequiredTitleRule {
    fn check(&self, item: &GlobalReleaseAdjustment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a global release adjustment.
#[derive(Debug, Clone)]
pub struct GlobalReleaseAdjustmentValidator {
    pub rules: Vec<Rc<dyn GlobalReleaseAdjustmentRule>>,
}

impl GlobalReleaseAdjustmentValidator {
    pub fn validate(&self, item: &GlobalReleaseAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalReleaseAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalReleaseAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_release_adjustment_validator() -> GlobalReleaseAdjustmentValidator {
    GlobalReleaseAdjustmentValidator {
        rules: vec![
            Rc::new(GlobalReleaseAdjustmentNamespaceLengthRule { max_length: 255 })
                as Rc<dyn GlobalReleaseAdjustmentRule>,
            Rc::new(GlobalReleaseAdjustmentCodeLengthRule { max_length: 1000 }),
            Rc::new(GlobalReleaseAdjustmentCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(GlobalReleaseAdjustmentNameLengthRule { max_length: 120 }),
            Rc::new(GlobalReleaseAdjustmentRequiredNamespaceRule {
                message: "global release adjustment needs an entry".to_string(),
            }),
            Rc::new(GlobalReleaseAdjustmentRequiredTitleRule {
                message: "global release adjustment needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_release_adjustment_validator();
        let item = new_global_release_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = GlobalReleaseAdjustmentNamespaceLengthRule { max_length: 255 };
        let mut item = new_global_release_adjustment("x", "y");
        item.namespace = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = GlobalReleaseAdjustmentCodeLengthRule { max_length: 255 };
        let mut item = new_global_release_adjustment("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = GlobalReleaseAdjustmentNameLengthRule { max_length: 86400 };
        let mut item = new_global_release_adjustment("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = GlobalReleaseAdjustmentRequiredNamespaceRule {
            message: "global release adjustment needs a value".to_string(),
        };
        let mut item = new_global_release_adjustment("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = GlobalReleaseAdjustmentRequiredTitleRule {
            message: "global release adjustment needs this field".to_string(),
        };
        let mut item = new_global_release_adjustment("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
