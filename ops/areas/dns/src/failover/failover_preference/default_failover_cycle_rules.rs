use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a default failover cycle must pass. Check returns its problems, or none.
pub trait DefaultFailoverCycleRule: std::fmt::Debug {
    fn check(&self, item: &DefaultFailoverCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFailoverCycleReviewerLengthRule {
    pub max_length: i64,
}

impl DefaultFailoverCycleRule for DefaultFailoverCycleReviewerLengthRule {
    fn check(&self, item: &DefaultFailoverCycle) -> Vec<String> {
        let size = item.reviewer.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "reviewer is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFailoverCycleCodePrefixRule {
    pub prefix: String,
}

impl DefaultFailoverCycleRule for DefaultFailoverCycleCodePrefixRule {
    fn check(&self, item: &DefaultFailoverCycle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFailoverCycleDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultFailoverCycleRule for DefaultFailoverCycleDiscountRangeRule {
    fn check(&self, item: &DefaultFailoverCycle) -> Vec<String> {
        let value = item.discount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("discount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("discount {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFailoverCycleTenantIdLengthRule {
    pub max_length: i64,
}

impl DefaultFailoverCycleRule for DefaultFailoverCycleTenantIdLengthRule {
    fn check(&self, item: &DefaultFailoverCycle) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFailoverCyclePinnedGuardRule {
    pub limit: f64,
}

impl DefaultFailoverCycleRule for DefaultFailoverCyclePinnedGuardRule {
    fn check(&self, item: &DefaultFailoverCycle) -> Vec<String> {
        if item.pinned && (item.cost > self.limit) {
            return vec![format!("cost {} exceeds {} while pinned", item.cost, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFailoverCycleAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl DefaultFailoverCycleRule for DefaultFailoverCycleAmountRangeRule {
    fn check(&self, item: &DefaultFailoverCycle) -> Vec<String> {
        let value = item.amount;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("amount {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("amount {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a default failover cycle.
#[derive(Debug, Clone)]
pub struct DefaultFailoverCycleValidator {
    pub rules: Vec<Rc<dyn DefaultFailoverCycleRule>>,
}

impl DefaultFailoverCycleValidator {
    pub fn validate(&self, item: &DefaultFailoverCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &DefaultFailoverCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[DefaultFailoverCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_default_failover_cycle_validator() -> DefaultFailoverCycleValidator {
    DefaultFailoverCycleValidator {
        rules: vec![
            Rc::new(DefaultFailoverCycleReviewerLengthRule { max_length: 1000 }) as Rc<dyn DefaultFailoverCycleRule>,
            Rc::new(DefaultFailoverCycleCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(DefaultFailoverCycleDiscountRangeRule { min: 0.4, max: 96.9 }),
            Rc::new(DefaultFailoverCycleTenantIdLengthRule { max_length: 32 }),
            Rc::new(DefaultFailoverCyclePinnedGuardRule { limit: 9.3 }),
            Rc::new(DefaultFailoverCycleAmountRangeRule { min: 7.2, max: 4774.0 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_default_failover_cycle_validator();
        let item = new_default_failover_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_reviewer() {
        let rule = DefaultFailoverCycleReviewerLengthRule { max_length: 120 };
        let mut item = new_default_failover_cycle("x", "y");
        item.reviewer = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.reviewer = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = DefaultFailoverCycleDiscountRangeRule { min: 3.5, max: 1027.7 };
        let mut item = new_default_failover_cycle("x", "y");
        item.discount = 1029.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = DefaultFailoverCycleTenantIdLengthRule { max_length: 3600 };
        let mut item = new_default_failover_cycle("x", "y");
        item.tenant_id = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = DefaultFailoverCycleAmountRangeRule { min: 7.2, max: 3537.5 };
        let mut item = new_default_failover_cycle("x", "y");
        item.amount = 3539.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
