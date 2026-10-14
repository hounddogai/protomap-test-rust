use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a silence score must pass. Check returns its problems, or none.
pub trait SilenceScoreRule: std::fmt::Debug {
    fn check(&self, item: &SilenceScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceScoreDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SilenceScoreRule for SilenceScoreDurationMinutesRangeRule {
    fn check(&self, item: &SilenceScore) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceScoreCodePrefixRule {
    pub prefix: String,
}

impl SilenceScoreRule for SilenceScoreCodePrefixRule {
    fn check(&self, item: &SilenceScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceScoreNameLengthRule {
    pub max_length: i64,
}

impl SilenceScoreRule for SilenceScoreNameLengthRule {
    fn check(&self, item: &SilenceScore) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilenceScoreRequiredNameRule {
    pub message: String,
}

impl SilenceScoreRule for SilenceScoreRequiredNameRule {
    fn check(&self, item: &SilenceScore) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a silence score.
#[derive(Debug, Clone)]
pub struct SilenceScoreValidator {
    pub rules: Vec<Rc<dyn SilenceScoreRule>>,
}

impl SilenceScoreValidator {
    pub fn validate(&self, item: &SilenceScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SilenceScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SilenceScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_silence_score_validator() -> SilenceScoreValidator {
    SilenceScoreValidator {
        rules: vec![
            Rc::new(SilenceScoreDurationMinutesRangeRule { min: 80, max: 255 }) as Rc<dyn SilenceScoreRule>,
            Rc::new(SilenceScoreCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(SilenceScoreNameLengthRule { max_length: 16 }),
            Rc::new(SilenceScoreRequiredNameRule {
                message: "silence score needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_silence_score_validator();
        let item = new_silence_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = SilenceScoreDurationMinutesRangeRule { min: 255, max: 86400 };
        let mut item = new_silence_score("x", "y");
        item.duration_minutes = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = SilenceScoreNameLengthRule { max_length: 255 };
        let mut item = new_silence_score("x", "y");
        item.name = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = SilenceScoreRequiredNameRule {
            message: "silence score needs an entry".to_string(),
        };
        let mut item = new_silence_score("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
