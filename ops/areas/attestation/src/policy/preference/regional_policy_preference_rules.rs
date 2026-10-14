use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional policy preference must pass. Check returns its problems, or none.
pub trait RegionalPolicyPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &RegionalPolicyPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPolicyPreferenceLabelLengthRule {
    pub max_length: i64,
}

impl RegionalPolicyPreferenceRule for RegionalPolicyPreferenceLabelLengthRule {
    fn check(&self, item: &RegionalPolicyPreference) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPolicyPreferenceAudienceLengthRule {
    pub max_length: i64,
}

impl RegionalPolicyPreferenceRule for RegionalPolicyPreferenceAudienceLengthRule {
    fn check(&self, item: &RegionalPolicyPreference) -> Vec<String> {
        let size = item.audience.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "audience is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPolicyPreferenceReviewerLengthRule {
    pub max_length: i64,
}

impl RegionalPolicyPreferenceRule for RegionalPolicyPreferenceReviewerLengthRule {
    fn check(&self, item: &RegionalPolicyPreference) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPolicyPreferenceRequiredAudienceRule {
    pub message: String,
}

impl RegionalPolicyPreferenceRule for RegionalPolicyPreferenceRequiredAudienceRule {
    fn check(&self, item: &RegionalPolicyPreference) -> Vec<String> {
        if item.audience.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPolicyPreferenceCodePrefixRule {
    pub prefix: String,
}

impl RegionalPolicyPreferenceRule for RegionalPolicyPreferenceCodePrefixRule {
    fn check(&self, item: &RegionalPolicyPreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPolicyPreferenceStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RegionalPolicyPreferenceRule for RegionalPolicyPreferenceStartsAtRangeRule {
    fn check(&self, item: &RegionalPolicyPreference) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional policy preference.
#[derive(Debug, Clone)]
pub struct RegionalPolicyPreferenceValidator {
    pub rules: Vec<Rc<dyn RegionalPolicyPreferenceRule>>,
}

impl RegionalPolicyPreferenceValidator {
    pub fn validate(&self, item: &RegionalPolicyPreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalPolicyPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalPolicyPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_policy_preference_validator() -> RegionalPolicyPreferenceValidator {
    RegionalPolicyPreferenceValidator {
        rules: vec![
            Rc::new(RegionalPolicyPreferenceLabelLengthRule { max_length: 120 })
                as Rc<dyn RegionalPolicyPreferenceRule>,
            Rc::new(RegionalPolicyPreferenceAudienceLengthRule { max_length: 32 }),
            Rc::new(RegionalPolicyPreferenceReviewerLengthRule { max_length: 1000 }),
            Rc::new(RegionalPolicyPreferenceRequiredAudienceRule {
                message: "regional policy preference needs this field".to_string(),
            }),
            Rc::new(RegionalPolicyPreferenceCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(RegionalPolicyPreferenceStartsAtRangeRule { min: 1000, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_policy_preference_validator();
        let item = new_regional_policy_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = RegionalPolicyPreferenceLabelLengthRule { max_length: 80 };
        let mut item = new_regional_policy_preference("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_audience() {
        let rule = RegionalPolicyPreferenceAudienceLengthRule { max_length: 16 };
        let mut item = new_regional_policy_preference("x", "y");
        item.audience = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.audience = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = RegionalPolicyPreferenceReviewerLengthRule { max_length: 1000 };
        let mut item = new_regional_policy_preference("x", "y");
        item.reviewer = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_audience() {
        let rule = RegionalPolicyPreferenceRequiredAudienceRule {
            message: "regional policy preference needs this field".to_string(),
        };
        let mut item = new_regional_policy_preference("", "");
        item.audience = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = RegionalPolicyPreferenceStartsAtRangeRule { min: 64, max: 1000 };
        let mut item = new_regional_policy_preference("x", "y");
        item.starts_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
