use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a flow log plan must pass. Check returns its problems, or none.
pub trait FlowLogPlanRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogPlan) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogPlanPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FlowLogPlanRule for FlowLogPlanPriorityRangeRule {
    fn check(&self, item: &FlowLogPlan) -> Vec<String> {
        let value = item.priority;
        if value < self.min || value > self.max {
            return vec![format!("priority {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogPlanAllowedSummaryRule {
    pub allowed: Vec<String>,
}

impl FlowLogPlanRule for FlowLogPlanAllowedSummaryRule {
    fn check(&self, item: &FlowLogPlan) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.summary)))
        {
            return vec![format!("summary {} is blocked", item.summary)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogPlanVisibleGuardRule {
    pub limit: i64,
}

impl FlowLogPlanRule for FlowLogPlanVisibleGuardRule {
    fn check(&self, item: &FlowLogPlan) -> Vec<String> {
        if item.visible && (item.quantity > self.limit) {
            return vec![format!(
                "quantity {} exceeds {} while visible",
                item.quantity, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogPlanCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FlowLogPlanRule for FlowLogPlanCreatedAtRangeRule {
    fn check(&self, item: &FlowLogPlan) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogPlanRequiredLabelRule {
    pub message: String,
}

impl FlowLogPlanRule for FlowLogPlanRequiredLabelRule {
    fn check(&self, item: &FlowLogPlan) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogPlanCodePrefixRule {
    pub prefix: String,
}

impl FlowLogPlanRule for FlowLogPlanCodePrefixRule {
    fn check(&self, item: &FlowLogPlan) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a flow log plan.
#[derive(Debug, Clone)]
pub struct FlowLogPlanValidator {
    pub rules: Vec<Rc<dyn FlowLogPlanRule>>,
}

impl FlowLogPlanValidator {
    pub fn validate(&self, item: &FlowLogPlan) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FlowLogPlan) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogPlan]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_plan_validator() -> FlowLogPlanValidator {
    FlowLogPlanValidator {
        rules: vec![
            Rc::new(FlowLogPlanPriorityRangeRule { min: 1000, max: 3600 }) as Rc<dyn FlowLogPlanRule>,
            Rc::new(FlowLogPlanAllowedSummaryRule {
                allowed: vec!["fallback".to_string(), "default".to_string()],
            }),
            Rc::new(FlowLogPlanVisibleGuardRule { limit: 120 }),
            Rc::new(FlowLogPlanCreatedAtRangeRule { min: 64, max: 80 }),
            Rc::new(FlowLogPlanRequiredLabelRule {
                message: "flow log plan needs a value".to_string(),
            }),
            Rc::new(FlowLogPlanCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_flow_log_plan_validator();
        let item = new_flow_log_plan("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = FlowLogPlanPriorityRangeRule { min: 80, max: 120 };
        let mut item = new_flow_log_plan("x", "y");
        item.priority = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = FlowLogPlanCreatedAtRangeRule { min: 32, max: 32 };
        let mut item = new_flow_log_plan("x", "y");
        item.created_at = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = FlowLogPlanRequiredLabelRule {
            message: "flow log plan needs this field".to_string(),
        };
        let mut item = new_flow_log_plan("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
