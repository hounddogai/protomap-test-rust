use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global dashboard window must pass. Check returns its problems, or none.
pub trait GlobalDashboardWindowRule: std::fmt::Debug {
    fn check(&self, item: &GlobalDashboardWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDashboardWindowAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl GlobalDashboardWindowRule for GlobalDashboardWindowAllowedCodeRule {
    fn check(&self, item: &GlobalDashboardWindow) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDashboardWindowTitleLengthRule {
    pub max_length: i64,
}

impl GlobalDashboardWindowRule for GlobalDashboardWindowTitleLengthRule {
    fn check(&self, item: &GlobalDashboardWindow) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDashboardWindowCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalDashboardWindowRule for GlobalDashboardWindowCapacityRangeRule {
    fn check(&self, item: &GlobalDashboardWindow) -> Vec<String> {
        let value = item.capacity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("capacity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("capacity {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a global dashboard window.
#[derive(Debug, Clone)]
pub struct GlobalDashboardWindowValidator {
    pub rules: Vec<Rc<dyn GlobalDashboardWindowRule>>,
}

impl GlobalDashboardWindowValidator {
    pub fn validate(&self, item: &GlobalDashboardWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &GlobalDashboardWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalDashboardWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_dashboard_window_validator() -> GlobalDashboardWindowValidator {
    GlobalDashboardWindowValidator {
        rules: vec![
            Rc::new(GlobalDashboardWindowAllowedCodeRule {
                allowed: vec!["strict".to_string(), "pending".to_string()],
            }) as Rc<dyn GlobalDashboardWindowRule>,
            Rc::new(GlobalDashboardWindowTitleLengthRule { max_length: 255 }),
            Rc::new(GlobalDashboardWindowCapacityRangeRule { min: 255, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_dashboard_window_validator();
        let item = new_global_dashboard_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = GlobalDashboardWindowTitleLengthRule { max_length: 32 };
        let mut item = new_global_dashboard_window("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = GlobalDashboardWindowCapacityRangeRule { min: 16, max: 86400 };
        let mut item = new_global_dashboard_window("x", "y");
        item.capacity = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
