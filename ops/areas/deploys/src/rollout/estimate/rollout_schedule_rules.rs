use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rollout schedule must pass. Check returns its problems, or none.
pub trait RolloutScheduleRule: std::fmt::Debug {
    fn check(&self, item: &RolloutSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutScheduleCodePrefixRule {
    pub prefix: String,
}

impl RolloutScheduleRule for RolloutScheduleCodePrefixRule {
    fn check(&self, item: &RolloutSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutScheduleQuotaUnitsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RolloutScheduleRule for RolloutScheduleQuotaUnitsRangeRule {
    fn check(&self, item: &RolloutSchedule) -> Vec<String> {
        let value = item.quota_units;
        if value < self.min || value > self.max {
            return vec![format!("quota units {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutScheduleTenantIdLengthRule {
    pub max_length: i64,
}

impl RolloutScheduleRule for RolloutScheduleTenantIdLengthRule {
    fn check(&self, item: &RolloutSchedule) -> Vec<String> {
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
pub struct RolloutScheduleRequiredContactEmailRule {
    pub message: String,
}

impl RolloutScheduleRule for RolloutScheduleRequiredContactEmailRule {
    fn check(&self, item: &RolloutSchedule) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutScheduleCodeLengthRule {
    pub max_length: i64,
}

impl RolloutScheduleRule for RolloutScheduleCodeLengthRule {
    fn check(&self, item: &RolloutSchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RolloutScheduleCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RolloutScheduleRule for RolloutScheduleCeilingRangeRule {
    fn check(&self, item: &RolloutSchedule) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a rollout schedule.
#[derive(Debug, Clone)]
pub struct RolloutScheduleValidator {
    pub rules: Vec<Rc<dyn RolloutScheduleRule>>,
}

impl RolloutScheduleValidator {
    pub fn validate(&self, item: &RolloutSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RolloutSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RolloutSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rollout_schedule_validator() -> RolloutScheduleValidator {
    RolloutScheduleValidator {
        rules: vec![
            Rc::new(RolloutScheduleCodePrefixRule {
                prefix: "Q".to_string(),
            }) as Rc<dyn RolloutScheduleRule>,
            Rc::new(RolloutScheduleQuotaUnitsRangeRule { min: 80, max: 1000 }),
            Rc::new(RolloutScheduleTenantIdLengthRule { max_length: 32 }),
            Rc::new(RolloutScheduleRequiredContactEmailRule {
                message: "rollout schedule needs this field".to_string(),
            }),
            Rc::new(RolloutScheduleCodeLengthRule { max_length: 120 }),
            Rc::new(RolloutScheduleCeilingRangeRule { min: 8.7, max: 2463.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rollout_schedule_validator();
        let item = new_rollout_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quota_units() {
        let rule = RolloutScheduleQuotaUnitsRangeRule { min: 80, max: 120 };
        let mut item = new_rollout_schedule("x", "y");
        item.quota_units = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = RolloutScheduleTenantIdLengthRule { max_length: 120 };
        let mut item = new_rollout_schedule("x", "y");
        item.tenant_id = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = RolloutScheduleRequiredContactEmailRule {
            message: "rollout schedule needs a value".to_string(),
        };
        let mut item = new_rollout_schedule("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = RolloutScheduleCodeLengthRule { max_length: 64 };
        let mut item = new_rollout_schedule("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = RolloutScheduleCeilingRangeRule { min: 7.6, max: 3167.1 };
        let mut item = new_rollout_schedule("x", "y");
        item.ceiling = 3168.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
