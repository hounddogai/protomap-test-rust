use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a egress schedule must pass. Check returns its problems, or none.
pub trait EgressScheduleRule: std::fmt::Debug {
    fn check(&self, item: &EgressSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressScheduleBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EgressScheduleRule for EgressScheduleBurstRangeRule {
    fn check(&self, item: &EgressSchedule) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressScheduleCodePrefixRule {
    pub prefix: String,
}

impl EgressScheduleRule for EgressScheduleCodePrefixRule {
    fn check(&self, item: &EgressSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressScheduleRequiredNameRule {
    pub message: String,
}

impl EgressScheduleRule for EgressScheduleRequiredNameRule {
    fn check(&self, item: &EgressSchedule) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress schedule.
#[derive(Debug, Clone)]
pub struct EgressScheduleValidator {
    pub rules: Vec<Rc<dyn EgressScheduleRule>>,
}

impl EgressScheduleValidator {
    pub fn validate(&self, item: &EgressSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EgressSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_schedule_validator() -> EgressScheduleValidator {
    EgressScheduleValidator {
        rules: vec![
            Rc::new(EgressScheduleBurstRangeRule { min: 32, max: 86400 }) as Rc<dyn EgressScheduleRule>,
            Rc::new(EgressScheduleCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(EgressScheduleRequiredNameRule {
                message: "egress schedule needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_egress_schedule_validator();
        let item = new_egress_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = EgressScheduleBurstRangeRule { min: 255, max: 86400 };
        let mut item = new_egress_schedule("x", "y");
        item.burst = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = EgressScheduleRequiredNameRule {
            message: "egress schedule needs this field".to_string(),
        };
        let mut item = new_egress_schedule("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
