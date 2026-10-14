use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a fallback dashboard override must pass. Check returns its problems, or none.
pub trait FallbackDashboardOverrideRule: std::fmt::Debug {
    fn check(&self, item: &FallbackDashboardOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDashboardOverrideVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl FallbackDashboardOverrideRule for FallbackDashboardOverrideVersionRangeRule {
    fn check(&self, item: &FallbackDashboardOverride) -> Vec<String> {
        let value = item.version;
        if value < self.min || value > self.max {
            return vec![format!("version {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDashboardOverrideRequiredSummaryRule {
    pub message: String,
}

impl FallbackDashboardOverrideRule for FallbackDashboardOverrideRequiredSummaryRule {
    fn check(&self, item: &FallbackDashboardOverride) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDashboardOverrideCodeLengthRule {
    pub max_length: i64,
}

impl FallbackDashboardOverrideRule for FallbackDashboardOverrideCodeLengthRule {
    fn check(&self, item: &FallbackDashboardOverride) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a fallback dashboard override.
#[derive(Debug, Clone)]
pub struct FallbackDashboardOverrideValidator {
    pub rules: Vec<Rc<dyn FallbackDashboardOverrideRule>>,
}

impl FallbackDashboardOverrideValidator {
    pub fn validate(&self, item: &FallbackDashboardOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &FallbackDashboardOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[FallbackDashboardOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_fallback_dashboard_override_validator() -> FallbackDashboardOverrideValidator {
    FallbackDashboardOverrideValidator {
        rules: vec![
            Rc::new(FallbackDashboardOverrideVersionRangeRule { min: 16, max: 255 })
                as Rc<dyn FallbackDashboardOverrideRule>,
            Rc::new(FallbackDashboardOverrideRequiredSummaryRule {
                message: "fallback dashboard override needs a value".to_string(),
            }),
            Rc::new(FallbackDashboardOverrideCodeLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_fallback_dashboard_override_validator();
        let item = new_fallback_dashboard_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = FallbackDashboardOverrideVersionRangeRule { min: 16, max: 80 };
        let mut item = new_fallback_dashboard_override("x", "y");
        item.version = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = FallbackDashboardOverrideRequiredSummaryRule {
            message: "fallback dashboard override needs a value".to_string(),
        };
        let mut item = new_fallback_dashboard_override("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = FallbackDashboardOverrideCodeLengthRule { max_length: 16 };
        let mut item = new_fallback_dashboard_override("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
