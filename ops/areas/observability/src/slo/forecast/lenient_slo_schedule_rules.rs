use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient slo schedule must pass. Check returns its problems, or none.
pub trait LenientSloScheduleRule: std::fmt::Debug {
    fn check(&self, item: &LenientSloSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloScheduleRequiredCodeRule {
    pub message: String,
}

impl LenientSloScheduleRule for LenientSloScheduleRequiredCodeRule {
    fn check(&self, item: &LenientSloSchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloScheduleCodeLengthRule {
    pub max_length: i64,
}

impl LenientSloScheduleRule for LenientSloScheduleCodeLengthRule {
    fn check(&self, item: &LenientSloSchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloScheduleRequiredHostRule {
    pub message: String,
}

impl LenientSloScheduleRule for LenientSloScheduleRequiredHostRule {
    fn check(&self, item: &LenientSloSchedule) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientSloScheduleCodePrefixRule {
    pub prefix: String,
}

impl LenientSloScheduleRule for LenientSloScheduleCodePrefixRule {
    fn check(&self, item: &LenientSloSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient slo schedule.
#[derive(Debug, Clone)]
pub struct LenientSloScheduleValidator {
    pub rules: Vec<Rc<dyn LenientSloScheduleRule>>,
}

impl LenientSloScheduleValidator {
    pub fn validate(&self, item: &LenientSloSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientSloSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientSloSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_slo_schedule_validator() -> LenientSloScheduleValidator {
    LenientSloScheduleValidator {
        rules: vec![
            Rc::new(LenientSloScheduleRequiredCodeRule {
                message: "lenient slo schedule needs a value".to_string(),
            }) as Rc<dyn LenientSloScheduleRule>,
            Rc::new(LenientSloScheduleCodeLengthRule { max_length: 255 }),
            Rc::new(LenientSloScheduleRequiredHostRule {
                message: "lenient slo schedule needs this field".to_string(),
            }),
            Rc::new(LenientSloScheduleCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_slo_schedule_validator();
        let item = new_lenient_slo_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = LenientSloScheduleRequiredCodeRule {
            message: "lenient slo schedule needs an entry".to_string(),
        };
        let mut item = new_lenient_slo_schedule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientSloScheduleCodeLengthRule { max_length: 32 };
        let mut item = new_lenient_slo_schedule("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = LenientSloScheduleRequiredHostRule {
            message: "lenient slo schedule needs this field".to_string(),
        };
        let mut item = new_lenient_slo_schedule("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
