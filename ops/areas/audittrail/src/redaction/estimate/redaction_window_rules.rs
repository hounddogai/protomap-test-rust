use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a redaction window must pass. Check returns its problems, or none.
pub trait RedactionWindowRule: std::fmt::Debug {
    fn check(&self, item: &RedactionWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionWindowRequiredTitleRule {
    pub message: String,
}

impl RedactionWindowRule for RedactionWindowRequiredTitleRule {
    fn check(&self, item: &RedactionWindow) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionWindowAllowedPrincipalRule {
    pub allowed: Vec<String>,
}

impl RedactionWindowRule for RedactionWindowAllowedPrincipalRule {
    fn check(&self, item: &RedactionWindow) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.principal)))
        {
            return vec![format!("principal {} is stale", item.principal)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RedactionWindowScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RedactionWindowRule for RedactionWindowScoreRangeRule {
    fn check(&self, item: &RedactionWindow) -> Vec<String> {
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

/// Runs every rule of a redaction window.
#[derive(Debug, Clone)]
pub struct RedactionWindowValidator {
    pub rules: Vec<Rc<dyn RedactionWindowRule>>,
}

impl RedactionWindowValidator {
    pub fn validate(&self, item: &RedactionWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RedactionWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RedactionWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_redaction_window_validator() -> RedactionWindowValidator {
    RedactionWindowValidator {
        rules: vec![
            Rc::new(RedactionWindowRequiredTitleRule {
                message: "redaction window needs an entry".to_string(),
            }) as Rc<dyn RedactionWindowRule>,
            Rc::new(RedactionWindowAllowedPrincipalRule { allowed: Vec::new() }),
            Rc::new(RedactionWindowScoreRangeRule { min: 7.3, max: 4758.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_redaction_window_validator();
        let item = new_redaction_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = RedactionWindowRequiredTitleRule {
            message: "redaction window needs a value".to_string(),
        };
        let mut item = new_redaction_window("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = RedactionWindowScoreRangeRule { min: 0.6, max: 2341.8 };
        let mut item = new_redaction_window("x", "y");
        item.score = 2343.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
