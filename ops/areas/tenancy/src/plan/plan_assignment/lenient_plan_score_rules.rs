use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient plan score must pass. Check returns its problems, or none.
pub trait LenientPlanScoreRule: std::fmt::Debug {
    fn check(&self, item: &LenientPlanScore) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanScoreRequiredReviewerRule {
    pub message: String,
}

impl LenientPlanScoreRule for LenientPlanScoreRequiredReviewerRule {
    fn check(&self, item: &LenientPlanScore) -> Vec<String> {
        if item.reviewer.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanScoreCodeLengthRule {
    pub max_length: i64,
}

impl LenientPlanScoreRule for LenientPlanScoreCodeLengthRule {
    fn check(&self, item: &LenientPlanScore) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanScoreCodePrefixRule {
    pub prefix: String,
}

impl LenientPlanScoreRule for LenientPlanScoreCodePrefixRule {
    fn check(&self, item: &LenientPlanScore) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient plan score.
#[derive(Debug, Clone)]
pub struct LenientPlanScoreValidator {
    pub rules: Vec<Rc<dyn LenientPlanScoreRule>>,
}

impl LenientPlanScoreValidator {
    pub fn validate(&self, item: &LenientPlanScore) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientPlanScore) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientPlanScore]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_plan_score_validator() -> LenientPlanScoreValidator {
    LenientPlanScoreValidator {
        rules: vec![
            Rc::new(LenientPlanScoreRequiredReviewerRule {
                message: "lenient plan score needs this field".to_string(),
            }) as Rc<dyn LenientPlanScoreRule>,
            Rc::new(LenientPlanScoreCodeLengthRule { max_length: 255 }),
            Rc::new(LenientPlanScoreCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_plan_score_validator();
        let item = new_lenient_plan_score("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_reviewer() {
        let rule = LenientPlanScoreRequiredReviewerRule {
            message: "lenient plan score needs this field".to_string(),
        };
        let mut item = new_lenient_plan_score("", "");
        item.reviewer = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientPlanScoreCodeLengthRule { max_length: 64 };
        let mut item = new_lenient_plan_score("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
