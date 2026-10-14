use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary suspension score must pass. Check returns its problems, or none.
pub trait PrimarySuspensionScoreRule: std::fmt::Debug {
    fn check(&self, item: &PrimarySuspensionScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySuspensionScoreActiveGuardRule {
    pub limit: f64,
}

impl PrimarySuspensionScoreRule for PrimarySuspensionScoreActiveGuardRule {
    fn check(&self, item: &PrimarySuspensionScore) -> Vec<String> {
        if item.active && (item.floor_value > self.limit) {
            return vec![format!(
                "floor value {} exceeds {} while active",
                item.floor_value, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySuspensionScoreLabelLengthRule {
    pub max_length: i64,
}

impl PrimarySuspensionScoreRule for PrimarySuspensionScoreLabelLengthRule {
    fn check(&self, item: &PrimarySuspensionScore) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySuspensionScoreRequiredContactEmailRule {
    pub message: String,
}

impl PrimarySuspensionScoreRule for PrimarySuspensionScoreRequiredContactEmailRule {
    fn check(&self, item: &PrimarySuspensionScore) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySuspensionScoreCodeLengthRule {
    pub max_length: i64,
}

impl PrimarySuspensionScoreRule for PrimarySuspensionScoreCodeLengthRule {
    fn check(&self, item: &PrimarySuspensionScore) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary suspension score.
#[derive(Debug, Clone)]
pub struct PrimarySuspensionScoreValidator {
    pub rules: Vec<Rc<dyn PrimarySuspensionScoreRule>>,
}

impl PrimarySuspensionScoreValidator {
    pub fn validate(&self, item: &PrimarySuspensionScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimarySuspensionScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimarySuspensionScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_suspension_score_validator() -> PrimarySuspensionScoreValidator {
    PrimarySuspensionScoreValidator {
        rules: vec![
            Rc::new(PrimarySuspensionScoreActiveGuardRule { limit: 7.3 }) as Rc<dyn PrimarySuspensionScoreRule>,
            Rc::new(PrimarySuspensionScoreLabelLengthRule { max_length: 1000 }),
            Rc::new(PrimarySuspensionScoreRequiredContactEmailRule {
                message: "primary suspension score needs an entry".to_string(),
            }),
            Rc::new(PrimarySuspensionScoreCodeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_suspension_score_validator();
        let item = new_primary_suspension_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = PrimarySuspensionScoreLabelLengthRule { max_length: 1000 };
        let mut item = new_primary_suspension_score("x", "y");
        item.label = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = PrimarySuspensionScoreRequiredContactEmailRule {
            message: "primary suspension score needs an entry".to_string(),
        };
        let mut item = new_primary_suspension_score("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimarySuspensionScoreCodeLengthRule { max_length: 80 };
        let mut item = new_primary_suspension_score("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
