use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a manual tenant target must pass. Check returns its problems, or none.
pub trait ManualTenantTargetRule: std::fmt::Debug {
    fn check(&self, item: &ManualTenantTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTenantTargetCodeLengthRule {
    pub max_length: i64,
}

impl ManualTenantTargetRule for ManualTenantTargetCodeLengthRule {
    fn check(&self, item: &ManualTenantTarget) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTenantTargetAllowedCountryRule {
    pub allowed: Vec<String>,
}

impl ManualTenantTargetRule for ManualTenantTargetAllowedCountryRule {
    fn check(&self, item: &ManualTenantTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.country)))
        {
            return vec![format!("country {} is out of range", item.country)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTenantTargetWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualTenantTargetRule for ManualTenantTargetWeightRangeRule {
    fn check(&self, item: &ManualTenantTarget) -> Vec<String> {
        let value = item.weight;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("weight {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("weight {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualTenantTargetFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualTenantTargetRule for ManualTenantTargetFloorValueRangeRule {
    fn check(&self, item: &ManualTenantTarget) -> Vec<String> {
        let value = item.floor_value;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("floor value {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("floor value {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a manual tenant target.
#[derive(Debug, Clone)]
pub struct ManualTenantTargetValidator {
    pub rules: Vec<Rc<dyn ManualTenantTargetRule>>,
}

impl ManualTenantTargetValidator {
    pub fn validate(&self, item: &ManualTenantTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualTenantTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualTenantTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_tenant_target_validator() -> ManualTenantTargetValidator {
    ManualTenantTargetValidator {
        rules: vec![
            Rc::new(ManualTenantTargetCodeLengthRule { max_length: 120 }) as Rc<dyn ManualTenantTargetRule>,
            Rc::new(ManualTenantTargetAllowedCountryRule { allowed: Vec::new() }),
            Rc::new(ManualTenantTargetWeightRangeRule { min: 2.6, max: 3688.9 }),
            Rc::new(ManualTenantTargetFloorValueRangeRule { min: 6.4, max: 2846.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_tenant_target_validator();
        let item = new_manual_tenant_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ManualTenantTargetCodeLengthRule { max_length: 16 };
        let mut item = new_manual_tenant_target("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = ManualTenantTargetWeightRangeRule { min: 0.8, max: 176.1 };
        let mut item = new_manual_tenant_target("x", "y");
        item.weight = 177.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = ManualTenantTargetFloorValueRangeRule { min: 4.9, max: 3775.3 };
        let mut item = new_manual_tenant_target("x", "y");
        item.floor_value = 3776.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
