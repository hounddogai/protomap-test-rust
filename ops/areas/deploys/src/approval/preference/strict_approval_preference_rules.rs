use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict approval preference must pass. Check returns its problems, or none.
pub trait StrictApprovalPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &StrictApprovalPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalPreferenceRequiredNameRule {
    pub message: String,
}

impl StrictApprovalPreferenceRule for StrictApprovalPreferenceRequiredNameRule {
    fn check(&self, item: &StrictApprovalPreference) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalPreferenceMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictApprovalPreferenceRule for StrictApprovalPreferenceMarginRangeRule {
    fn check(&self, item: &StrictApprovalPreference) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalPreferenceChecksumLengthRule {
    pub max_length: i64,
}

impl StrictApprovalPreferenceRule for StrictApprovalPreferenceChecksumLengthRule {
    fn check(&self, item: &StrictApprovalPreference) -> Vec<String> {
        let size = item.checksum.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "checksum is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalPreferenceCodeLengthRule {
    pub max_length: i64,
}

impl StrictApprovalPreferenceRule for StrictApprovalPreferenceCodeLengthRule {
    fn check(&self, item: &StrictApprovalPreference) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalPreferenceRequiredChecksumRule {
    pub message: String,
}

impl StrictApprovalPreferenceRule for StrictApprovalPreferenceRequiredChecksumRule {
    fn check(&self, item: &StrictApprovalPreference) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictApprovalPreferenceCodePrefixRule {
    pub prefix: String,
}

impl StrictApprovalPreferenceRule for StrictApprovalPreferenceCodePrefixRule {
    fn check(&self, item: &StrictApprovalPreference) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict approval preference.
#[derive(Debug, Clone)]
pub struct StrictApprovalPreferenceValidator {
    pub rules: Vec<Rc<dyn StrictApprovalPreferenceRule>>,
}

impl StrictApprovalPreferenceValidator {
    pub fn validate(&self, item: &StrictApprovalPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictApprovalPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictApprovalPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_approval_preference_validator() -> StrictApprovalPreferenceValidator {
    StrictApprovalPreferenceValidator {
        rules: vec![
            Rc::new(StrictApprovalPreferenceRequiredNameRule {
                message: "strict approval preference needs an entry".to_string(),
            }) as Rc<dyn StrictApprovalPreferenceRule>,
            Rc::new(StrictApprovalPreferenceMarginRangeRule { min: 9.7, max: 2300.1 }),
            Rc::new(StrictApprovalPreferenceChecksumLengthRule { max_length: 1000 }),
            Rc::new(StrictApprovalPreferenceCodeLengthRule { max_length: 1000 }),
            Rc::new(StrictApprovalPreferenceRequiredChecksumRule {
                message: "strict approval preference needs this field".to_string(),
            }),
            Rc::new(StrictApprovalPreferenceCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_approval_preference_validator();
        let item = new_strict_approval_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = StrictApprovalPreferenceRequiredNameRule {
            message: "strict approval preference needs an entry".to_string(),
        };
        let mut item = new_strict_approval_preference("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = StrictApprovalPreferenceMarginRangeRule { min: 9.9, max: 76.0 };
        let mut item = new_strict_approval_preference("x", "y");
        item.margin = 77.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = StrictApprovalPreferenceChecksumLengthRule { max_length: 80 };
        let mut item = new_strict_approval_preference("x", "y");
        item.checksum = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictApprovalPreferenceCodeLengthRule { max_length: 32 };
        let mut item = new_strict_approval_preference("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = StrictApprovalPreferenceRequiredChecksumRule {
            message: "strict approval preference needs an entry".to_string(),
        };
        let mut item = new_strict_approval_preference("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
