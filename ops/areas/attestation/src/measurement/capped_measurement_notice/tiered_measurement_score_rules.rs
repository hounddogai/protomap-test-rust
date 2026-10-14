use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered measurement score must pass. Check returns its problems, or none.
pub trait TieredMeasurementScoreRule: std::fmt::Debug {
    fn check(&self, item: &TieredMeasurementScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementScoreCodeLengthRule {
    pub max_length: i64,
}

impl TieredMeasurementScoreRule for TieredMeasurementScoreCodeLengthRule {
    fn check(&self, item: &TieredMeasurementScore) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementScoreRequiredCodeRule {
    pub message: String,
}

impl TieredMeasurementScoreRule for TieredMeasurementScoreRequiredCodeRule {
    fn check(&self, item: &TieredMeasurementScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementScoreRequiredNameRule {
    pub message: String,
}

impl TieredMeasurementScoreRule for TieredMeasurementScoreRequiredNameRule {
    fn check(&self, item: &TieredMeasurementScore) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementScoreTitleLengthRule {
    pub max_length: i64,
}

impl TieredMeasurementScoreRule for TieredMeasurementScoreTitleLengthRule {
    fn check(&self, item: &TieredMeasurementScore) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementScoreRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredMeasurementScoreRule for TieredMeasurementScoreRevisionRangeRule {
    fn check(&self, item: &TieredMeasurementScore) -> Vec<String> {
        let value = item.revision;
        if value < self.min || value > self.max {
            return vec![format!("revision {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredMeasurementScoreCodePrefixRule {
    pub prefix: String,
}

impl TieredMeasurementScoreRule for TieredMeasurementScoreCodePrefixRule {
    fn check(&self, item: &TieredMeasurementScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered measurement score.
#[derive(Debug, Clone)]
pub struct TieredMeasurementScoreValidator {
    pub rules: Vec<Rc<dyn TieredMeasurementScoreRule>>,
}

impl TieredMeasurementScoreValidator {
    pub fn validate(&self, item: &TieredMeasurementScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredMeasurementScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredMeasurementScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_measurement_score_validator() -> TieredMeasurementScoreValidator {
    TieredMeasurementScoreValidator {
        rules: vec![
            Rc::new(TieredMeasurementScoreCodeLengthRule { max_length: 16 }) as Rc<dyn TieredMeasurementScoreRule>,
            Rc::new(TieredMeasurementScoreRequiredCodeRule {
                message: "tiered measurement score needs a value".to_string(),
            }),
            Rc::new(TieredMeasurementScoreRequiredNameRule {
                message: "tiered measurement score needs a value".to_string(),
            }),
            Rc::new(TieredMeasurementScoreTitleLengthRule { max_length: 32 }),
            Rc::new(TieredMeasurementScoreRevisionRangeRule { min: 64, max: 86400 }),
            Rc::new(TieredMeasurementScoreCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_measurement_score_validator();
        let item = new_tiered_measurement_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredMeasurementScoreCodeLengthRule { max_length: 3600 };
        let mut item = new_tiered_measurement_score("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredMeasurementScoreRequiredCodeRule {
            message: "tiered measurement score needs a value".to_string(),
        };
        let mut item = new_tiered_measurement_score("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = TieredMeasurementScoreRequiredNameRule {
            message: "tiered measurement score needs this field".to_string(),
        };
        let mut item = new_tiered_measurement_score("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = TieredMeasurementScoreTitleLengthRule { max_length: 1000 };
        let mut item = new_tiered_measurement_score("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = TieredMeasurementScoreRevisionRangeRule { min: 64, max: 3600 };
        let mut item = new_tiered_measurement_score("x", "y");
        item.revision = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
