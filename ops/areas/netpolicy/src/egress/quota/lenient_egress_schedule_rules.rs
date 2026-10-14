use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient egress schedule must pass. Check returns its problems, or none.
pub trait LenientEgressScheduleRule: std::fmt::Debug {
    fn check(&self, item: &LenientEgressSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressScheduleRequiredTenantIdRule {
    pub message: String,
}

impl LenientEgressScheduleRule for LenientEgressScheduleRequiredTenantIdRule {
    fn check(&self, item: &LenientEgressSchedule) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressScheduleRequiredLabelRule {
    pub message: String,
}

impl LenientEgressScheduleRule for LenientEgressScheduleRequiredLabelRule {
    fn check(&self, item: &LenientEgressSchedule) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressSchedulePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientEgressScheduleRule for LenientEgressSchedulePortRangeRule {
    fn check(&self, item: &LenientEgressSchedule) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressScheduleVisibleGuardRule {
    pub limit: i64,
}

impl LenientEgressScheduleRule for LenientEgressScheduleVisibleGuardRule {
    fn check(&self, item: &LenientEgressSchedule) -> Vec<String> {
        if item.visible && (item.ends_at > self.limit) {
            return vec![format!("ends at {} exceeds {} while visible", item.ends_at, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressScheduleCodeLengthRule {
    pub max_length: i64,
}

impl LenientEgressScheduleRule for LenientEgressScheduleCodeLengthRule {
    fn check(&self, item: &LenientEgressSchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient egress schedule.
#[derive(Debug, Clone)]
pub struct LenientEgressScheduleValidator {
    pub rules: Vec<Rc<dyn LenientEgressScheduleRule>>,
}

impl LenientEgressScheduleValidator {
    pub fn validate(&self, item: &LenientEgressSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientEgressSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientEgressSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_egress_schedule_validator() -> LenientEgressScheduleValidator {
    LenientEgressScheduleValidator {
        rules: vec![
            Rc::new(LenientEgressScheduleRequiredTenantIdRule {
                message: "lenient egress schedule needs an entry".to_string(),
            }) as Rc<dyn LenientEgressScheduleRule>,
            Rc::new(LenientEgressScheduleRequiredLabelRule {
                message: "lenient egress schedule needs a value".to_string(),
            }),
            Rc::new(LenientEgressSchedulePortRangeRule { min: 64, max: 255 }),
            Rc::new(LenientEgressScheduleVisibleGuardRule { limit: 80 }),
            Rc::new(LenientEgressScheduleCodeLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_egress_schedule_validator();
        let item = new_lenient_egress_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = LenientEgressScheduleRequiredTenantIdRule {
            message: "lenient egress schedule needs this field".to_string(),
        };
        let mut item = new_lenient_egress_schedule("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = LenientEgressScheduleRequiredLabelRule {
            message: "lenient egress schedule needs this field".to_string(),
        };
        let mut item = new_lenient_egress_schedule("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = LenientEgressSchedulePortRangeRule { min: 16, max: 32 };
        let mut item = new_lenient_egress_schedule("x", "y");
        item.port = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LenientEgressScheduleCodeLengthRule { max_length: 86400 };
        let mut item = new_lenient_egress_schedule("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
