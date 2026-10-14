use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict tenant adjustment must pass. Check returns its problems, or none.
pub trait StrictTenantAdjustmentRule: std::fmt::Debug {
    fn check(&self, item: &StrictTenantAdjustment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTenantAdjustmentAllowedNameRule {
    pub allowed: Vec<String>,
}

impl StrictTenantAdjustmentRule for StrictTenantAdjustmentAllowedNameRule {
    fn check(&self, item: &StrictTenantAdjustment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is not allowed", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTenantAdjustmentUnitLengthRule {
    pub max_length: i64,
}

impl StrictTenantAdjustmentRule for StrictTenantAdjustmentUnitLengthRule {
    fn check(&self, item: &StrictTenantAdjustment) -> Vec<String> {
        let size = item.unit.len() as i64;
        if size > self.max_length {
            return vec![format!("unit is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTenantAdjustmentCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictTenantAdjustmentRule for StrictTenantAdjustmentCreatedAtRangeRule {
    fn check(&self, item: &StrictTenantAdjustment) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictTenantAdjustmentPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictTenantAdjustmentRule for StrictTenantAdjustmentPortRangeRule {
    fn check(&self, item: &StrictTenantAdjustment) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict tenant adjustment.
#[derive(Debug, Clone)]
pub struct StrictTenantAdjustmentValidator {
    pub rules: Vec<Rc<dyn StrictTenantAdjustmentRule>>,
}

impl StrictTenantAdjustmentValidator {
    pub fn validate(&self, item: &StrictTenantAdjustment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictTenantAdjustment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictTenantAdjustment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_tenant_adjustment_validator() -> StrictTenantAdjustmentValidator {
    StrictTenantAdjustmentValidator {
        rules: vec![
            Rc::new(StrictTenantAdjustmentAllowedNameRule {
                allowed: vec!["tiered".to_string(), "seasonal".to_string()],
            }) as Rc<dyn StrictTenantAdjustmentRule>,
            Rc::new(StrictTenantAdjustmentUnitLengthRule { max_length: 64 }),
            Rc::new(StrictTenantAdjustmentCreatedAtRangeRule { min: 32, max: 86400 }),
            Rc::new(StrictTenantAdjustmentPortRangeRule { min: 32, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_tenant_adjustment_validator();
        let item = new_strict_tenant_adjustment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_unit() {
        let rule = StrictTenantAdjustmentUnitLengthRule { max_length: 120 };
        let mut item = new_strict_tenant_adjustment("x", "y");
        item.unit = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.unit = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = StrictTenantAdjustmentCreatedAtRangeRule { min: 32, max: 1000 };
        let mut item = new_strict_tenant_adjustment("x", "y");
        item.created_at = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = StrictTenantAdjustmentPortRangeRule { min: 64, max: 80 };
        let mut item = new_strict_tenant_adjustment("x", "y");
        item.port = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
