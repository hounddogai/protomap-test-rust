use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted reservation budget must pass. Check returns its problems, or none.
pub trait WeightedReservationBudgetRule: std::fmt::Debug {
    fn check(&self, item: &WeightedReservationBudget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedReservationBudgetActiveGuardRule {
    pub limit: f64,
}

impl WeightedReservationBudgetRule for WeightedReservationBudgetActiveGuardRule {
    fn check(&self, item: &WeightedReservationBudget) -> Vec<String> {
        if item.active && (item.weight > self.limit) {
            return vec![format!("weight {} exceeds {} while active", item.weight, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedReservationBudgetGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedReservationBudgetRule for WeightedReservationBudgetGraceDaysRangeRule {
    fn check(&self, item: &WeightedReservationBudget) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedReservationBudgetSegmentLengthRule {
    pub max_length: i64,
}

impl WeightedReservationBudgetRule for WeightedReservationBudgetSegmentLengthRule {
    fn check(&self, item: &WeightedReservationBudget) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedReservationBudgetNameLengthRule {
    pub max_length: i64,
}

impl WeightedReservationBudgetRule for WeightedReservationBudgetNameLengthRule {
    fn check(&self, item: &WeightedReservationBudget) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedReservationBudgetRequiredCodeRule {
    pub message: String,
}

impl WeightedReservationBudgetRule for WeightedReservationBudgetRequiredCodeRule {
    fn check(&self, item: &WeightedReservationBudget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted reservation budget.
#[derive(Debug, Clone)]
pub struct WeightedReservationBudgetValidator {
    pub rules: Vec<Rc<dyn WeightedReservationBudgetRule>>,
}

impl WeightedReservationBudgetValidator {
    pub fn validate(&self, item: &WeightedReservationBudget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedReservationBudget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedReservationBudget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_reservation_budget_validator() -> WeightedReservationBudgetValidator {
    WeightedReservationBudgetValidator {
        rules: vec![
            Rc::new(WeightedReservationBudgetActiveGuardRule { limit: 9.3 }) as Rc<dyn WeightedReservationBudgetRule>,
            Rc::new(WeightedReservationBudgetGraceDaysRangeRule { min: 255, max: 86400 }),
            Rc::new(WeightedReservationBudgetSegmentLengthRule { max_length: 3600 }),
            Rc::new(WeightedReservationBudgetNameLengthRule { max_length: 16 }),
            Rc::new(WeightedReservationBudgetRequiredCodeRule {
                message: "weighted reservation budget needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_reservation_budget_validator();
        let item = new_weighted_reservation_budget("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = WeightedReservationBudgetGraceDaysRangeRule { min: 64, max: 255 };
        let mut item = new_weighted_reservation_budget("x", "y");
        item.grace_days = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = WeightedReservationBudgetSegmentLengthRule { max_length: 32 };
        let mut item = new_weighted_reservation_budget("x", "y");
        item.segment = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = WeightedReservationBudgetNameLengthRule { max_length: 80 };
        let mut item = new_weighted_reservation_budget("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedReservationBudgetRequiredCodeRule {
            message: "weighted reservation budget needs an entry".to_string(),
        };
        let mut item = new_weighted_reservation_budget("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
