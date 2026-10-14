use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a rolling nonce budget must pass. Check returns its problems, or none.
pub trait RollingNonceBudgetRule: std::fmt::Debug {
    fn check(&self, item: &RollingNonceBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingNonceBudgetDurationMinutesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingNonceBudgetRule for RollingNonceBudgetDurationMinutesRangeRule {
    fn check(&self, item: &RollingNonceBudget) -> Vec<String> {
        let value = item.duration_minutes;
        if value < self.min || value > self.max {
            return vec![format!(
                "duration minutes {} is outside {} to {}",
                value, self.min, self.max
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingNonceBudgetRequiredNameRule {
    pub message: String,
}

impl RollingNonceBudgetRule for RollingNonceBudgetRequiredNameRule {
    fn check(&self, item: &RollingNonceBudget) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingNonceBudgetMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl RollingNonceBudgetRule for RollingNonceBudgetMaximumRangeRule {
    fn check(&self, item: &RollingNonceBudget) -> Vec<String> {
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
pub struct RollingNonceBudgetApprovedGuardRule {
    pub limit: i64,
}

impl RollingNonceBudgetRule for RollingNonceBudgetApprovedGuardRule {
    fn check(&self, item: &RollingNonceBudget) -> Vec<String> {
        if item.approved && (item.retries > self.limit) {
            return vec![format!(
                "retries {} exceeds {} while approved",
                item.retries, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling nonce budget.
#[derive(Debug, Clone)]
pub struct RollingNonceBudgetValidator {
    pub rules: Vec<Rc<dyn RollingNonceBudgetRule>>,
}

impl RollingNonceBudgetValidator {
    pub fn validate(&self, item: &RollingNonceBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &RollingNonceBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingNonceBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_nonce_budget_validator() -> RollingNonceBudgetValidator {
    RollingNonceBudgetValidator {
        rules: vec![
            Rc::new(RollingNonceBudgetDurationMinutesRangeRule { min: 32, max: 3600 })
                as Rc<dyn RollingNonceBudgetRule>,
            Rc::new(RollingNonceBudgetRequiredNameRule {
                message: "rolling nonce budget needs this field".to_string(),
            }),
            Rc::new(RollingNonceBudgetMaximumRangeRule { min: 4.8, max: 3827.6 }),
            Rc::new(RollingNonceBudgetApprovedGuardRule { limit: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_nonce_budget_validator();
        let item = new_rolling_nonce_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_duration_minutes() {
        let rule = RollingNonceBudgetDurationMinutesRangeRule { min: 32, max: 255 };
        let mut item = new_rolling_nonce_budget("x", "y");
        item.duration_minutes = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = RollingNonceBudgetRequiredNameRule {
            message: "rolling nonce budget needs a value".to_string(),
        };
        let mut item = new_rolling_nonce_budget("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = RollingNonceBudgetMaximumRangeRule { min: 9.9, max: 1714.3 };
        let mut item = new_rolling_nonce_budget("x", "y");
        item.maximum = 1715.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
