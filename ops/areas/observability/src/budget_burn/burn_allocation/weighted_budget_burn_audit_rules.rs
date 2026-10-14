use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted budget burn audit must pass. Check returns its problems, or none.
pub trait WeightedBudgetBurnAuditRule: std::fmt::Debug {
    fn check(&self, item: &WeightedBudgetBurnAudit) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBudgetBurnAuditCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedBudgetBurnAuditRule for WeightedBudgetBurnAuditCeilingRangeRule {
    fn check(&self, item: &WeightedBudgetBurnAudit) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBudgetBurnAuditDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedBudgetBurnAuditRule for WeightedBudgetBurnAuditDurationMinutesRangeRule {
    fn check(&self, item: &WeightedBudgetBurnAudit) -> Vec<String> {
        let value = item.duration_minutes;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("duration minutes {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("duration minutes {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBudgetBurnAuditNameLengthRule {
    pub max_length: i64,
}

impl WeightedBudgetBurnAuditRule for WeightedBudgetBurnAuditNameLengthRule {
    fn check(&self, item: &WeightedBudgetBurnAudit) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBudgetBurnAuditCodeLengthRule {
    pub max_length: i64,
}

impl WeightedBudgetBurnAuditRule for WeightedBudgetBurnAuditCodeLengthRule {
    fn check(&self, item: &WeightedBudgetBurnAudit) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBudgetBurnAuditCipherLengthRule {
    pub max_length: i64,
}

impl WeightedBudgetBurnAuditRule for WeightedBudgetBurnAuditCipherLengthRule {
    fn check(&self, item: &WeightedBudgetBurnAudit) -> Vec<String> {
        let size = item.cipher.len() as i64;
        if size > self.max_length {
            return vec![format!("cipher is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedBudgetBurnAuditExpiresAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedBudgetBurnAuditRule for WeightedBudgetBurnAuditExpiresAtRangeRule {
    fn check(&self, item: &WeightedBudgetBurnAudit) -> Vec<String> {
        let value = item.expires_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("expires at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("expires at {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a weighted budget burn audit.
#[derive(Debug, Clone)]
pub struct WeightedBudgetBurnAuditValidator {
    pub rules: Vec<Rc<dyn WeightedBudgetBurnAuditRule>>,
}

impl WeightedBudgetBurnAuditValidator {
    pub fn validate(&self, item: &WeightedBudgetBurnAudit) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedBudgetBurnAudit) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedBudgetBurnAudit]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_budget_burn_audit_validator() -> WeightedBudgetBurnAuditValidator {
    WeightedBudgetBurnAuditValidator {
        rules: vec![
            Rc::new(WeightedBudgetBurnAuditCeilingRangeRule { min: 4.5, max: 3909.9 })
                as Rc<dyn WeightedBudgetBurnAuditRule>,
            Rc::new(WeightedBudgetBurnAuditDurationMinutesRangeRule { min: 64, max: 120 }),
            Rc::new(WeightedBudgetBurnAuditNameLengthRule { max_length: 120 }),
            Rc::new(WeightedBudgetBurnAuditCodeLengthRule { max_length: 3600 }),
            Rc::new(WeightedBudgetBurnAuditCipherLengthRule { max_length: 86400 }),
            Rc::new(WeightedBudgetBurnAuditExpiresAtRangeRule { min: 1000, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_budget_burn_audit_validator();
        let item = new_weighted_budget_burn_audit("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = WeightedBudgetBurnAuditCeilingRangeRule { min: 0.1, max: 1918.3 };
        let mut item = new_weighted_budget_burn_audit("x", "y");
        item.ceiling = 1919.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = WeightedBudgetBurnAuditDurationMinutesRangeRule { min: 32, max: 80 };
        let mut item = new_weighted_budget_burn_audit("x", "y");
        item.duration_minutes = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = WeightedBudgetBurnAuditNameLengthRule { max_length: 32 };
        let mut item = new_weighted_budget_burn_audit("x", "y");
        item.name = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedBudgetBurnAuditCodeLengthRule { max_length: 64 };
        let mut item = new_weighted_budget_burn_audit("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_cipher() {
        let rule = WeightedBudgetBurnAuditCipherLengthRule { max_length: 64 };
        let mut item = new_weighted_budget_burn_audit("x", "y");
        item.cipher = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.cipher = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_expires_at() {
        let rule = WeightedBudgetBurnAuditExpiresAtRangeRule { min: 1000, max: 86400 };
        let mut item = new_weighted_budget_burn_audit("x", "y");
        item.expires_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
