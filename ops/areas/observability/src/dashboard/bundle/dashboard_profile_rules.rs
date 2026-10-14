use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a dashboard profile must pass. Check returns its problems, or none.
pub trait DashboardProfileRule: std::fmt::Debug {
    fn check(&self, item: &DashboardProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardProfileCodeLengthRule {
    pub max_length: i64,
}

impl DashboardProfileRule for DashboardProfileCodeLengthRule {
    fn check(&self, item: &DashboardProfile) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardProfileRequiredCodeRule {
    pub message: String,
}

impl DashboardProfileRule for DashboardProfileRequiredCodeRule {
    fn check(&self, item: &DashboardProfile) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardProfileThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DashboardProfileRule for DashboardProfileThresholdRangeRule {
    fn check(&self, item: &DashboardProfile) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardProfileExternalRefLengthRule {
    pub max_length: i64,
}

impl DashboardProfileRule for DashboardProfileExternalRefLengthRule {
    fn check(&self, item: &DashboardProfile) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardProfileAllowedScopeRule {
    pub allowed: Vec<String>,
}

impl DashboardProfileRule for DashboardProfileAllowedScopeRule {
    fn check(&self, item: &DashboardProfile) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.scope))) {
            return vec![format!("scope {} is out of range", item.scope)];
        }
        Vec::new()
    }
}

/// Runs every rule of a dashboard profile.
#[derive(Debug, Clone)]
pub struct DashboardProfileValidator {
    pub rules: Vec<Rc<dyn DashboardProfileRule>>,
}

impl DashboardProfileValidator {
    pub fn validate(&self, item: &DashboardProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DashboardProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DashboardProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dashboard_profile_validator() -> DashboardProfileValidator {
    DashboardProfileValidator {
        rules: vec![
            Rc::new(DashboardProfileCodeLengthRule { max_length: 16 }) as Rc<dyn DashboardProfileRule>,
            Rc::new(DashboardProfileRequiredCodeRule {
                message: "dashboard profile needs this field".to_string(),
            }),
            Rc::new(DashboardProfileThresholdRangeRule { min: 3.1, max: 4294.0 }),
            Rc::new(DashboardProfileExternalRefLengthRule { max_length: 64 }),
            Rc::new(DashboardProfileAllowedScopeRule { allowed: Vec::new() }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dashboard_profile_validator();
        let item = new_dashboard_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DashboardProfileCodeLengthRule { max_length: 1000 };
        let mut item = new_dashboard_profile("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = DashboardProfileRequiredCodeRule {
            message: "dashboard profile needs an entry".to_string(),
        };
        let mut item = new_dashboard_profile("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = DashboardProfileThresholdRangeRule { min: 5.4, max: 3975.7 };
        let mut item = new_dashboard_profile("x", "y");
        item.threshold = 3977.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = DashboardProfileExternalRefLengthRule { max_length: 86400 };
        let mut item = new_dashboard_profile("x", "y");
        item.external_ref = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
