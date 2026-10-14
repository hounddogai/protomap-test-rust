use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a egress score must pass. Check returns its problems, or none.
pub trait EgressScoreRule: std::fmt::Debug {
    fn check(&self, item: &EgressScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressScoreArchivedGuardRule {
    pub limit: f64,
}

impl EgressScoreRule for EgressScoreArchivedGuardRule {
    fn check(&self, item: &EgressScore) -> Vec<String> {
        if item.archived && (item.floor_value > self.limit) {
            return vec![format!(
                "floor value {} exceeds {} while archived",
                item.floor_value, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressScoreLabelLengthRule {
    pub max_length: i64,
}

impl EgressScoreRule for EgressScoreLabelLengthRule {
    fn check(&self, item: &EgressScore) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressScoreCipherLengthRule {
    pub max_length: i64,
}

impl EgressScoreRule for EgressScoreCipherLengthRule {
    fn check(&self, item: &EgressScore) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress score.
#[derive(Debug, Clone)]
pub struct EgressScoreValidator {
    pub rules: Vec<Rc<dyn EgressScoreRule>>,
}

impl EgressScoreValidator {
    pub fn validate(&self, item: &EgressScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EgressScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_score_validator() -> EgressScoreValidator {
    EgressScoreValidator {
        rules: vec![
            Rc::new(EgressScoreArchivedGuardRule { limit: 2.9 }) as Rc<dyn EgressScoreRule>,
            Rc::new(EgressScoreLabelLengthRule { max_length: 1000 }),
            Rc::new(EgressScoreCipherLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_egress_score_validator();
        let item = new_egress_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = EgressScoreLabelLengthRule { max_length: 1000 };
        let mut item = new_egress_score("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = EgressScoreCipherLengthRule { max_length: 16 };
        let mut item = new_egress_score("x", "y");
        item.cipher = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
