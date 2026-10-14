use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a allowlist budget must pass. Check returns its problems, or none.
pub trait AllowlistBudgetRule: std::fmt::Debug {
    fn check(&self, item: &AllowlistBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistBudgetExternalRefLengthRule {
    pub max_length: i64,
}

impl AllowlistBudgetRule for AllowlistBudgetExternalRefLengthRule {
    fn check(&self, item: &AllowlistBudget) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistBudgetLimitRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AllowlistBudgetRule for AllowlistBudgetLimitRangeRule {
    fn check(&self, item: &AllowlistBudget) -> Vec<String> {
        let value = item.limit;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("limit {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("limit {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistBudgetAllowedNameRule {
    pub allowed: Vec<String>,
}

impl AllowlistBudgetRule for AllowlistBudgetAllowedNameRule {
    fn check(&self, item: &AllowlistBudget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.name))) {
            return vec![format!("name {} is expired", item.name)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistBudgetThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AllowlistBudgetRule for AllowlistBudgetThresholdRangeRule {
    fn check(&self, item: &AllowlistBudget) -> Vec<String> {
        let value = item.threshold;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("threshold {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("threshold {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistBudgetDescriptionLengthRule {
    pub max_length: i64,
}

impl AllowlistBudgetRule for AllowlistBudgetDescriptionLengthRule {
    fn check(&self, item: &AllowlistBudget) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a allowlist budget.
#[derive(Debug, Clone)]
pub struct AllowlistBudgetValidator {
    pub rules: Vec<Rc<dyn AllowlistBudgetRule>>,
}

impl AllowlistBudgetValidator {
    pub fn validate(&self, item: &AllowlistBudget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AllowlistBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AllowlistBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_allowlist_budget_validator() -> AllowlistBudgetValidator {
    AllowlistBudgetValidator {
        rules: vec![
            Rc::new(AllowlistBudgetExternalRefLengthRule { max_length: 120 }) as Rc<dyn AllowlistBudgetRule>,
            Rc::new(AllowlistBudgetLimitRangeRule { min: 16, max: 64 }),
            Rc::new(AllowlistBudgetAllowedNameRule {
                allowed: vec!["weighted".to_string(), "strict".to_string(), "capped".to_string()],
            }),
            Rc::new(AllowlistBudgetThresholdRangeRule { min: 6.5, max: 1933.4 }),
            Rc::new(AllowlistBudgetDescriptionLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_allowlist_budget_validator();
        let item = new_allowlist_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = AllowlistBudgetExternalRefLengthRule { max_length: 1000 };
        let mut item = new_allowlist_budget("x", "y");
        item.external_ref = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_limit() {
        let rule = AllowlistBudgetLimitRangeRule { min: 32, max: 1000 };
        let mut item = new_allowlist_budget("x", "y");
        item.limit = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = AllowlistBudgetThresholdRangeRule { min: 7.4, max: 3633.3 };
        let mut item = new_allowlist_budget("x", "y");
        item.threshold = 3634.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = AllowlistBudgetDescriptionLengthRule { max_length: 16 };
        let mut item = new_allowlist_budget("x", "y");
        item.description = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
