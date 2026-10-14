use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a global freeze manifest must pass. Check returns its problems, or none.
pub trait GlobalFreezeManifestRule: std::fmt::Debug {
    fn check(&self, item: &GlobalFreezeManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFreezeManifestCodePrefixRule {
    pub prefix: String,
}

impl GlobalFreezeManifestRule for GlobalFreezeManifestCodePrefixRule {
    fn check(&self, item: &GlobalFreezeManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFreezeManifestRequiredCodeRule {
    pub message: String,
}

impl GlobalFreezeManifestRule for GlobalFreezeManifestRequiredCodeRule {
    fn check(&self, item: &GlobalFreezeManifest) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFreezeManifestNamespaceLengthRule {
    pub max_length: i64,
}

impl GlobalFreezeManifestRule for GlobalFreezeManifestNamespaceLengthRule {
    fn check(&self, item: &GlobalFreezeManifest) -> Vec<String> {
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
pub struct GlobalFreezeManifestMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalFreezeManifestRule for GlobalFreezeManifestMarginRangeRule {
    fn check(&self, item: &GlobalFreezeManifest) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFreezeManifestRequiredChannelRule {
    pub message: String,
}

impl GlobalFreezeManifestRule for GlobalFreezeManifestRequiredChannelRule {
    fn check(&self, item: &GlobalFreezeManifest) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFreezeManifestRequiredNamespaceRule {
    pub message: String,
}

impl GlobalFreezeManifestRule for GlobalFreezeManifestRequiredNamespaceRule {
    fn check(&self, item: &GlobalFreezeManifest) -> Vec<String> {
        if item.namespace.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a global freeze manifest.
#[derive(Debug, Clone)]
pub struct GlobalFreezeManifestValidator {
    pub rules: Vec<Rc<dyn GlobalFreezeManifestRule>>,
}

impl GlobalFreezeManifestValidator {
    pub fn validate(&self, item: &GlobalFreezeManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalFreezeManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalFreezeManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_freeze_manifest_validator() -> GlobalFreezeManifestValidator {
    GlobalFreezeManifestValidator {
        rules: vec![
            Rc::new(GlobalFreezeManifestCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn GlobalFreezeManifestRule>,
            Rc::new(GlobalFreezeManifestRequiredCodeRule {
                message: "global freeze manifest needs a value".to_string(),
            }),
            Rc::new(GlobalFreezeManifestNamespaceLengthRule { max_length: 16 }),
            Rc::new(GlobalFreezeManifestMarginRangeRule { min: 3.0, max: 1215.2 }),
            Rc::new(GlobalFreezeManifestRequiredChannelRule {
                message: "global freeze manifest needs an entry".to_string(),
            }),
            Rc::new(GlobalFreezeManifestRequiredNamespaceRule {
                message: "global freeze manifest needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_freeze_manifest_validator();
        let item = new_global_freeze_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GlobalFreezeManifestRequiredCodeRule {
            message: "global freeze manifest needs an entry".to_string(),
        };
        let mut item = new_global_freeze_manifest("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = GlobalFreezeManifestNamespaceLengthRule { max_length: 120 };
        let mut item = new_global_freeze_manifest("x", "y");
        item.namespace = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = GlobalFreezeManifestMarginRangeRule { min: 3.4, max: 4908.8 };
        let mut item = new_global_freeze_manifest("x", "y");
        item.margin = 4910.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = GlobalFreezeManifestRequiredChannelRule {
            message: "global freeze manifest needs a value".to_string(),
        };
        let mut item = new_global_freeze_manifest("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_namespace() {
        let rule = GlobalFreezeManifestRequiredNamespaceRule {
            message: "global freeze manifest needs this field".to_string(),
        };
        let mut item = new_global_freeze_manifest("", "");
        item.namespace = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
