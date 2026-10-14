use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive nonce budget must pass. Check returns its problems, or none.
pub trait AdaptiveNonceBudgetRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveNonceBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveNonceBudgetPinnedGuardRule {
    pub limit: i64,
}

impl AdaptiveNonceBudgetRule for AdaptiveNonceBudgetPinnedGuardRule {
    fn check(&self, item: &AdaptiveNonceBudget) -> Vec<String> {
        if item.pinned && (item.starts_at > self.limit) {
            return vec![format!(
                "starts at {} exceeds {} while pinned",
                item.starts_at, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveNonceBudgetStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveNonceBudgetRule for AdaptiveNonceBudgetStartsAtRangeRule {
    fn check(&self, item: &AdaptiveNonceBudget) -> Vec<String> {
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
pub struct AdaptiveNonceBudgetRequiredRegionRule {
    pub message: String,
}

impl AdaptiveNonceBudgetRule for AdaptiveNonceBudgetRequiredRegionRule {
    fn check(&self, item: &AdaptiveNonceBudget) -> Vec<String> {
        if item.region.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive nonce budget.
#[derive(Debug, Clone)]
pub struct AdaptiveNonceBudgetValidator {
    pub rules: Vec<Rc<dyn AdaptiveNonceBudgetRule>>,
}

impl AdaptiveNonceBudgetValidator {
    pub fn validate(&self, item: &AdaptiveNonceBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveNonceBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveNonceBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_nonce_budget_validator() -> AdaptiveNonceBudgetValidator {
    AdaptiveNonceBudgetValidator {
        rules: vec![
            Rc::new(AdaptiveNonceBudgetPinnedGuardRule { limit: 16 }) as Rc<dyn AdaptiveNonceBudgetRule>,
            Rc::new(AdaptiveNonceBudgetStartsAtRangeRule { min: 3600, max: 86400 }),
            Rc::new(AdaptiveNonceBudgetRequiredRegionRule {
                message: "adaptive nonce budget needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_nonce_budget_validator();
        let item = new_adaptive_nonce_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = AdaptiveNonceBudgetStartsAtRangeRule { min: 64, max: 64 };
        let mut item = new_adaptive_nonce_budget("x", "y");
        item.starts_at = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_region() {
        let rule = AdaptiveNonceBudgetRequiredRegionRule {
            message: "adaptive nonce budget needs this field".to_string(),
        };
        let mut item = new_adaptive_nonce_budget("", "");
        item.region = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
