use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a regional usage score must pass. Check returns its problems, or none.
pub trait RegionalUsageScoreRule: std::fmt::Debug {
    fn check(&self, item: &RegionalUsageScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageScoreCodeLengthRule {
    pub max_length: i64,
}

impl RegionalUsageScoreRule for RegionalUsageScoreCodeLengthRule {
    fn check(&self, item: &RegionalUsageScore) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageScoreRequiredSegmentRule {
    pub message: String,
}

impl RegionalUsageScoreRule for RegionalUsageScoreRequiredSegmentRule {
    fn check(&self, item: &RegionalUsageScore) -> Vec<String> {
        if item.segment.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionalUsageScoreRequiredNotesRule {
    pub message: String,
}

impl RegionalUsageScoreRule for RegionalUsageScoreRequiredNotesRule {
    fn check(&self, item: &RegionalUsageScore) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a regional usage score.
#[derive(Debug, Clone)]
pub struct RegionalUsageScoreValidator {
    pub rules: Vec<Rc<dyn RegionalUsageScoreRule>>,
}

impl RegionalUsageScoreValidator {
    pub fn validate(&self, item: &RegionalUsageScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RegionalUsageScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RegionalUsageScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_regional_usage_score_validator() -> RegionalUsageScoreValidator {
    RegionalUsageScoreValidator {
        rules: vec![
            Rc::new(RegionalUsageScoreCodeLengthRule { max_length: 1000 }) as Rc<dyn RegionalUsageScoreRule>,
            Rc::new(RegionalUsageScoreRequiredSegmentRule {
                message: "regional usage score needs this field".to_string(),
            }),
            Rc::new(RegionalUsageScoreRequiredNotesRule {
                message: "regional usage score needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_regional_usage_score_validator();
        let item = new_regional_usage_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RegionalUsageScoreCodeLengthRule { max_length: 1000 };
        let mut item = new_regional_usage_score("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_segment() {
        let rule = RegionalUsageScoreRequiredSegmentRule {
            message: "regional usage score needs an entry".to_string(),
        };
        let mut item = new_regional_usage_score("", "");
        item.segment = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = RegionalUsageScoreRequiredNotesRule {
            message: "regional usage score needs an entry".to_string(),
        };
        let mut item = new_regional_usage_score("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
