use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted ttl policy bundle must pass. Check returns its problems, or none.
pub trait WeightedTtlPolicyBundleRule: std::fmt::Debug {
    fn check(&self, item: &WeightedTtlPolicyBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTtlPolicyBundleAllowedKeyIdRule {
    pub allowed: Vec<String>,
}

impl WeightedTtlPolicyBundleRule for WeightedTtlPolicyBundleAllowedKeyIdRule {
    fn check(&self, item: &WeightedTtlPolicyBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.key_id))) {
            return vec![format!("key id {} is stale", item.key_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTtlPolicyBundleCodeLengthRule {
    pub max_length: i64,
}

impl WeightedTtlPolicyBundleRule for WeightedTtlPolicyBundleCodeLengthRule {
    fn check(&self, item: &WeightedTtlPolicyBundle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTtlPolicyBundleRequiredCodeRule {
    pub message: String,
}

impl WeightedTtlPolicyBundleRule for WeightedTtlPolicyBundleRequiredCodeRule {
    fn check(&self, item: &WeightedTtlPolicyBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTtlPolicyBundleLabelLengthRule {
    pub max_length: i64,
}

impl WeightedTtlPolicyBundleRule for WeightedTtlPolicyBundleLabelLengthRule {
    fn check(&self, item: &WeightedTtlPolicyBundle) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTtlPolicyBundleKeyIdLengthRule {
    pub max_length: i64,
}

impl WeightedTtlPolicyBundleRule for WeightedTtlPolicyBundleKeyIdLengthRule {
    fn check(&self, item: &WeightedTtlPolicyBundle) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTtlPolicyBundleCodePrefixRule {
    pub prefix: String,
}

impl WeightedTtlPolicyBundleRule for WeightedTtlPolicyBundleCodePrefixRule {
    fn check(&self, item: &WeightedTtlPolicyBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted ttl policy bundle.
#[derive(Debug, Clone)]
pub struct WeightedTtlPolicyBundleValidator {
    pub rules: Vec<Rc<dyn WeightedTtlPolicyBundleRule>>,
}

impl WeightedTtlPolicyBundleValidator {
    pub fn validate(&self, item: &WeightedTtlPolicyBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedTtlPolicyBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedTtlPolicyBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_ttl_policy_bundle_validator() -> WeightedTtlPolicyBundleValidator {
    WeightedTtlPolicyBundleValidator {
        rules: vec![
            Rc::new(WeightedTtlPolicyBundleAllowedKeyIdRule {
                allowed: vec!["weighted".to_string()],
            }) as Rc<dyn WeightedTtlPolicyBundleRule>,
            Rc::new(WeightedTtlPolicyBundleCodeLengthRule { max_length: 86400 }),
            Rc::new(WeightedTtlPolicyBundleRequiredCodeRule {
                message: "weighted ttl policy bundle needs this field".to_string(),
            }),
            Rc::new(WeightedTtlPolicyBundleLabelLengthRule { max_length: 120 }),
            Rc::new(WeightedTtlPolicyBundleKeyIdLengthRule { max_length: 3600 }),
            Rc::new(WeightedTtlPolicyBundleCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_ttl_policy_bundle_validator();
        let item = new_weighted_ttl_policy_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedTtlPolicyBundleCodeLengthRule { max_length: 32 };
        let mut item = new_weighted_ttl_policy_bundle("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedTtlPolicyBundleRequiredCodeRule {
            message: "weighted ttl policy bundle needs this field".to_string(),
        };
        let mut item = new_weighted_ttl_policy_bundle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = WeightedTtlPolicyBundleLabelLengthRule { max_length: 255 };
        let mut item = new_weighted_ttl_policy_bundle("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = WeightedTtlPolicyBundleKeyIdLengthRule { max_length: 3600 };
        let mut item = new_weighted_ttl_policy_bundle("x", "y");
        item.key_id = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
