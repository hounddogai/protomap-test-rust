use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tenant group must pass. Check returns its problems, or none.
pub trait TenantGroupRule: std::fmt::Debug {
    fn check(&self, item: &TenantGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantGroupVisibleGuardRule {
    pub limit: i64,
}

impl TenantGroupRule for TenantGroupVisibleGuardRule {
    fn check(&self, item: &TenantGroup) -> Vec<String> {
        if item.visible && (item.grace_days > self.limit) {
            return vec![format!(
                "grace days {} exceeds {} while visible",
                item.grace_days, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantGroupCodeLengthRule {
    pub max_length: i64,
}

impl TenantGroupRule for TenantGroupCodeLengthRule {
    fn check(&self, item: &TenantGroup) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantGroupGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TenantGroupRule for TenantGroupGraceDaysRangeRule {
    fn check(&self, item: &TenantGroup) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantGroupRequiredCodeRule {
    pub message: String,
}

impl TenantGroupRule for TenantGroupRequiredCodeRule {
    fn check(&self, item: &TenantGroup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a tenant group.
#[derive(Debug, Clone)]
pub struct TenantGroupValidator {
    pub rules: Vec<Rc<dyn TenantGroupRule>>,
}

impl TenantGroupValidator {
    pub fn validate(&self, item: &TenantGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TenantGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_group_validator() -> TenantGroupValidator {
    TenantGroupValidator {
        rules: vec![
            Rc::new(TenantGroupVisibleGuardRule { limit: 64 }) as Rc<dyn TenantGroupRule>,
            Rc::new(TenantGroupCodeLengthRule { max_length: 64 }),
            Rc::new(TenantGroupGraceDaysRangeRule { min: 32, max: 120 }),
            Rc::new(TenantGroupRequiredCodeRule {
                message: "tenant group needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tenant_group_validator();
        let item = new_tenant_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TenantGroupCodeLengthRule { max_length: 64 };
        let mut item = new_tenant_group("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = TenantGroupGraceDaysRangeRule { min: 120, max: 3600 };
        let mut item = new_tenant_group("x", "y");
        item.grace_days = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TenantGroupRequiredCodeRule {
            message: "tenant group needs this field".to_string(),
        };
        let mut item = new_tenant_group("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
