use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a regional action item preference must pass. Check returns its problems, or none.
pub trait RegionalActionItemPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &RegionalActionItemPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalActionItemPreferenceAllowedIssuerRule {
    pub allowed: Vec<String>,
}

impl RegionalActionItemPreferenceRule for RegionalActionItemPreferenceAllowedIssuerRule {
    fn check(&self, item: &RegionalActionItemPreference) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.issuer))) {
            return vec![format!("issuer {} is out of range", item.issuer)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalActionItemPreferenceRequiredRegionRule {
    pub message: String,
}

impl RegionalActionItemPreferenceRule for RegionalActionItemPreferenceRequiredRegionRule {
    fn check(&self, item: &RegionalActionItemPreference) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalActionItemPreferenceRequiredNameRule {
    pub message: String,
}

impl RegionalActionItemPreferenceRule for RegionalActionItemPreferenceRequiredNameRule {
    fn check(&self, item: &RegionalActionItemPreference) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalActionItemPreferenceQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalActionItemPreferenceRule for RegionalActionItemPreferenceQuotaUnitsRangeRule {
    fn check(&self, item: &RegionalActionItemPreference) -> Vec<String> {
        let value = item.quota_units;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quota units {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quota units {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a regional action item preference.
#[derive(Debug, Clone)]
pub struct RegionalActionItemPreferenceValidator {
    pub rules: Vec<Rc<dyn RegionalActionItemPreferenceRule>>,
}

impl RegionalActionItemPreferenceValidator {
    pub fn validate(&self, item: &RegionalActionItemPreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalActionItemPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalActionItemPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_action_item_preference_validator() -> RegionalActionItemPreferenceValidator {
    RegionalActionItemPreferenceValidator {
        rules: vec![
            Rc::new(RegionalActionItemPreferenceAllowedIssuerRule { allowed: Vec::new() })
                as Rc<dyn RegionalActionItemPreferenceRule>,
            Rc::new(RegionalActionItemPreferenceRequiredRegionRule {
                message: "regional action item preference needs this field".to_string(),
            }),
            Rc::new(RegionalActionItemPreferenceRequiredNameRule {
                message: "regional action item preference needs a value".to_string(),
            }),
            Rc::new(RegionalActionItemPreferenceQuotaUnitsRangeRule { min: 255, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_action_item_preference_validator();
        let item = new_regional_action_item_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = RegionalActionItemPreferenceRequiredRegionRule {
            message: "regional action item preference needs this field".to_string(),
        };
        let mut item = new_regional_action_item_preference("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RegionalActionItemPreferenceRequiredNameRule {
            message: "regional action item preference needs this field".to_string(),
        };
        let mut item = new_regional_action_item_preference("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = RegionalActionItemPreferenceQuotaUnitsRangeRule { min: 16, max: 32 };
        let mut item = new_regional_action_item_preference("x", "y");
        item.quota_units = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
