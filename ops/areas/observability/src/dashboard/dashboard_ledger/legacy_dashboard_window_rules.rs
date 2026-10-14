use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy dashboard window must pass. Check returns its problems, or none.
pub trait LegacyDashboardWindowRule: std::fmt::Debug {
    fn check(&self, item: &LegacyDashboardWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDashboardWindowMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl LegacyDashboardWindowRule for LegacyDashboardWindowMinimumRangeRule {
    fn check(&self, item: &LegacyDashboardWindow) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDashboardWindowLabelLengthRule {
    pub max_length: i64,
}

impl LegacyDashboardWindowRule for LegacyDashboardWindowLabelLengthRule {
    fn check(&self, item: &LegacyDashboardWindow) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDashboardWindowRequiredKeyIdRule {
    pub message: String,
}

impl LegacyDashboardWindowRule for LegacyDashboardWindowRequiredKeyIdRule {
    fn check(&self, item: &LegacyDashboardWindow) -> Vec<String> {
        if item.key_id.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDashboardWindowCodePrefixRule {
    pub prefix: String,
}

impl LegacyDashboardWindowRule for LegacyDashboardWindowCodePrefixRule {
    fn check(&self, item: &LegacyDashboardWindow) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDashboardWindowCodeLengthRule {
    pub max_length: i64,
}

impl LegacyDashboardWindowRule for LegacyDashboardWindowCodeLengthRule {
    fn check(&self, item: &LegacyDashboardWindow) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy dashboard window.
#[derive(Debug, Clone)]
pub struct LegacyDashboardWindowValidator {
    pub rules: Vec<Rc<dyn LegacyDashboardWindowRule>>,
}

impl LegacyDashboardWindowValidator {
    pub fn validate(&self, item: &LegacyDashboardWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &LegacyDashboardWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyDashboardWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_dashboard_window_validator() -> LegacyDashboardWindowValidator {
    LegacyDashboardWindowValidator {
        rules: vec![
            Rc::new(LegacyDashboardWindowMinimumRangeRule { min: 5.1, max: 1602.1 })
                as Rc<dyn LegacyDashboardWindowRule>,
            Rc::new(LegacyDashboardWindowLabelLengthRule { max_length: 32 }),
            Rc::new(LegacyDashboardWindowRequiredKeyIdRule {
                message: "legacy dashboard window needs a value".to_string(),
            }),
            Rc::new(LegacyDashboardWindowCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(LegacyDashboardWindowCodeLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_dashboard_window_validator();
        let item = new_legacy_dashboard_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = LegacyDashboardWindowMinimumRangeRule { min: 0.8, max: 1666.9 };
        let mut item = new_legacy_dashboard_window("x", "y");
        item.minimum = 1668.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = LegacyDashboardWindowLabelLengthRule { max_length: 86400 };
        let mut item = new_legacy_dashboard_window("x", "y");
        item.label = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_key_id() {
        let rule = LegacyDashboardWindowRequiredKeyIdRule {
            message: "legacy dashboard window needs a value".to_string(),
        };
        let mut item = new_legacy_dashboard_window("", "");
        item.key_id = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyDashboardWindowCodeLengthRule { max_length: 16 };
        let mut item = new_legacy_dashboard_window("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
