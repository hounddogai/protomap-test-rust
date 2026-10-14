use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict limit variance must pass. Check returns its problems, or none.
pub trait StrictLimitVarianceRule: std::fmt::Debug {
    fn check(&self, item: &StrictLimitVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictLimitVarianceRequiredNameRule {
    pub message: String,
}

impl StrictLimitVarianceRule for StrictLimitVarianceRequiredNameRule {
    fn check(&self, item: &StrictLimitVariance) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictLimitVarianceNotesLengthRule {
    pub max_length: i64,
}

impl StrictLimitVarianceRule for StrictLimitVarianceNotesLengthRule {
    fn check(&self, item: &StrictLimitVariance) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictLimitVarianceRequiredCodeRule {
    pub message: String,
}

impl StrictLimitVarianceRule for StrictLimitVarianceRequiredCodeRule {
    fn check(&self, item: &StrictLimitVariance) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictLimitVarianceRequiredNotesRule {
    pub message: String,
}

impl StrictLimitVarianceRule for StrictLimitVarianceRequiredNotesRule {
    fn check(&self, item: &StrictLimitVariance) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictLimitVarianceWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictLimitVarianceRule for StrictLimitVarianceWindowDaysRangeRule {
    fn check(&self, item: &StrictLimitVariance) -> Vec<String> {
        let value = item.window_days;
        if value < self.min || value > self.max {
            return vec![format!("window days {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict limit variance.
#[derive(Debug, Clone)]
pub struct StrictLimitVarianceValidator {
    pub rules: Vec<Rc<dyn StrictLimitVarianceRule>>,
}

impl StrictLimitVarianceValidator {
    pub fn validate(&self, item: &StrictLimitVariance) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictLimitVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictLimitVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_limit_variance_validator() -> StrictLimitVarianceValidator {
    StrictLimitVarianceValidator {
        rules: vec![
            Rc::new(StrictLimitVarianceRequiredNameRule {
                message: "strict limit variance needs an entry".to_string(),
            }) as Rc<dyn StrictLimitVarianceRule>,
            Rc::new(StrictLimitVarianceNotesLengthRule { max_length: 32 }),
            Rc::new(StrictLimitVarianceRequiredCodeRule {
                message: "strict limit variance needs a value".to_string(),
            }),
            Rc::new(StrictLimitVarianceRequiredNotesRule {
                message: "strict limit variance needs a value".to_string(),
            }),
            Rc::new(StrictLimitVarianceWindowDaysRangeRule { min: 32, max: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_limit_variance_validator();
        let item = new_strict_limit_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = StrictLimitVarianceRequiredNameRule {
            message: "strict limit variance needs this field".to_string(),
        };
        let mut item = new_strict_limit_variance("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = StrictLimitVarianceNotesLengthRule { max_length: 64 };
        let mut item = new_strict_limit_variance("x", "y");
        item.notes = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = StrictLimitVarianceRequiredCodeRule {
            message: "strict limit variance needs a value".to_string(),
        };
        let mut item = new_strict_limit_variance("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = StrictLimitVarianceRequiredNotesRule {
            message: "strict limit variance needs a value".to_string(),
        };
        let mut item = new_strict_limit_variance("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = StrictLimitVarianceWindowDaysRangeRule { min: 16, max: 32 };
        let mut item = new_strict_limit_variance("x", "y");
        item.window_days = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
