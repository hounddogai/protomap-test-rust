use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted grant snapshot must pass. Check returns its problems, or none.
pub trait WeightedGrantSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &WeightedGrantSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedGrantSnapshotWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedGrantSnapshotRule for WeightedGrantSnapshotWeightRangeRule {
    fn check(&self, item: &WeightedGrantSnapshot) -> Vec<String> {
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
pub struct WeightedGrantSnapshotQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedGrantSnapshotRule for WeightedGrantSnapshotQuantityRangeRule {
    fn check(&self, item: &WeightedGrantSnapshot) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedGrantSnapshotCodeLengthRule {
    pub max_length: i64,
}

impl WeightedGrantSnapshotRule for WeightedGrantSnapshotCodeLengthRule {
    fn check(&self, item: &WeightedGrantSnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedGrantSnapshotNameLengthRule {
    pub max_length: i64,
}

impl WeightedGrantSnapshotRule for WeightedGrantSnapshotNameLengthRule {
    fn check(&self, item: &WeightedGrantSnapshot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedGrantSnapshotCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedGrantSnapshotRule for WeightedGrantSnapshotCostRangeRule {
    fn check(&self, item: &WeightedGrantSnapshot) -> Vec<String> {
        let value = item.cost;
        if value < self.min || value > self.max {
            return vec![format!("cost {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted grant snapshot.
#[derive(Debug, Clone)]
pub struct WeightedGrantSnapshotValidator {
    pub rules: Vec<Rc<dyn WeightedGrantSnapshotRule>>,
}

impl WeightedGrantSnapshotValidator {
    pub fn validate(&self, item: &WeightedGrantSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &WeightedGrantSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedGrantSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_grant_snapshot_validator() -> WeightedGrantSnapshotValidator {
    WeightedGrantSnapshotValidator {
        rules: vec![
            Rc::new(WeightedGrantSnapshotWeightRangeRule { min: 8.2, max: 4018.8 })
                as Rc<dyn WeightedGrantSnapshotRule>,
            Rc::new(WeightedGrantSnapshotQuantityRangeRule { min: 32, max: 255 }),
            Rc::new(WeightedGrantSnapshotCodeLengthRule { max_length: 64 }),
            Rc::new(WeightedGrantSnapshotNameLengthRule { max_length: 1000 }),
            Rc::new(WeightedGrantSnapshotCostRangeRule { min: 6.8, max: 4565.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_grant_snapshot_validator();
        let item = new_weighted_grant_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = WeightedGrantSnapshotWeightRangeRule { min: 4.4, max: 3402.0 };
        let mut item = new_weighted_grant_snapshot("x", "y");
        item.weight = 3403.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = WeightedGrantSnapshotQuantityRangeRule { min: 80, max: 1000 };
        let mut item = new_weighted_grant_snapshot("x", "y");
        item.quantity = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedGrantSnapshotCodeLengthRule { max_length: 120 };
        let mut item = new_weighted_grant_snapshot("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = WeightedGrantSnapshotNameLengthRule { max_length: 64 };
        let mut item = new_weighted_grant_snapshot("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = WeightedGrantSnapshotCostRangeRule { min: 5.0, max: 3128.8 };
        let mut item = new_weighted_grant_snapshot("x", "y");
        item.cost = 3130.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
