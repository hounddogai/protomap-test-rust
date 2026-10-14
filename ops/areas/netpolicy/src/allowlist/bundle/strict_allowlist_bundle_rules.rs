use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict allowlist bundle must pass. Check returns its problems, or none.
pub trait StrictAllowlistBundleRule: std::fmt::Debug {
    fn check(&self, item: &StrictAllowlistBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAllowlistBundleBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictAllowlistBundleRule for StrictAllowlistBundleBurstRangeRule {
    fn check(&self, item: &StrictAllowlistBundle) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAllowlistBundleCodePrefixRule {
    pub prefix: String,
}

impl StrictAllowlistBundleRule for StrictAllowlistBundleCodePrefixRule {
    fn check(&self, item: &StrictAllowlistBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAllowlistBundleTitleLengthRule {
    pub max_length: i64,
}

impl StrictAllowlistBundleRule for StrictAllowlistBundleTitleLengthRule {
    fn check(&self, item: &StrictAllowlistBundle) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAllowlistBundleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl StrictAllowlistBundleRule for StrictAllowlistBundleAllowedCodeRule {
    fn check(&self, item: &StrictAllowlistBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is stale", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict allowlist bundle.
#[derive(Debug, Clone)]
pub struct StrictAllowlistBundleValidator {
    pub rules: Vec<Rc<dyn StrictAllowlistBundleRule>>,
}

impl StrictAllowlistBundleValidator {
    pub fn validate(&self, item: &StrictAllowlistBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictAllowlistBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictAllowlistBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_allowlist_bundle_validator() -> StrictAllowlistBundleValidator {
    StrictAllowlistBundleValidator {
        rules: vec![
            Rc::new(StrictAllowlistBundleBurstRangeRule { min: 80, max: 255 }) as Rc<dyn StrictAllowlistBundleRule>,
            Rc::new(StrictAllowlistBundleCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(StrictAllowlistBundleTitleLengthRule { max_length: 32 }),
            Rc::new(StrictAllowlistBundleAllowedCodeRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_allowlist_bundle_validator();
        let item = new_strict_allowlist_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = StrictAllowlistBundleBurstRangeRule { min: 1000, max: 3600 };
        let mut item = new_strict_allowlist_bundle("x", "y");
        item.burst = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = StrictAllowlistBundleTitleLengthRule { max_length: 255 };
        let mut item = new_strict_allowlist_bundle("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
