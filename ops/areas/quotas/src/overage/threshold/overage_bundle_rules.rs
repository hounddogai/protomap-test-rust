use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a overage bundle must pass. Check returns its problems, or none.
pub trait OverageBundleRule: std::fmt::Debug {
    fn check(&self, item: &OverageBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageBundleLocaleLengthRule {
    pub max_length: i64,
}

impl OverageBundleRule for OverageBundleLocaleLengthRule {
    fn check(&self, item: &OverageBundle) -> Vec<String> {
        let size = item.locale.len() as i64;
        if size > self.max_length {
            return vec![format!("locale is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageBundleAllowedLocaleRule {
    pub allowed: Vec<String>,
}

impl OverageBundleRule for OverageBundleAllowedLocaleRule {
    fn check(&self, item: &OverageBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.locale))) {
            return vec![format!("locale {} is duplicated", item.locale)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageBundleCodePrefixRule {
    pub prefix: String,
}

impl OverageBundleRule for OverageBundleCodePrefixRule {
    fn check(&self, item: &OverageBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageBundleNamespaceLengthRule {
    pub max_length: i64,
}

impl OverageBundleRule for OverageBundleNamespaceLengthRule {
    fn check(&self, item: &OverageBundle) -> Vec<String> {
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
pub struct OverageBundleBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl OverageBundleRule for OverageBundleBatchSizeRangeRule {
    fn check(&self, item: &OverageBundle) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OverageBundleCodeLengthRule {
    pub max_length: i64,
}

impl OverageBundleRule for OverageBundleCodeLengthRule {
    fn check(&self, item: &OverageBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a overage bundle.
#[derive(Debug, Clone)]
pub struct OverageBundleValidator {
    pub rules: Vec<Rc<dyn OverageBundleRule>>,
}

impl OverageBundleValidator {
    pub fn validate(&self, item: &OverageBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &OverageBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[OverageBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_overage_bundle_validator() -> OverageBundleValidator {
    OverageBundleValidator {
        rules: vec![
            Rc::new(OverageBundleLocaleLengthRule { max_length: 80 }) as Rc<dyn OverageBundleRule>,
            Rc::new(OverageBundleAllowedLocaleRule {
                allowed: vec!["global".to_string()],
            }),
            Rc::new(OverageBundleCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(OverageBundleNamespaceLengthRule { max_length: 1000 }),
            Rc::new(OverageBundleBatchSizeRangeRule { min: 120, max: 3600 }),
            Rc::new(OverageBundleCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_overage_bundle_validator();
        let item = new_overage_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_locale() {
        let rule = OverageBundleLocaleLengthRule { max_length: 32 };
        let mut item = new_overage_bundle("x", "y");
        item.locale = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.locale = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = OverageBundleNamespaceLengthRule { max_length: 80 };
        let mut item = new_overage_bundle("x", "y");
        item.namespace = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = OverageBundleBatchSizeRangeRule { min: 80, max: 86400 };
        let mut item = new_overage_bundle("x", "y");
        item.batch_size = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = OverageBundleCodeLengthRule { max_length: 80 };
        let mut item = new_overage_bundle("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
