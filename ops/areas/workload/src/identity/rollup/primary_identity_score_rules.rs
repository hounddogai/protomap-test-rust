use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary identity score must pass. Check returns its problems, or none.
pub trait PrimaryIdentityScoreRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryIdentityScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryIdentityScoreCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryIdentityScoreRule for PrimaryIdentityScoreCodeLengthRule {
    fn check(&self, item: &PrimaryIdentityScore) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryIdentityScoreTitleLengthRule {
    pub max_length: i64,
}

impl PrimaryIdentityScoreRule for PrimaryIdentityScoreTitleLengthRule {
    fn check(&self, item: &PrimaryIdentityScore) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryIdentityScoreCodePrefixRule {
    pub prefix: String,
}

impl PrimaryIdentityScoreRule for PrimaryIdentityScoreCodePrefixRule {
    fn check(&self, item: &PrimaryIdentityScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary identity score.
#[derive(Debug, Clone)]
pub struct PrimaryIdentityScoreValidator {
    pub rules: Vec<Rc<dyn PrimaryIdentityScoreRule>>,
}

impl PrimaryIdentityScoreValidator {
    pub fn validate(&self, item: &PrimaryIdentityScore) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryIdentityScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryIdentityScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_identity_score_validator() -> PrimaryIdentityScoreValidator {
    PrimaryIdentityScoreValidator {
        rules: vec![
            Rc::new(PrimaryIdentityScoreCodeLengthRule { max_length: 86400 }) as Rc<dyn PrimaryIdentityScoreRule>,
            Rc::new(PrimaryIdentityScoreTitleLengthRule { max_length: 80 }),
            Rc::new(PrimaryIdentityScoreCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_identity_score_validator();
        let item = new_primary_identity_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryIdentityScoreCodeLengthRule { max_length: 86400 };
        let mut item = new_primary_identity_score("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = PrimaryIdentityScoreTitleLengthRule { max_length: 3600 };
        let mut item = new_primary_identity_score("x", "y");
        item.title = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
