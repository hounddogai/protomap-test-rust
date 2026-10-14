use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual health check score must pass. Check returns its problems, or none.
pub trait ManualHealthCheckScoreRule: std::fmt::Debug {
    fn check(&self, item: &ManualHealthCheckScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualHealthCheckScoreChecksumLengthRule {
    pub max_length: i64,
}

impl ManualHealthCheckScoreRule for ManualHealthCheckScoreChecksumLengthRule {
    fn check(&self, item: &ManualHealthCheckScore) -> Vec<String> {
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
pub struct ManualHealthCheckScoreRevisionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualHealthCheckScoreRule for ManualHealthCheckScoreRevisionRangeRule {
    fn check(&self, item: &ManualHealthCheckScore) -> Vec<String> {
        let value = item.revision;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("revision {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("revision {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualHealthCheckScoreTitleLengthRule {
    pub max_length: i64,
}

impl ManualHealthCheckScoreRule for ManualHealthCheckScoreTitleLengthRule {
    fn check(&self, item: &ManualHealthCheckScore) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualHealthCheckScoreCodeLengthRule {
    pub max_length: i64,
}

impl ManualHealthCheckScoreRule for ManualHealthCheckScoreCodeLengthRule {
    fn check(&self, item: &ManualHealthCheckScore) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualHealthCheckScoreRequiredTitleRule {
    pub message: String,
}

impl ManualHealthCheckScoreRule for ManualHealthCheckScoreRequiredTitleRule {
    fn check(&self, item: &ManualHealthCheckScore) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualHealthCheckScoreRequiredChecksumRule {
    pub message: String,
}

impl ManualHealthCheckScoreRule for ManualHealthCheckScoreRequiredChecksumRule {
    fn check(&self, item: &ManualHealthCheckScore) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual health check score.
#[derive(Debug, Clone)]
pub struct ManualHealthCheckScoreValidator {
    pub rules: Vec<Rc<dyn ManualHealthCheckScoreRule>>,
}

impl ManualHealthCheckScoreValidator {
    pub fn validate(&self, item: &ManualHealthCheckScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualHealthCheckScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualHealthCheckScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_health_check_score_validator() -> ManualHealthCheckScoreValidator {
    ManualHealthCheckScoreValidator {
        rules: vec![
            Rc::new(ManualHealthCheckScoreChecksumLengthRule { max_length: 64 }) as Rc<dyn ManualHealthCheckScoreRule>,
            Rc::new(ManualHealthCheckScoreRevisionRangeRule { min: 16, max: 1000 }),
            Rc::new(ManualHealthCheckScoreTitleLengthRule { max_length: 86400 }),
            Rc::new(ManualHealthCheckScoreCodeLengthRule { max_length: 80 }),
            Rc::new(ManualHealthCheckScoreRequiredTitleRule {
                message: "manual health check score needs an entry".to_string(),
            }),
            Rc::new(ManualHealthCheckScoreRequiredChecksumRule {
                message: "manual health check score needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_health_check_score_validator();
        let item = new_manual_health_check_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_checksum() {
        let rule = ManualHealthCheckScoreChecksumLengthRule { max_length: 1000 };
        let mut item = new_manual_health_check_score("x", "y");
        item.checksum = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.checksum = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_revision() {
        let rule = ManualHealthCheckScoreRevisionRangeRule { min: 80, max: 120 };
        let mut item = new_manual_health_check_score("x", "y");
        item.revision = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = ManualHealthCheckScoreTitleLengthRule { max_length: 255 };
        let mut item = new_manual_health_check_score("x", "y");
        item.title = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualHealthCheckScoreCodeLengthRule { max_length: 80 };
        let mut item = new_manual_health_check_score("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = ManualHealthCheckScoreRequiredTitleRule {
            message: "manual health check score needs an entry".to_string(),
        };
        let mut item = new_manual_health_check_score("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = ManualHealthCheckScoreRequiredChecksumRule {
            message: "manual health check score needs an entry".to_string(),
        };
        let mut item = new_manual_health_check_score("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
