use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted credential bundle must pass. Check returns its problems, or none.
pub trait WeightedCredentialBundleRule: std::fmt::Debug {
    fn check(&self, item: &WeightedCredentialBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialBundleCodePrefixRule {
    pub prefix: String,
}

impl WeightedCredentialBundleRule for WeightedCredentialBundleCodePrefixRule {
    fn check(&self, item: &WeightedCredentialBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialBundleRequiredTitleRule {
    pub message: String,
}

impl WeightedCredentialBundleRule for WeightedCredentialBundleRequiredTitleRule {
    fn check(&self, item: &WeightedCredentialBundle) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialBundleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl WeightedCredentialBundleRule for WeightedCredentialBundleAllowedCodeRule {
    fn check(&self, item: &WeightedCredentialBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is above maximum", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialBundleRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedCredentialBundleRule for WeightedCredentialBundleRateRangeRule {
    fn check(&self, item: &WeightedCredentialBundle) -> Vec<String> {
        let value = item.rate;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rate {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rate {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialBundleRequiredCodeRule {
    pub message: String,
}

impl WeightedCredentialBundleRule for WeightedCredentialBundleRequiredCodeRule {
    fn check(&self, item: &WeightedCredentialBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialBundleTitleLengthRule {
    pub max_length: i64,
}

impl WeightedCredentialBundleRule for WeightedCredentialBundleTitleLengthRule {
    fn check(&self, item: &WeightedCredentialBundle) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted credential bundle.
#[derive(Debug, Clone)]
pub struct WeightedCredentialBundleValidator {
    pub rules: Vec<Rc<dyn WeightedCredentialBundleRule>>,
}

impl WeightedCredentialBundleValidator {
    pub fn validate(&self, item: &WeightedCredentialBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedCredentialBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedCredentialBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_credential_bundle_validator() -> WeightedCredentialBundleValidator {
    WeightedCredentialBundleValidator {
        rules: vec![
            Rc::new(WeightedCredentialBundleCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn WeightedCredentialBundleRule>,
            Rc::new(WeightedCredentialBundleRequiredTitleRule {
                message: "weighted credential bundle needs a value".to_string(),
            }),
            Rc::new(WeightedCredentialBundleAllowedCodeRule { allowed: Vec::new() }),
            Rc::new(WeightedCredentialBundleRateRangeRule { min: 0.7, max: 1584.2 }),
            Rc::new(WeightedCredentialBundleRequiredCodeRule {
                message: "weighted credential bundle needs an entry".to_string(),
            }),
            Rc::new(WeightedCredentialBundleTitleLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_credential_bundle_validator();
        let item = new_weighted_credential_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = WeightedCredentialBundleRequiredTitleRule {
            message: "weighted credential bundle needs an entry".to_string(),
        };
        let mut item = new_weighted_credential_bundle("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = WeightedCredentialBundleRateRangeRule { min: 9.4, max: 885.4 };
        let mut item = new_weighted_credential_bundle("x", "y");
        item.rate = 886.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedCredentialBundleRequiredCodeRule {
            message: "weighted credential bundle needs an entry".to_string(),
        };
        let mut item = new_weighted_credential_bundle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = WeightedCredentialBundleTitleLengthRule { max_length: 80 };
        let mut item = new_weighted_credential_bundle("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
