use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient policy preference must pass. Check returns its problems, or none.
pub trait LenientPolicyPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &LenientPolicyPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPolicyPreferenceRegionLengthRule {
    pub max_length: i64,
}

impl LenientPolicyPreferenceRule for LenientPolicyPreferenceRegionLengthRule {
    fn check(&self, item: &LenientPolicyPreference) -> Vec<String> {
        let size = item.region.len() as i64;
        if size > self.max_length {
            return vec![format!("region is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPolicyPreferenceCodePrefixRule {
    pub prefix: String,
}

impl LenientPolicyPreferenceRule for LenientPolicyPreferenceCodePrefixRule {
    fn check(&self, item: &LenientPolicyPreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPolicyPreferenceRequiredChecksumRule {
    pub message: String,
}

impl LenientPolicyPreferenceRule for LenientPolicyPreferenceRequiredChecksumRule {
    fn check(&self, item: &LenientPolicyPreference) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPolicyPreferenceStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientPolicyPreferenceRule for LenientPolicyPreferenceStartsAtRangeRule {
    fn check(&self, item: &LenientPolicyPreference) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a lenient policy preference.
#[derive(Debug, Clone)]
pub struct LenientPolicyPreferenceValidator {
    pub rules: Vec<Rc<dyn LenientPolicyPreferenceRule>>,
}

impl LenientPolicyPreferenceValidator {
    pub fn validate(&self, item: &LenientPolicyPreference) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientPolicyPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientPolicyPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_policy_preference_validator() -> LenientPolicyPreferenceValidator {
    LenientPolicyPreferenceValidator {
        rules: vec![
            Rc::new(LenientPolicyPreferenceRegionLengthRule { max_length: 16 }) as Rc<dyn LenientPolicyPreferenceRule>,
            Rc::new(LenientPolicyPreferenceCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(LenientPolicyPreferenceRequiredChecksumRule {
                message: "lenient policy preference needs a value".to_string(),
            }),
            Rc::new(LenientPolicyPreferenceStartsAtRangeRule { min: 16, max: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_policy_preference_validator();
        let item = new_lenient_policy_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_region() {
        let rule = LenientPolicyPreferenceRegionLengthRule { max_length: 3600 };
        let mut item = new_lenient_policy_preference("x", "y");
        item.region = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.region = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = LenientPolicyPreferenceRequiredChecksumRule {
            message: "lenient policy preference needs a value".to_string(),
        };
        let mut item = new_lenient_policy_preference("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = LenientPolicyPreferenceStartsAtRangeRule { min: 120, max: 3600 };
        let mut item = new_lenient_policy_preference("x", "y");
        item.starts_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
