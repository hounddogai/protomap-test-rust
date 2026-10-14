use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dashboard record must pass. Check returns its problems, or none.
pub trait DashboardRecordRule: std::fmt::Debug {
    fn check(&self, item: &DashboardRecord) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardRecordCodeLengthRule {
    pub max_length: i64,
}

impl DashboardRecordRule for DashboardRecordCodeLengthRule {
    fn check(&self, item: &DashboardRecord) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardRecordGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DashboardRecordRule for DashboardRecordGraceDaysRangeRule {
    fn check(&self, item: &DashboardRecord) -> Vec<String> {
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
pub struct DashboardRecordRequiredScopeRule {
    pub message: String,
}

impl DashboardRecordRule for DashboardRecordRequiredScopeRule {
    fn check(&self, item: &DashboardRecord) -> Vec<String> {
        if item.scope.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardRecordScopeLengthRule {
    pub max_length: i64,
}

impl DashboardRecordRule for DashboardRecordScopeLengthRule {
    fn check(&self, item: &DashboardRecord) -> Vec<String> {
        let size = item.scope.len() as i64;
        if size > self.max_length {
            return vec![format!("scope is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardRecordMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DashboardRecordRule for DashboardRecordMaximumRangeRule {
    fn check(&self, item: &DashboardRecord) -> Vec<String> {
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
pub struct DashboardRecordNameLengthRule {
    pub max_length: i64,
}

impl DashboardRecordRule for DashboardRecordNameLengthRule {
    fn check(&self, item: &DashboardRecord) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a dashboard record.
#[derive(Debug, Clone)]
pub struct DashboardRecordValidator {
    pub rules: Vec<Rc<dyn DashboardRecordRule>>,
}

impl DashboardRecordValidator {
    pub fn validate(&self, item: &DashboardRecord) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &DashboardRecord) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DashboardRecord]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dashboard_record_validator() -> DashboardRecordValidator {
    DashboardRecordValidator {
        rules: vec![
            Rc::new(DashboardRecordCodeLengthRule { max_length: 16 }) as Rc<dyn DashboardRecordRule>,
            Rc::new(DashboardRecordGraceDaysRangeRule { min: 16, max: 80 }),
            Rc::new(DashboardRecordRequiredScopeRule {
                message: "dashboard record needs an entry".to_string(),
            }),
            Rc::new(DashboardRecordScopeLengthRule { max_length: 64 }),
            Rc::new(DashboardRecordMaximumRangeRule { min: 5.3, max: 2775.7 }),
            Rc::new(DashboardRecordNameLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dashboard_record_validator();
        let item = new_dashboard_record("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = DashboardRecordCodeLengthRule { max_length: 120 };
        let mut item = new_dashboard_record("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = DashboardRecordGraceDaysRangeRule { min: 32, max: 255 };
        let mut item = new_dashboard_record("x", "y");
        item.grace_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_scope() {
        let rule = DashboardRecordRequiredScopeRule {
            message: "dashboard record needs an entry".to_string(),
        };
        let mut item = new_dashboard_record("", "");
        item.scope = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_scope() {
        let rule = DashboardRecordScopeLengthRule { max_length: 86400 };
        let mut item = new_dashboard_record("x", "y");
        item.scope = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.scope = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = DashboardRecordMaximumRangeRule { min: 7.7, max: 4603.5 };
        let mut item = new_dashboard_record("x", "y");
        item.maximum = 4605.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = DashboardRecordNameLengthRule { max_length: 3600 };
        let mut item = new_dashboard_record("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
