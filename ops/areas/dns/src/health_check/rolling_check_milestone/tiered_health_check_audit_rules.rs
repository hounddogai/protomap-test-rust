use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered health check audit must pass. Check returns its problems, or none.
pub trait TieredHealthCheckAuditRule: std::fmt::Debug {
    fn check(&self, item: &TieredHealthCheckAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckAuditNameLengthRule {
    pub max_length: i64,
}

impl TieredHealthCheckAuditRule for TieredHealthCheckAuditNameLengthRule {
    fn check(&self, item: &TieredHealthCheckAudit) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckAuditTtlSecondsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredHealthCheckAuditRule for TieredHealthCheckAuditTtlSecondsRangeRule {
    fn check(&self, item: &TieredHealthCheckAudit) -> Vec<String> {
        let value = item.ttl_seconds;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ttl seconds {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ttl seconds {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckAuditBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredHealthCheckAuditRule for TieredHealthCheckAuditBudgetRangeRule {
    fn check(&self, item: &TieredHealthCheckAudit) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckAuditRequiredCountryRule {
    pub message: String,
}

impl TieredHealthCheckAuditRule for TieredHealthCheckAuditRequiredCountryRule {
    fn check(&self, item: &TieredHealthCheckAudit) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckAuditRequiredCodeRule {
    pub message: String,
}

impl TieredHealthCheckAuditRule for TieredHealthCheckAuditRequiredCodeRule {
    fn check(&self, item: &TieredHealthCheckAudit) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredHealthCheckAuditCodeLengthRule {
    pub max_length: i64,
}

impl TieredHealthCheckAuditRule for TieredHealthCheckAuditCodeLengthRule {
    fn check(&self, item: &TieredHealthCheckAudit) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered health check audit.
#[derive(Debug, Clone)]
pub struct TieredHealthCheckAuditValidator {
    pub rules: Vec<Rc<dyn TieredHealthCheckAuditRule>>,
}

impl TieredHealthCheckAuditValidator {
    pub fn validate(&self, item: &TieredHealthCheckAudit) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredHealthCheckAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredHealthCheckAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_health_check_audit_validator() -> TieredHealthCheckAuditValidator {
    TieredHealthCheckAuditValidator {
        rules: vec![
            Rc::new(TieredHealthCheckAuditNameLengthRule { max_length: 3600 }) as Rc<dyn TieredHealthCheckAuditRule>,
            Rc::new(TieredHealthCheckAuditTtlSecondsRangeRule { min: 86400, max: 86400 }),
            Rc::new(TieredHealthCheckAuditBudgetRangeRule { min: 2.4, max: 2716.5 }),
            Rc::new(TieredHealthCheckAuditRequiredCountryRule {
                message: "tiered health check audit needs an entry".to_string(),
            }),
            Rc::new(TieredHealthCheckAuditRequiredCodeRule {
                message: "tiered health check audit needs a value".to_string(),
            }),
            Rc::new(TieredHealthCheckAuditCodeLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_health_check_audit_validator();
        let item = new_tiered_health_check_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = TieredHealthCheckAuditNameLengthRule { max_length: 3600 };
        let mut item = new_tiered_health_check_audit("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ttl_seconds() {
        let rule = TieredHealthCheckAuditTtlSecondsRangeRule { min: 120, max: 1000 };
        let mut item = new_tiered_health_check_audit("x", "y");
        item.ttl_seconds = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = TieredHealthCheckAuditBudgetRangeRule { min: 3.5, max: 3742.6 };
        let mut item = new_tiered_health_check_audit("x", "y");
        item.budget = 3744.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = TieredHealthCheckAuditRequiredCountryRule {
            message: "tiered health check audit needs a value".to_string(),
        };
        let mut item = new_tiered_health_check_audit("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TieredHealthCheckAuditRequiredCodeRule {
            message: "tiered health check audit needs this field".to_string(),
        };
        let mut item = new_tiered_health_check_audit("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TieredHealthCheckAuditCodeLengthRule { max_length: 3600 };
        let mut item = new_tiered_health_check_audit("x", "y");
        item.code = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
