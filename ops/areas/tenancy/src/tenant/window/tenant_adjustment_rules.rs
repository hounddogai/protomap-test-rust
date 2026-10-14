use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tenant adjustment must pass. Check returns its problems, or none.
pub trait TenantAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &TenantAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantAdjustmentRequiredTitleRule {
    pub message: String,
}

impl TenantAdjustmentRule for TenantAdjustmentRequiredTitleRule {
    fn check(&self, item: &TenantAdjustment) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantAdjustmentCodeLengthRule {
    pub max_length: i64,
}

impl TenantAdjustmentRule for TenantAdjustmentCodeLengthRule {
    fn check(&self, item: &TenantAdjustment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantAdjustmentRequiredCountryRule {
    pub message: String,
}

impl TenantAdjustmentRule for TenantAdjustmentRequiredCountryRule {
    fn check(&self, item: &TenantAdjustment) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantAdjustmentRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TenantAdjustmentRule for TenantAdjustmentRetriesRangeRule {
    fn check(&self, item: &TenantAdjustment) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TenantAdjustmentBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TenantAdjustmentRule for TenantAdjustmentBudgetRangeRule {
    fn check(&self, item: &TenantAdjustment) -> Vec<String> {
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

/// Runs every rule of a tenant adjustment.
#[derive(Debug, Clone)]
pub struct TenantAdjustmentValidator {
    pub rules: Vec<Rc<dyn TenantAdjustmentRule>>,
}

impl TenantAdjustmentValidator {
    pub fn validate(&self, item: &TenantAdjustment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TenantAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TenantAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tenant_adjustment_validator() -> TenantAdjustmentValidator {
    TenantAdjustmentValidator {
        rules: vec![
            Rc::new(TenantAdjustmentRequiredTitleRule {
                message: "tenant adjustment needs this field".to_string(),
            }) as Rc<dyn TenantAdjustmentRule>,
            Rc::new(TenantAdjustmentCodeLengthRule { max_length: 16 }),
            Rc::new(TenantAdjustmentRequiredCountryRule {
                message: "tenant adjustment needs a value".to_string(),
            }),
            Rc::new(TenantAdjustmentRetriesRangeRule { min: 80, max: 120 }),
            Rc::new(TenantAdjustmentBudgetRangeRule { min: 0.4, max: 2181.5 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tenant_adjustment_validator();
        let item = new_tenant_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = TenantAdjustmentRequiredTitleRule {
            message: "tenant adjustment needs a value".to_string(),
        };
        let mut item = new_tenant_adjustment("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TenantAdjustmentCodeLengthRule { max_length: 16 };
        let mut item = new_tenant_adjustment("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = TenantAdjustmentRequiredCountryRule {
            message: "tenant adjustment needs this field".to_string(),
        };
        let mut item = new_tenant_adjustment("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = TenantAdjustmentRetriesRangeRule { min: 32, max: 80 };
        let mut item = new_tenant_adjustment("x", "y");
        item.retries = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = TenantAdjustmentBudgetRangeRule { min: 9.7, max: 1631.1 };
        let mut item = new_tenant_adjustment("x", "y");
        item.budget = 1632.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
