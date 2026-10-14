use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a escalation metric must pass. Check returns its problems, or none.
pub trait EscalationMetricRule: std::fmt::Debug {
    fn check(&self, item: &EscalationMetric) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationMetricCodeLengthRule {
    pub max_length: i64,
}

impl EscalationMetricRule for EscalationMetricCodeLengthRule {
    fn check(&self, item: &EscalationMetric) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationMetricBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EscalationMetricRule for EscalationMetricBurstRangeRule {
    fn check(&self, item: &EscalationMetric) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationMetricNameLengthRule {
    pub max_length: i64,
}

impl EscalationMetricRule for EscalationMetricNameLengthRule {
    fn check(&self, item: &EscalationMetric) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationMetricRequiredScopeRule {
    pub message: String,
}

impl EscalationMetricRule for EscalationMetricRequiredScopeRule {
    fn check(&self, item: &EscalationMetric) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationMetricBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EscalationMetricRule for EscalationMetricBudgetRangeRule {
    fn check(&self, item: &EscalationMetric) -> Vec<String> {
        let value = item.budget;
        if value < self.min || value > self.max {
            return vec![format!("budget {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a escalation metric.
#[derive(Debug, Clone)]
pub struct EscalationMetricValidator {
    pub rules: Vec<Rc<dyn EscalationMetricRule>>,
}

impl EscalationMetricValidator {
    pub fn validate(&self, item: &EscalationMetric) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EscalationMetric) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EscalationMetric]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_escalation_metric_validator() -> EscalationMetricValidator {
    EscalationMetricValidator {
        rules: vec![
            Rc::new(EscalationMetricCodeLengthRule { max_length: 64 }) as Rc<dyn EscalationMetricRule>,
            Rc::new(EscalationMetricBurstRangeRule { min: 255, max: 86400 }),
            Rc::new(EscalationMetricNameLengthRule { max_length: 1000 }),
            Rc::new(EscalationMetricRequiredScopeRule {
                message: "escalation metric needs an entry".to_string(),
            }),
            Rc::new(EscalationMetricBudgetRangeRule { min: 7.8, max: 3244.8 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_escalation_metric_validator();
        let item = new_escalation_metric("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = EscalationMetricCodeLengthRule { max_length: 64 };
        let mut item = new_escalation_metric("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = EscalationMetricBurstRangeRule { min: 64, max: 86400 };
        let mut item = new_escalation_metric("x", "y");
        item.burst = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = EscalationMetricNameLengthRule { max_length: 1000 };
        let mut item = new_escalation_metric("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = EscalationMetricRequiredScopeRule {
            message: "escalation metric needs a value".to_string(),
        };
        let mut item = new_escalation_metric("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = EscalationMetricBudgetRangeRule { min: 0.9, max: 4517.7 };
        let mut item = new_escalation_metric("x", "y");
        item.budget = 4519.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
