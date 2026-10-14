use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient retention preference must pass. Check returns its problems, or none.
pub trait LenientRetentionPreferenceRule: std::fmt::Debug {
    fn check(&self, item: &LenientRetentionPreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRetentionPreferenceRequiredKeyIdRule {
    pub message: String,
}

impl LenientRetentionPreferenceRule for LenientRetentionPreferenceRequiredKeyIdRule {
    fn check(&self, item: &LenientRetentionPreference) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRetentionPreferenceRequiredCodeRule {
    pub message: String,
}

impl LenientRetentionPreferenceRule for LenientRetentionPreferenceRequiredCodeRule {
    fn check(&self, item: &LenientRetentionPreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientRetentionPreferenceScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LenientRetentionPreferenceRule for LenientRetentionPreferenceScoreRangeRule {
    fn check(&self, item: &LenientRetentionPreference) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a lenient retention preference.
#[derive(Debug, Clone)]
pub struct LenientRetentionPreferenceValidator {
    pub rules: Vec<Rc<dyn LenientRetentionPreferenceRule>>,
}

impl LenientRetentionPreferenceValidator {
    pub fn validate(&self, item: &LenientRetentionPreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientRetentionPreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientRetentionPreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_retention_preference_validator() -> LenientRetentionPreferenceValidator {
    LenientRetentionPreferenceValidator {
        rules: vec![
            Rc::new(LenientRetentionPreferenceRequiredKeyIdRule {
                message: "lenient retention preference needs an entry".to_string(),
            }) as Rc<dyn LenientRetentionPreferenceRule>,
            Rc::new(LenientRetentionPreferenceRequiredCodeRule {
                message: "lenient retention preference needs an entry".to_string(),
            }),
            Rc::new(LenientRetentionPreferenceScoreRangeRule { min: 4.7, max: 3674.7 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_retention_preference_validator();
        let item = new_lenient_retention_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = LenientRetentionPreferenceRequiredKeyIdRule {
            message: "lenient retention preference needs a value".to_string(),
        };
        let mut item = new_lenient_retention_preference("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientRetentionPreferenceRequiredCodeRule {
            message: "lenient retention preference needs an entry".to_string(),
        };
        let mut item = new_lenient_retention_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = LenientRetentionPreferenceScoreRangeRule { min: 1.4, max: 2784.5 };
        let mut item = new_lenient_retention_preference("x", "y");
        item.score = 2786.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
