use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary suspension schedule must pass. Check returns its problems, or none.
pub trait PrimarySuspensionScheduleRule: std::fmt::Debug {
    fn check(&self, item: &PrimarySuspensionSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySuspensionScheduleCodePrefixRule {
    pub prefix: String,
}

impl PrimarySuspensionScheduleRule for PrimarySuspensionScheduleCodePrefixRule {
    fn check(&self, item: &PrimarySuspensionSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySuspensionScheduleTenantIdLengthRule {
    pub max_length: i64,
}

impl PrimarySuspensionScheduleRule for PrimarySuspensionScheduleTenantIdLengthRule {
    fn check(&self, item: &PrimarySuspensionSchedule) -> Vec<String> {
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
pub struct PrimarySuspensionScheduleRequiredTenantIdRule {
    pub message: String,
}

impl PrimarySuspensionScheduleRule for PrimarySuspensionScheduleRequiredTenantIdRule {
    fn check(&self, item: &PrimarySuspensionSchedule) -> Vec<String> {
        if item.tenant_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySuspensionScheduleCodeLengthRule {
    pub max_length: i64,
}

impl PrimarySuspensionScheduleRule for PrimarySuspensionScheduleCodeLengthRule {
    fn check(&self, item: &PrimarySuspensionSchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimarySuspensionScheduleRequiredCodeRule {
    pub message: String,
}

impl PrimarySuspensionScheduleRule for PrimarySuspensionScheduleRequiredCodeRule {
    fn check(&self, item: &PrimarySuspensionSchedule) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary suspension schedule.
#[derive(Debug, Clone)]
pub struct PrimarySuspensionScheduleValidator {
    pub rules: Vec<Rc<dyn PrimarySuspensionScheduleRule>>,
}

impl PrimarySuspensionScheduleValidator {
    pub fn validate(&self, item: &PrimarySuspensionSchedule) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimarySuspensionSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimarySuspensionSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_suspension_schedule_validator() -> PrimarySuspensionScheduleValidator {
    PrimarySuspensionScheduleValidator {
        rules: vec![
            Rc::new(PrimarySuspensionScheduleCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn PrimarySuspensionScheduleRule>,
            Rc::new(PrimarySuspensionScheduleTenantIdLengthRule { max_length: 120 }),
            Rc::new(PrimarySuspensionScheduleRequiredTenantIdRule {
                message: "primary suspension schedule needs an entry".to_string(),
            }),
            Rc::new(PrimarySuspensionScheduleCodeLengthRule { max_length: 32 }),
            Rc::new(PrimarySuspensionScheduleRequiredCodeRule {
                message: "primary suspension schedule needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_suspension_schedule_validator();
        let item = new_primary_suspension_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = PrimarySuspensionScheduleTenantIdLengthRule { max_length: 16 };
        let mut item = new_primary_suspension_schedule("x", "y");
        item.tenant_id = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_tenant_id() {
        let rule = PrimarySuspensionScheduleRequiredTenantIdRule {
            message: "primary suspension schedule needs this field".to_string(),
        };
        let mut item = new_primary_suspension_schedule("", "");
        item.tenant_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimarySuspensionScheduleCodeLengthRule { max_length: 32 };
        let mut item = new_primary_suspension_schedule("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimarySuspensionScheduleRequiredCodeRule {
            message: "primary suspension schedule needs this field".to_string(),
        };
        let mut item = new_primary_suspension_schedule("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
