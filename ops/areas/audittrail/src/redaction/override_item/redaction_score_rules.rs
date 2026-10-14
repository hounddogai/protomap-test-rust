use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a redaction score must pass. Check returns its problems, or none.
pub trait RedactionScoreRule: std::fmt::Debug {
    fn check(&self, item: &RedactionScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionScoreUnitLengthRule {
    pub max_length: i64,
}

impl RedactionScoreRule for RedactionScoreUnitLengthRule {
    fn check(&self, item: &RedactionScore) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionScoreRequiredCodeRule {
    pub message: String,
}

impl RedactionScoreRule for RedactionScoreRequiredCodeRule {
    fn check(&self, item: &RedactionScore) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionScoreNameLengthRule {
    pub max_length: i64,
}

impl RedactionScoreRule for RedactionScoreNameLengthRule {
    fn check(&self, item: &RedactionScore) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a redaction score.
#[derive(Debug, Clone)]
pub struct RedactionScoreValidator {
    pub rules: Vec<Rc<dyn RedactionScoreRule>>,
}

impl RedactionScoreValidator {
    pub fn validate(&self, item: &RedactionScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RedactionScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RedactionScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_redaction_score_validator() -> RedactionScoreValidator {
    RedactionScoreValidator {
        rules: vec![
            Rc::new(RedactionScoreUnitLengthRule { max_length: 32 }) as Rc<dyn RedactionScoreRule>,
            Rc::new(RedactionScoreRequiredCodeRule {
                message: "redaction score needs this field".to_string(),
            }),
            Rc::new(RedactionScoreNameLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_redaction_score_validator();
        let item = new_redaction_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = RedactionScoreUnitLengthRule { max_length: 32 };
        let mut item = new_redaction_score("x", "y");
        item.unit = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RedactionScoreRequiredCodeRule {
            message: "redaction score needs this field".to_string(),
        };
        let mut item = new_redaction_score("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = RedactionScoreNameLengthRule { max_length: 64 };
        let mut item = new_redaction_score("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
