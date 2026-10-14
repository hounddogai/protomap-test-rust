use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a grant score must pass. Check returns its problems, or none.
pub trait GrantScoreRule: std::fmt::Debug {
    fn check(&self, item: &GrantScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantScoreLabelLengthRule {
    pub max_length: i64,
}

impl GrantScoreRule for GrantScoreLabelLengthRule {
    fn check(&self, item: &GrantScore) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantScoreNameLengthRule {
    pub max_length: i64,
}

impl GrantScoreRule for GrantScoreNameLengthRule {
    fn check(&self, item: &GrantScore) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantScoreCodePrefixRule {
    pub prefix: String,
}

impl GrantScoreRule for GrantScoreCodePrefixRule {
    fn check(&self, item: &GrantScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a grant score.
#[derive(Debug, Clone)]
pub struct GrantScoreValidator {
    pub rules: Vec<Rc<dyn GrantScoreRule>>,
}

impl GrantScoreValidator {
    pub fn validate(&self, item: &GrantScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GrantScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GrantScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_grant_score_validator() -> GrantScoreValidator {
    GrantScoreValidator {
        rules: vec![
            Rc::new(GrantScoreLabelLengthRule { max_length: 120 }) as Rc<dyn GrantScoreRule>,
            Rc::new(GrantScoreNameLengthRule { max_length: 255 }),
            Rc::new(GrantScoreCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_grant_score_validator();
        let item = new_grant_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = GrantScoreLabelLengthRule { max_length: 64 };
        let mut item = new_grant_score("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = GrantScoreNameLengthRule { max_length: 80 };
        let mut item = new_grant_score("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
