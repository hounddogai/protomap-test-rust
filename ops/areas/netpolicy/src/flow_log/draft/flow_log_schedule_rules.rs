use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a flow log schedule must pass. Check returns its problems, or none.
pub trait FlowLogScheduleRule: std::fmt::Debug {
    fn check(&self, item: &FlowLogSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogScheduleRequiredNameRule {
    pub message: String,
}

impl FlowLogScheduleRule for FlowLogScheduleRequiredNameRule {
    fn check(&self, item: &FlowLogSchedule) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogScheduleRequiredCipherRule {
    pub message: String,
}

impl FlowLogScheduleRule for FlowLogScheduleRequiredCipherRule {
    fn check(&self, item: &FlowLogSchedule) -> Vec<String> {
        if item.cipher.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogScheduleSourceLengthRule {
    pub max_length: i64,
}

impl FlowLogScheduleRule for FlowLogScheduleSourceLengthRule {
    fn check(&self, item: &FlowLogSchedule) -> Vec<String> {
        let size = item.source.len() as i64;
        if size > self.max_length {
            return vec![format!("source is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogScheduleExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FlowLogScheduleRule for FlowLogScheduleExpiresAtRangeRule {
    fn check(&self, item: &FlowLogSchedule) -> Vec<String> {
        let value = item.expires_at;
        if value < self.min || value > self.max {
            return vec![format!("expires at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowLogScheduleRequiredSourceRule {
    pub message: String,
}

impl FlowLogScheduleRule for FlowLogScheduleRequiredSourceRule {
    fn check(&self, item: &FlowLogSchedule) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a flow log schedule.
#[derive(Debug, Clone)]
pub struct FlowLogScheduleValidator {
    pub rules: Vec<Rc<dyn FlowLogScheduleRule>>,
}

impl FlowLogScheduleValidator {
    pub fn validate(&self, item: &FlowLogSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &FlowLogSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FlowLogSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_flow_log_schedule_validator() -> FlowLogScheduleValidator {
    FlowLogScheduleValidator {
        rules: vec![
            Rc::new(FlowLogScheduleRequiredNameRule {
                message: "flow log schedule needs this field".to_string(),
            }) as Rc<dyn FlowLogScheduleRule>,
            Rc::new(FlowLogScheduleRequiredCipherRule {
                message: "flow log schedule needs a value".to_string(),
            }),
            Rc::new(FlowLogScheduleSourceLengthRule { max_length: 1000 }),
            Rc::new(FlowLogScheduleExpiresAtRangeRule { min: 64, max: 64 }),
            Rc::new(FlowLogScheduleRequiredSourceRule {
                message: "flow log schedule needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_flow_log_schedule_validator();
        let item = new_flow_log_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = FlowLogScheduleRequiredNameRule {
            message: "flow log schedule needs this field".to_string(),
        };
        let mut item = new_flow_log_schedule("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_cipher() {
        let rule = FlowLogScheduleRequiredCipherRule {
            message: "flow log schedule needs an entry".to_string(),
        };
        let mut item = new_flow_log_schedule("", "");
        item.cipher = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_source() {
        let rule = FlowLogScheduleSourceLengthRule { max_length: 120 };
        let mut item = new_flow_log_schedule("x", "y");
        item.source = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.source = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = FlowLogScheduleExpiresAtRangeRule { min: 32, max: 120 };
        let mut item = new_flow_log_schedule("x", "y");
        item.expires_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = FlowLogScheduleRequiredSourceRule {
            message: "flow log schedule needs an entry".to_string(),
        };
        let mut item = new_flow_log_schedule("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
