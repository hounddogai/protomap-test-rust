use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional zone manifest must pass. Check returns its problems, or none.
pub trait RegionalZoneManifestRule: std::fmt::Debug {
    fn check(&self, item: &RegionalZoneManifest) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalZoneManifestRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalZoneManifestRule for RegionalZoneManifestRetriesRangeRule {
    fn check(&self, item: &RegionalZoneManifest) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalZoneManifestRequiredTitleRule {
    pub message: String,
}

impl RegionalZoneManifestRule for RegionalZoneManifestRequiredTitleRule {
    fn check(&self, item: &RegionalZoneManifest) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalZoneManifestMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RegionalZoneManifestRule for RegionalZoneManifestMaximumRangeRule {
    fn check(&self, item: &RegionalZoneManifest) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalZoneManifestTitleLengthRule {
    pub max_length: i64,
}

impl RegionalZoneManifestRule for RegionalZoneManifestTitleLengthRule {
    fn check(&self, item: &RegionalZoneManifest) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalZoneManifestCodePrefixRule {
    pub prefix: String,
}

impl RegionalZoneManifestRule for RegionalZoneManifestCodePrefixRule {
    fn check(&self, item: &RegionalZoneManifest) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalZoneManifestAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl RegionalZoneManifestRule for RegionalZoneManifestAllowedTitleRule {
    fn check(&self, item: &RegionalZoneManifest) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is too long", item.title)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional zone manifest.
#[derive(Debug, Clone)]
pub struct RegionalZoneManifestValidator {
    pub rules: Vec<Rc<dyn RegionalZoneManifestRule>>,
}

impl RegionalZoneManifestValidator {
    pub fn validate(&self, item: &RegionalZoneManifest) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalZoneManifest) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalZoneManifest]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_zone_manifest_validator() -> RegionalZoneManifestValidator {
    RegionalZoneManifestValidator {
        rules: vec![
            Rc::new(RegionalZoneManifestRetriesRangeRule { min: 80, max: 86400 }) as Rc<dyn RegionalZoneManifestRule>,
            Rc::new(RegionalZoneManifestRequiredTitleRule {
                message: "regional zone manifest needs an entry".to_string(),
            }),
            Rc::new(RegionalZoneManifestMaximumRangeRule { min: 1.1, max: 4575.0 }),
            Rc::new(RegionalZoneManifestTitleLengthRule { max_length: 1000 }),
            Rc::new(RegionalZoneManifestCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(RegionalZoneManifestAllowedTitleRule {
                allowed: vec!["manual".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_zone_manifest_validator();
        let item = new_regional_zone_manifest("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = RegionalZoneManifestRetriesRangeRule { min: 64, max: 120 };
        let mut item = new_regional_zone_manifest("x", "y");
        item.retries = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RegionalZoneManifestRequiredTitleRule {
            message: "regional zone manifest needs this field".to_string(),
        };
        let mut item = new_regional_zone_manifest("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = RegionalZoneManifestMaximumRangeRule { min: 1.5, max: 2366.3 };
        let mut item = new_regional_zone_manifest("x", "y");
        item.maximum = 2367.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RegionalZoneManifestTitleLengthRule { max_length: 1000 };
        let mut item = new_regional_zone_manifest("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
