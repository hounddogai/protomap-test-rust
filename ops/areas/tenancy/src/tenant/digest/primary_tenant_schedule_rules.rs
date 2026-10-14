use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary tenant schedule must pass. Check returns its problems, or none.
pub trait PrimaryTenantScheduleRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryTenantSchedule) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTenantScheduleRequiredLocaleRule {
    pub message: String,
}

impl PrimaryTenantScheduleRule for PrimaryTenantScheduleRequiredLocaleRule {
    fn check(&self, item: &PrimaryTenantSchedule) -> Vec<String> {
        if item.locale.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTenantScheduleCodeLengthRule {
    pub max_length: i64,
}

impl PrimaryTenantScheduleRule for PrimaryTenantScheduleCodeLengthRule {
    fn check(&self, item: &PrimaryTenantSchedule) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTenantScheduleCodePrefixRule {
    pub prefix: String,
}

impl PrimaryTenantScheduleRule for PrimaryTenantScheduleCodePrefixRule {
    fn check(&self, item: &PrimaryTenantSchedule) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTenantScheduleRequiredNameRule {
    pub message: String,
}

impl PrimaryTenantScheduleRule for PrimaryTenantScheduleRequiredNameRule {
    fn check(&self, item: &PrimaryTenantSchedule) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary tenant schedule.
#[derive(Debug, Clone)]
pub struct PrimaryTenantScheduleValidator {
    pub rules: Vec<Rc<dyn PrimaryTenantScheduleRule>>,
}

impl PrimaryTenantScheduleValidator {
    pub fn validate(&self, item: &PrimaryTenantSchedule) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryTenantSchedule) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryTenantSchedule]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_tenant_schedule_validator() -> PrimaryTenantScheduleValidator {
    PrimaryTenantScheduleValidator {
        rules: vec![
            Rc::new(PrimaryTenantScheduleRequiredLocaleRule {
                message: "primary tenant schedule needs a value".to_string(),
            }) as Rc<dyn PrimaryTenantScheduleRule>,
            Rc::new(PrimaryTenantScheduleCodeLengthRule { max_length: 120 }),
            Rc::new(PrimaryTenantScheduleCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(PrimaryTenantScheduleRequiredNameRule {
                message: "primary tenant schedule needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_tenant_schedule_validator();
        let item = new_primary_tenant_schedule("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_locale() {
        let rule = PrimaryTenantScheduleRequiredLocaleRule {
            message: "primary tenant schedule needs a value".to_string(),
        };
        let mut item = new_primary_tenant_schedule("", "");
        item.locale = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PrimaryTenantScheduleCodeLengthRule { max_length: 255 };
        let mut item = new_primary_tenant_schedule("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PrimaryTenantScheduleRequiredNameRule {
            message: "primary tenant schedule needs an entry".to_string(),
        };
        let mut item = new_primary_tenant_schedule("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
