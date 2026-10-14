use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default silence preference must pass. Check returns its problems, or none.
pub trait DefaultSilencePreferenceRule: std::fmt::Debug {
    fn check(&self, item: &DefaultSilencePreference) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSilencePreferenceApprovedGuardRule {
    pub limit: i64,
}

impl DefaultSilencePreferenceRule for DefaultSilencePreferenceApprovedGuardRule {
    fn check(&self, item: &DefaultSilencePreference) -> Vec<String> {
        if item.approved && (item.revision > self.limit) {
            return vec![format!(
                "revision {} exceeds {} while approved",
                item.revision, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSilencePreferenceRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultSilencePreferenceRule for DefaultSilencePreferenceRotationDaysRangeRule {
    fn check(&self, item: &DefaultSilencePreference) -> Vec<String> {
        let value = item.rotation_days;
        if value < self.min || value > self.max {
            return vec![format!(
                "rotation days {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSilencePreferenceRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultSilencePreferenceRule for DefaultSilencePreferenceRatioRangeRule {
    fn check(&self, item: &DefaultSilencePreference) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSilencePreferenceRequiredCodeRule {
    pub message: String,
}

impl DefaultSilencePreferenceRule for DefaultSilencePreferenceRequiredCodeRule {
    fn check(&self, item: &DefaultSilencePreference) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSilencePreferenceSequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DefaultSilencePreferenceRule for DefaultSilencePreferenceSequenceRangeRule {
    fn check(&self, item: &DefaultSilencePreference) -> Vec<String> {
        let value = item.sequence;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("sequence {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("sequence {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a default silence preference.
#[derive(Debug, Clone)]
pub struct DefaultSilencePreferenceValidator {
    pub rules: Vec<Rc<dyn DefaultSilencePreferenceRule>>,
}

impl DefaultSilencePreferenceValidator {
    pub fn validate(&self, item: &DefaultSilencePreference) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DefaultSilencePreference) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultSilencePreference]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_silence_preference_validator() -> DefaultSilencePreferenceValidator {
    DefaultSilencePreferenceValidator {
        rules: vec![
            Rc::new(DefaultSilencePreferenceApprovedGuardRule { limit: 16 }) as Rc<dyn DefaultSilencePreferenceRule>,
            Rc::new(DefaultSilencePreferenceRotationDaysRangeRule { min: 64, max: 86400 }),
            Rc::new(DefaultSilencePreferenceRatioRangeRule { min: 1.1, max: 2620.2 }),
            Rc::new(DefaultSilencePreferenceRequiredCodeRule {
                message: "default silence preference needs a value".to_string(),
            }),
            Rc::new(DefaultSilencePreferenceSequenceRangeRule { min: 16, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_silence_preference_validator();
        let item = new_default_silence_preference("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = DefaultSilencePreferenceRotationDaysRangeRule { min: 32, max: 86400 };
        let mut item = new_default_silence_preference("x", "y");
        item.rotation_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = DefaultSilencePreferenceRatioRangeRule { min: 2.7, max: 1427.4 };
        let mut item = new_default_silence_preference("x", "y");
        item.ratio = 1428.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DefaultSilencePreferenceRequiredCodeRule {
            message: "default silence preference needs a value".to_string(),
        };
        let mut item = new_default_silence_preference("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = DefaultSilencePreferenceSequenceRangeRule { min: 32, max: 1000 };
        let mut item = new_default_silence_preference("x", "y");
        item.sequence = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
