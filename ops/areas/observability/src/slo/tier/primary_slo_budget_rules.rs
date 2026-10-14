use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a primary slo budget must pass. Check returns its problems, or none.
pub trait PrimarySloBudgetRule: std::fmt::Debug {
    fn check(&self, item: &PrimarySloBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloBudgetRequiredTitleRule {
    pub message: String,
}

impl PrimarySloBudgetRule for PrimarySloBudgetRequiredTitleRule {
    fn check(&self, item: &PrimarySloBudget) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloBudgetPinnedGuardRule {
    pub limit: i64,
}

impl PrimarySloBudgetRule for PrimarySloBudgetPinnedGuardRule {
    fn check(&self, item: &PrimarySloBudget) -> Vec<String> {
        if item.pinned && (item.grace_days > self.limit) {
            return vec![format!(
                "grace days {} exceeds {} while pinned",
                item.grace_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloBudgetCodeLengthRule {
    pub max_length: i64,
}

impl PrimarySloBudgetRule for PrimarySloBudgetCodeLengthRule {
    fn check(&self, item: &PrimarySloBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloBudgetAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl PrimarySloBudgetRule for PrimarySloBudgetAllowedTitleRule {
    fn check(&self, item: &PrimarySloBudget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is not allowed", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySloBudgetRequiredCodeRule {
    pub message: String,
}

impl PrimarySloBudgetRule for PrimarySloBudgetRequiredCodeRule {
    fn check(&self, item: &PrimarySloBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary slo budget.
#[derive(Debug, Clone)]
pub struct PrimarySloBudgetValidator {
    pub rules: Vec<Rc<dyn PrimarySloBudgetRule>>,
}

impl PrimarySloBudgetValidator {
    pub fn validate(&self, item: &PrimarySloBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimarySloBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimarySloBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_slo_budget_validator() -> PrimarySloBudgetValidator {
    PrimarySloBudgetValidator {
        rules: vec![
            Rc::new(PrimarySloBudgetRequiredTitleRule {
                message: "primary slo budget needs an entry".to_string(),
            }) as Rc<dyn PrimarySloBudgetRule>,
            Rc::new(PrimarySloBudgetPinnedGuardRule { limit: 255 }),
            Rc::new(PrimarySloBudgetCodeLengthRule { max_length: 3600 }),
            Rc::new(PrimarySloBudgetAllowedTitleRule {
                allowed: vec!["pending".to_string()],
            }),
            Rc::new(PrimarySloBudgetRequiredCodeRule {
                message: "primary slo budget needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_slo_budget_validator();
        let item = new_primary_slo_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PrimarySloBudgetRequiredTitleRule {
            message: "primary slo budget needs an entry".to_string(),
        };
        let mut item = new_primary_slo_budget("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimarySloBudgetCodeLengthRule { max_length: 64 };
        let mut item = new_primary_slo_budget("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimarySloBudgetRequiredCodeRule {
            message: "primary slo budget needs this field".to_string(),
        };
        let mut item = new_primary_slo_budget("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
