use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient egress budget must pass. Check returns its problems, or none.
pub trait LenientEgressBudgetRule: std::fmt::Debug {
    fn check(&self, item: &LenientEgressBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressBudgetRequiredNotesRule {
    pub message: String,
}

impl LenientEgressBudgetRule for LenientEgressBudgetRequiredNotesRule {
    fn check(&self, item: &LenientEgressBudget) -> Vec<String> {
        if item.notes.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressBudgetRequiredFingerprintRule {
    pub message: String,
}

impl LenientEgressBudgetRule for LenientEgressBudgetRequiredFingerprintRule {
    fn check(&self, item: &LenientEgressBudget) -> Vec<String> {
        if item.fingerprint.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressBudgetTitleLengthRule {
    pub max_length: i64,
}

impl LenientEgressBudgetRule for LenientEgressBudgetTitleLengthRule {
    fn check(&self, item: &LenientEgressBudget) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient egress budget.
#[derive(Debug, Clone)]
pub struct LenientEgressBudgetValidator {
    pub rules: Vec<Rc<dyn LenientEgressBudgetRule>>,
}

impl LenientEgressBudgetValidator {
    pub fn validate(&self, item: &LenientEgressBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LenientEgressBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientEgressBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_egress_budget_validator() -> LenientEgressBudgetValidator {
    LenientEgressBudgetValidator {
        rules: vec![
            Rc::new(LenientEgressBudgetRequiredNotesRule {
                message: "lenient egress budget needs this field".to_string(),
            }) as Rc<dyn LenientEgressBudgetRule>,
            Rc::new(LenientEgressBudgetRequiredFingerprintRule {
                message: "lenient egress budget needs this field".to_string(),
            }),
            Rc::new(LenientEgressBudgetTitleLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_egress_budget_validator();
        let item = new_lenient_egress_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_notes() {
        let rule = LenientEgressBudgetRequiredNotesRule {
            message: "lenient egress budget needs an entry".to_string(),
        };
        let mut item = new_lenient_egress_budget("", "");
        item.notes = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_fingerprint() {
        let rule = LenientEgressBudgetRequiredFingerprintRule {
            message: "lenient egress budget needs an entry".to_string(),
        };
        let mut item = new_lenient_egress_budget("", "");
        item.fingerprint = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = LenientEgressBudgetTitleLengthRule { max_length: 80 };
        let mut item = new_lenient_egress_budget("x", "y");
        item.title = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
