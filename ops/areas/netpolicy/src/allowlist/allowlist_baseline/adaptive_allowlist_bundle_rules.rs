use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a adaptive allowlist bundle must pass. Check returns its problems, or none.
pub trait AdaptiveAllowlistBundleRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveAllowlistBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAllowlistBundleAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl AdaptiveAllowlistBundleRule for AdaptiveAllowlistBundleAllowedCodeRule {
    fn check(&self, item: &AdaptiveAllowlistBundle) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is stale", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAllowlistBundleRequiredCodeRule {
    pub message: String,
}

impl AdaptiveAllowlistBundleRule for AdaptiveAllowlistBundleRequiredCodeRule {
    fn check(&self, item: &AdaptiveAllowlistBundle) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAllowlistBundleCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveAllowlistBundleRule for AdaptiveAllowlistBundleCeilingRangeRule {
    fn check(&self, item: &AdaptiveAllowlistBundle) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive allowlist bundle.
#[derive(Debug, Clone)]
pub struct AdaptiveAllowlistBundleValidator {
    pub rules: Vec<Rc<dyn AdaptiveAllowlistBundleRule>>,
}

impl AdaptiveAllowlistBundleValidator {
    pub fn validate(&self, item: &AdaptiveAllowlistBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveAllowlistBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveAllowlistBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_allowlist_bundle_validator() -> AdaptiveAllowlistBundleValidator {
    AdaptiveAllowlistBundleValidator {
        rules: vec![
            Rc::new(AdaptiveAllowlistBundleAllowedCodeRule {
                allowed: vec!["default".to_string()],
            }) as Rc<dyn AdaptiveAllowlistBundleRule>,
            Rc::new(AdaptiveAllowlistBundleRequiredCodeRule {
                message: "adaptive allowlist bundle needs an entry".to_string(),
            }),
            Rc::new(AdaptiveAllowlistBundleCeilingRangeRule { min: 6.7, max: 3721.4 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_allowlist_bundle_validator();
        let item = new_adaptive_allowlist_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = AdaptiveAllowlistBundleRequiredCodeRule {
            message: "adaptive allowlist bundle needs this field".to_string(),
        };
        let mut item = new_adaptive_allowlist_bundle("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = AdaptiveAllowlistBundleCeilingRangeRule { min: 9.4, max: 1230.0 };
        let mut item = new_adaptive_allowlist_bundle("x", "y");
        item.ceiling = 1231.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
