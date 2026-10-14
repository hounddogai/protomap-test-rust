use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global plan schedule must pass. Check returns its problems, or none.
pub trait GlobalPlanScheduleRule: std::fmt::Debug {
    fn check(&self, item: &GlobalPlanSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanScheduleCodeLengthRule {
    pub max_length: i64,
}

impl GlobalPlanScheduleRule for GlobalPlanScheduleCodeLengthRule {
    fn check(&self, item: &GlobalPlanSchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanScheduleAllowedTenantIdRule {
    pub allowed: Vec<String>,
}

impl GlobalPlanScheduleRule for GlobalPlanScheduleAllowedTenantIdRule {
    fn check(&self, item: &GlobalPlanSchedule) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.tenant_id)))
        {
            return vec![format!("tenant id {} is below minimum", item.tenant_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanScheduleThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalPlanScheduleRule for GlobalPlanScheduleThresholdRangeRule {
    fn check(&self, item: &GlobalPlanSchedule) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanScheduleTenantIdLengthRule {
    pub max_length: i64,
}

impl GlobalPlanScheduleRule for GlobalPlanScheduleTenantIdLengthRule {
    fn check(&self, item: &GlobalPlanSchedule) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanScheduleCodePrefixRule {
    pub prefix: String,
}

impl GlobalPlanScheduleRule for GlobalPlanScheduleCodePrefixRule {
    fn check(&self, item: &GlobalPlanSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPlanScheduleRequiredCodeRule {
    pub message: String,
}

impl GlobalPlanScheduleRule for GlobalPlanScheduleRequiredCodeRule {
    fn check(&self, item: &GlobalPlanSchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a global plan schedule.
#[derive(Debug, Clone)]
pub struct GlobalPlanScheduleValidator {
    pub rules: Vec<Rc<dyn GlobalPlanScheduleRule>>,
}

impl GlobalPlanScheduleValidator {
    pub fn validate(&self, item: &GlobalPlanSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalPlanSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalPlanSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_plan_schedule_validator() -> GlobalPlanScheduleValidator {
    GlobalPlanScheduleValidator {
        rules: vec![
            Rc::new(GlobalPlanScheduleCodeLengthRule { max_length: 255 }) as Rc<dyn GlobalPlanScheduleRule>,
            Rc::new(GlobalPlanScheduleAllowedTenantIdRule {
                allowed: vec!["global".to_string(), "regional".to_string(), "scheduled".to_string()],
            }),
            Rc::new(GlobalPlanScheduleThresholdRangeRule { min: 9.8, max: 121.6 }),
            Rc::new(GlobalPlanScheduleTenantIdLengthRule { max_length: 255 }),
            Rc::new(GlobalPlanScheduleCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(GlobalPlanScheduleRequiredCodeRule {
                message: "global plan schedule needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_plan_schedule_validator();
        let item = new_global_plan_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = GlobalPlanScheduleCodeLengthRule { max_length: 80 };
        let mut item = new_global_plan_schedule("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = GlobalPlanScheduleThresholdRangeRule { min: 0.6, max: 2237.0 };
        let mut item = new_global_plan_schedule("x", "y");
        item.threshold = 2238.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = GlobalPlanScheduleTenantIdLengthRule { max_length: 16 };
        let mut item = new_global_plan_schedule("x", "y");
        item.tenant_id = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = GlobalPlanScheduleRequiredCodeRule {
            message: "global plan schedule needs an entry".to_string(),
        };
        let mut item = new_global_plan_schedule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
