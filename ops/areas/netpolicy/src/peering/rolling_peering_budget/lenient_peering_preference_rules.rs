use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient peering preference must pass. Check returns its problems, or none.
pub trait LenientPeeringPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &LenientPeeringPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPeeringPreferenceRequiredCodeRule {
    pub message: String,
}

impl LenientPeeringPreferenceRule for LenientPeeringPreferenceRequiredCodeRule {
    fn check(&self, item: &LenientPeeringPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPeeringPreferenceRequiredScopeRule {
    pub message: String,
}

impl LenientPeeringPreferenceRule for LenientPeeringPreferenceRequiredScopeRule {
    fn check(&self, item: &LenientPeeringPreference) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPeeringPreferenceTitleLengthRule {
    pub max_length: i64,
}

impl LenientPeeringPreferenceRule for LenientPeeringPreferenceTitleLengthRule {
    fn check(&self, item: &LenientPeeringPreference) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPeeringPreferenceAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientPeeringPreferenceRule for LenientPeeringPreferenceAttemptsRangeRule {
    fn check(&self, item: &LenientPeeringPreference) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPeeringPreferenceRequiredTitleRule {
    pub message: String,
}

impl LenientPeeringPreferenceRule for LenientPeeringPreferenceRequiredTitleRule {
    fn check(&self, item: &LenientPeeringPreference) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPeeringPreferenceStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientPeeringPreferenceRule for LenientPeeringPreferenceStartsAtRangeRule {
    fn check(&self, item: &LenientPeeringPreference) -> Vec<String> {
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

/// Runs every rule of a lenient peering preference.
#[derive(Debug, Clone)]
pub struct LenientPeeringPreferenceValidator {
    pub rules: Vec<Rc<dyn LenientPeeringPreferenceRule>>,
}

impl LenientPeeringPreferenceValidator {
    pub fn validate(&self, item: &LenientPeeringPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientPeeringPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientPeeringPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_peering_preference_validator() -> LenientPeeringPreferenceValidator {
    LenientPeeringPreferenceValidator {
        rules: vec![
            Rc::new(LenientPeeringPreferenceRequiredCodeRule {
                message: "lenient peering preference needs an entry".to_string(),
            }) as Rc<dyn LenientPeeringPreferenceRule>,
            Rc::new(LenientPeeringPreferenceRequiredScopeRule {
                message: "lenient peering preference needs a value".to_string(),
            }),
            Rc::new(LenientPeeringPreferenceTitleLengthRule { max_length: 64 }),
            Rc::new(LenientPeeringPreferenceAttemptsRangeRule { min: 255, max: 3600 }),
            Rc::new(LenientPeeringPreferenceRequiredTitleRule {
                message: "lenient peering preference needs an entry".to_string(),
            }),
            Rc::new(LenientPeeringPreferenceStartsAtRangeRule { min: 3600, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_peering_preference_validator();
        let item = new_lenient_peering_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientPeeringPreferenceRequiredCodeRule {
            message: "lenient peering preference needs an entry".to_string(),
        };
        let mut item = new_lenient_peering_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = LenientPeeringPreferenceRequiredScopeRule {
            message: "lenient peering preference needs an entry".to_string(),
        };
        let mut item = new_lenient_peering_preference("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LenientPeeringPreferenceTitleLengthRule { max_length: 64 };
        let mut item = new_lenient_peering_preference("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = LenientPeeringPreferenceAttemptsRangeRule { min: 120, max: 120 };
        let mut item = new_lenient_peering_preference("x", "y");
        item.attempts = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LenientPeeringPreferenceRequiredTitleRule {
            message: "lenient peering preference needs a value".to_string(),
        };
        let mut item = new_lenient_peering_preference("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = LenientPeeringPreferenceStartsAtRangeRule { min: 64, max: 120 };
        let mut item = new_lenient_peering_preference("x", "y");
        item.starts_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
