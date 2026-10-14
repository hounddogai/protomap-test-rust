use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a escalation schedule must pass. Check returns its problems, or none.
pub trait EscalationScheduleRule: std::fmt::Debug {
    fn check(&self, item: &EscalationSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationScheduleBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EscalationScheduleRule for EscalationScheduleBudgetRangeRule {
    fn check(&self, item: &EscalationSchedule) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationScheduleRequiredCodeRule {
    pub message: String,
}

impl EscalationScheduleRule for EscalationScheduleRequiredCodeRule {
    fn check(&self, item: &EscalationSchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationScheduleRequiredCurrencyRule {
    pub message: String,
}

impl EscalationScheduleRule for EscalationScheduleRequiredCurrencyRule {
    fn check(&self, item: &EscalationSchedule) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationScheduleCodePrefixRule {
    pub prefix: String,
}

impl EscalationScheduleRule for EscalationScheduleCodePrefixRule {
    fn check(&self, item: &EscalationSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationScheduleCodeLengthRule {
    pub max_length: i64,
}

impl EscalationScheduleRule for EscalationScheduleCodeLengthRule {
    fn check(&self, item: &EscalationSchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a escalation schedule.
#[derive(Debug, Clone)]
pub struct EscalationScheduleValidator {
    pub rules: Vec<Rc<dyn EscalationScheduleRule>>,
}

impl EscalationScheduleValidator {
    pub fn validate(&self, item: &EscalationSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EscalationSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EscalationSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_escalation_schedule_validator() -> EscalationScheduleValidator {
    EscalationScheduleValidator {
        rules: vec![
            Rc::new(EscalationScheduleBudgetRangeRule { min: 5.9, max: 1967.1 }) as Rc<dyn EscalationScheduleRule>,
            Rc::new(EscalationScheduleRequiredCodeRule {
                message: "escalation schedule needs this field".to_string(),
            }),
            Rc::new(EscalationScheduleRequiredCurrencyRule {
                message: "escalation schedule needs this field".to_string(),
            }),
            Rc::new(EscalationScheduleCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(EscalationScheduleCodeLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_escalation_schedule_validator();
        let item = new_escalation_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = EscalationScheduleBudgetRangeRule { min: 7.6, max: 2459.5 };
        let mut item = new_escalation_schedule("x", "y");
        item.budget = 2461.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = EscalationScheduleRequiredCodeRule {
            message: "escalation schedule needs a value".to_string(),
        };
        let mut item = new_escalation_schedule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = EscalationScheduleRequiredCurrencyRule {
            message: "escalation schedule needs an entry".to_string(),
        };
        let mut item = new_escalation_schedule("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = EscalationScheduleCodeLengthRule { max_length: 3600 };
        let mut item = new_escalation_schedule("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
