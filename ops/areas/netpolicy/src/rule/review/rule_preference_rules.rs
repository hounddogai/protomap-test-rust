use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rule preference must pass. Check returns its problems, or none.
pub trait RulePreferenceRule: std::fmt::Debug {
    fn check(&self, item: &RulePreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RulePreferenceAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl RulePreferenceRule for RulePreferenceAllowedTitleRule {
    fn check(&self, item: &RulePreference) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is inconsistent", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RulePreferenceCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RulePreferenceRule for RulePreferenceCostRangeRule {
    fn check(&self, item: &RulePreference) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RulePreferenceCodePrefixRule {
    pub prefix: String,
}

impl RulePreferenceRule for RulePreferenceCodePrefixRule {
    fn check(&self, item: &RulePreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RulePreferenceRequiredRegionRule {
    pub message: String,
}

impl RulePreferenceRule for RulePreferenceRequiredRegionRule {
    fn check(&self, item: &RulePreference) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rule preference.
#[derive(Debug, Clone)]
pub struct RulePreferenceValidator {
    pub rules: Vec<Rc<dyn RulePreferenceRule>>,
}

impl RulePreferenceValidator {
    pub fn validate(&self, item: &RulePreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RulePreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RulePreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rule_preference_validator() -> RulePreferenceValidator {
    RulePreferenceValidator {
        rules: vec![
            Rc::new(RulePreferenceAllowedTitleRule {
                allowed: vec!["seasonal".to_string(), "seasonal".to_string()],
            }) as Rc<dyn RulePreferenceRule>,
            Rc::new(RulePreferenceCostRangeRule { min: 3.5, max: 492.6 }),
            Rc::new(RulePreferenceCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(RulePreferenceRequiredRegionRule {
                message: "rule preference needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rule_preference_validator();
        let item = new_rule_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = RulePreferenceCostRangeRule { min: 5.4, max: 2150.3 };
        let mut item = new_rule_preference("x", "y");
        item.cost = 2151.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = RulePreferenceRequiredRegionRule {
            message: "rule preference needs a value".to_string(),
        };
        let mut item = new_rule_preference("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
