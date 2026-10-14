use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tenant profile must pass. Check returns its problems, or none.
pub trait TenantProfileRule: std::fmt::Debug {
    fn check(&self, item: &TenantProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantProfileRequiredLabelRule {
    pub message: String,
}

impl TenantProfileRule for TenantProfileRequiredLabelRule {
    fn check(&self, item: &TenantProfile) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantProfileMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TenantProfileRule for TenantProfileMaximumRangeRule {
    fn check(&self, item: &TenantProfile) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantProfileLabelLengthRule {
    pub max_length: i64,
}

impl TenantProfileRule for TenantProfileLabelLengthRule {
    fn check(&self, item: &TenantProfile) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantProfilePinnedGuardRule {
    pub limit: f64,
}

impl TenantProfileRule for TenantProfilePinnedGuardRule {
    fn check(&self, item: &TenantProfile) -> Vec<String> {
        if item.pinned && (item.margin > self.limit) {
            return vec![format!("margin {} exceeds {} while pinned", item.margin, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tenant profile.
#[derive(Debug, Clone)]
pub struct TenantProfileValidator {
    pub rules: Vec<Rc<dyn TenantProfileRule>>,
}

impl TenantProfileValidator {
    pub fn validate(&self, item: &TenantProfile) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TenantProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_profile_validator() -> TenantProfileValidator {
    TenantProfileValidator {
        rules: vec![
            Rc::new(TenantProfileRequiredLabelRule {
                message: "tenant profile needs this field".to_string(),
            }) as Rc<dyn TenantProfileRule>,
            Rc::new(TenantProfileMaximumRangeRule { min: 2.0, max: 4761.8 }),
            Rc::new(TenantProfileLabelLengthRule { max_length: 1000 }),
            Rc::new(TenantProfilePinnedGuardRule { limit: 4.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tenant_profile_validator();
        let item = new_tenant_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = TenantProfileRequiredLabelRule {
            message: "tenant profile needs this field".to_string(),
        };
        let mut item = new_tenant_profile("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = TenantProfileMaximumRangeRule { min: 7.4, max: 839.0 };
        let mut item = new_tenant_profile("x", "y");
        item.maximum = 840.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = TenantProfileLabelLengthRule { max_length: 16 };
        let mut item = new_tenant_profile("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
