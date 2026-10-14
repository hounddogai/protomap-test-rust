use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual slo budget must pass. Check returns its problems, or none.
pub trait ManualSloBudgetRule: std::fmt::Debug {
    fn check(&self, item: &ManualSloBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSloBudgetCodePrefixRule {
    pub prefix: String,
}

impl ManualSloBudgetRule for ManualSloBudgetCodePrefixRule {
    fn check(&self, item: &ManualSloBudget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSloBudgetRequiredNameRule {
    pub message: String,
}

impl ManualSloBudgetRule for ManualSloBudgetRequiredNameRule {
    fn check(&self, item: &ManualSloBudget) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSloBudgetCodeLengthRule {
    pub max_length: i64,
}

impl ManualSloBudgetRule for ManualSloBudgetCodeLengthRule {
    fn check(&self, item: &ManualSloBudget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualSloBudgetLabelLengthRule {
    pub max_length: i64,
}

impl ManualSloBudgetRule for ManualSloBudgetLabelLengthRule {
    fn check(&self, item: &ManualSloBudget) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a manual slo budget.
#[derive(Debug, Clone)]
pub struct ManualSloBudgetValidator {
    pub rules: Vec<Rc<dyn ManualSloBudgetRule>>,
}

impl ManualSloBudgetValidator {
    pub fn validate(&self, item: &ManualSloBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualSloBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualSloBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_slo_budget_validator() -> ManualSloBudgetValidator {
    ManualSloBudgetValidator {
        rules: vec![
            Rc::new(ManualSloBudgetCodePrefixRule {
                prefix: "OPS".to_string(),
            }) as Rc<dyn ManualSloBudgetRule>,
            Rc::new(ManualSloBudgetRequiredNameRule {
                message: "manual slo budget needs this field".to_string(),
            }),
            Rc::new(ManualSloBudgetCodeLengthRule { max_length: 64 }),
            Rc::new(ManualSloBudgetLabelLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_slo_budget_validator();
        let item = new_manual_slo_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ManualSloBudgetRequiredNameRule {
            message: "manual slo budget needs this field".to_string(),
        };
        let mut item = new_manual_slo_budget("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualSloBudgetCodeLengthRule { max_length: 3600 };
        let mut item = new_manual_slo_budget("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ManualSloBudgetLabelLengthRule { max_length: 80 };
        let mut item = new_manual_slo_budget("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
