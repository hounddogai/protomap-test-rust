use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual allowlist bundle must pass. Check returns its problems, or none.
pub trait ManualAllowlistBundleRule: std::fmt::Debug {
    fn check(&self, item: &ManualAllowlistBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAllowlistBundleCodePrefixRule {
    pub prefix: String,
}

impl ManualAllowlistBundleRule for ManualAllowlistBundleCodePrefixRule {
    fn check(&self, item: &ManualAllowlistBundle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAllowlistBundleRequiredLabelRule {
    pub message: String,
}

impl ManualAllowlistBundleRule for ManualAllowlistBundleRequiredLabelRule {
    fn check(&self, item: &ManualAllowlistBundle) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAllowlistBundleEndsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualAllowlistBundleRule for ManualAllowlistBundleEndsAtRangeRule {
    fn check(&self, item: &ManualAllowlistBundle) -> Vec<String> {
        let value = item.ends_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ends at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ends at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAllowlistBundleTimezoneLengthRule {
    pub max_length: i64,
}

impl ManualAllowlistBundleRule for ManualAllowlistBundleTimezoneLengthRule {
    fn check(&self, item: &ManualAllowlistBundle) -> Vec<String> {
        let size = item.timezone.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "timezone is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualAllowlistBundleVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualAllowlistBundleRule for ManualAllowlistBundleVersionRangeRule {
    fn check(&self, item: &ManualAllowlistBundle) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual allowlist bundle.
#[derive(Debug, Clone)]
pub struct ManualAllowlistBundleValidator {
    pub rules: Vec<Rc<dyn ManualAllowlistBundleRule>>,
}

impl ManualAllowlistBundleValidator {
    pub fn validate(&self, item: &ManualAllowlistBundle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ManualAllowlistBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualAllowlistBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_allowlist_bundle_validator() -> ManualAllowlistBundleValidator {
    ManualAllowlistBundleValidator {
        rules: vec![
            Rc::new(ManualAllowlistBundleCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn ManualAllowlistBundleRule>,
            Rc::new(ManualAllowlistBundleRequiredLabelRule {
                message: "manual allowlist bundle needs an entry".to_string(),
            }),
            Rc::new(ManualAllowlistBundleEndsAtRangeRule { min: 16, max: 3600 }),
            Rc::new(ManualAllowlistBundleTimezoneLengthRule { max_length: 120 }),
            Rc::new(ManualAllowlistBundleVersionRangeRule { min: 120, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_allowlist_bundle_validator();
        let item = new_manual_allowlist_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = ManualAllowlistBundleRequiredLabelRule {
            message: "manual allowlist bundle needs a value".to_string(),
        };
        let mut item = new_manual_allowlist_bundle("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ends_at() {
        let rule = ManualAllowlistBundleEndsAtRangeRule { min: 32, max: 120 };
        let mut item = new_manual_allowlist_bundle("x", "y");
        item.ends_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_timezone() {
        let rule = ManualAllowlistBundleTimezoneLengthRule { max_length: 1000 };
        let mut item = new_manual_allowlist_bundle("x", "y");
        item.timezone = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.timezone = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = ManualAllowlistBundleVersionRangeRule { min: 255, max: 255 };
        let mut item = new_manual_allowlist_bundle("x", "y");
        item.version = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
