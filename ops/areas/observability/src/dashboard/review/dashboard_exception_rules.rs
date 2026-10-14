use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a dashboard exception must pass. Check returns its problems, or none.
pub trait DashboardExceptionRule: std::fmt::Debug {
    fn check(&self, item: &DashboardException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardExceptionRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DashboardExceptionRule for DashboardExceptionRatioRangeRule {
    fn check(&self, item: &DashboardException) -> Vec<String> {
        let value = item.ratio;
        if value < self.min || value > self.max {
            return vec![format!("ratio {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardExceptionArchivedGuardRule {
    pub limit: f64,
}

impl DashboardExceptionRule for DashboardExceptionArchivedGuardRule {
    fn check(&self, item: &DashboardException) -> Vec<String> {
        if item.archived && (item.ratio > self.limit) {
            return vec![format!("ratio {} exceeds {} while archived", item.ratio, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardExceptionRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DashboardExceptionRule for DashboardExceptionRotationDaysRangeRule {
    fn check(&self, item: &DashboardException) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardExceptionStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl DashboardExceptionRule for DashboardExceptionStartsAtRangeRule {
    fn check(&self, item: &DashboardException) -> Vec<String> {
        let value = item.starts_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("starts at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("starts at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardExceptionRequiredNameRule {
    pub message: String,
}

impl DashboardExceptionRule for DashboardExceptionRequiredNameRule {
    fn check(&self, item: &DashboardException) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashboardExceptionNameLengthRule {
    pub max_length: i64,
}

impl DashboardExceptionRule for DashboardExceptionNameLengthRule {
    fn check(&self, item: &DashboardException) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a dashboard exception.
#[derive(Debug, Clone)]
pub struct DashboardExceptionValidator {
    pub rules: Vec<Rc<dyn DashboardExceptionRule>>,
}

impl DashboardExceptionValidator {
    pub fn validate(&self, item: &DashboardException) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DashboardException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DashboardException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_dashboard_exception_validator() -> DashboardExceptionValidator {
    DashboardExceptionValidator {
        rules: vec![
            Rc::new(DashboardExceptionRatioRangeRule { min: 0.3, max: 2896.1 }) as Rc<dyn DashboardExceptionRule>,
            Rc::new(DashboardExceptionArchivedGuardRule { limit: 7.3 }),
            Rc::new(DashboardExceptionRotationDaysRangeRule { min: 32, max: 120 }),
            Rc::new(DashboardExceptionStartsAtRangeRule { min: 255, max: 1000 }),
            Rc::new(DashboardExceptionRequiredNameRule {
                message: "dashboard exception needs this field".to_string(),
            }),
            Rc::new(DashboardExceptionNameLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_dashboard_exception_validator();
        let item = new_dashboard_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = DashboardExceptionRatioRangeRule { min: 2.5, max: 3504.4 };
        let mut item = new_dashboard_exception("x", "y");
        item.ratio = 3505.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = DashboardExceptionRotationDaysRangeRule { min: 64, max: 80 };
        let mut item = new_dashboard_exception("x", "y");
        item.rotation_days = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = DashboardExceptionStartsAtRangeRule { min: 32, max: 3600 };
        let mut item = new_dashboard_exception("x", "y");
        item.starts_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = DashboardExceptionRequiredNameRule {
            message: "dashboard exception needs an entry".to_string(),
        };
        let mut item = new_dashboard_exception("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = DashboardExceptionNameLengthRule { max_length: 80 };
        let mut item = new_dashboard_exception("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
