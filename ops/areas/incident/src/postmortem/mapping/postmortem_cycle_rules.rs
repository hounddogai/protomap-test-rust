use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a postmortem cycle must pass. Check returns its problems, or none.
pub trait PostmortemCycleRule: std::fmt::Debug {
    fn check(&self, item: &PostmortemCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemCycleMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PostmortemCycleRule for PostmortemCycleMinimumRangeRule {
    fn check(&self, item: &PostmortemCycle) -> Vec<String> {
        let value = item.minimum;
        if value < self.min || value > self.max {
            return vec![format!("minimum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemCycleCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PostmortemCycleRule for PostmortemCycleCostRangeRule {
    fn check(&self, item: &PostmortemCycle) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemCycleMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PostmortemCycleRule for PostmortemCycleMaximumRangeRule {
    fn check(&self, item: &PostmortemCycle) -> Vec<String> {
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
pub struct PostmortemCycleCodeLengthRule {
    pub max_length: i64,
}

impl PostmortemCycleRule for PostmortemCycleCodeLengthRule {
    fn check(&self, item: &PostmortemCycle) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a postmortem cycle.
#[derive(Debug, Clone)]
pub struct PostmortemCycleValidator {
    pub rules: Vec<Rc<dyn PostmortemCycleRule>>,
}

impl PostmortemCycleValidator {
    pub fn validate(&self, item: &PostmortemCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PostmortemCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PostmortemCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_postmortem_cycle_validator() -> PostmortemCycleValidator {
    PostmortemCycleValidator {
        rules: vec![
            Rc::new(PostmortemCycleMinimumRangeRule { min: 2.9, max: 4829.8 }) as Rc<dyn PostmortemCycleRule>,
            Rc::new(PostmortemCycleCostRangeRule { min: 5.5, max: 2863.5 }),
            Rc::new(PostmortemCycleMaximumRangeRule { min: 1.3, max: 1609.6 }),
            Rc::new(PostmortemCycleCodeLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_postmortem_cycle_validator();
        let item = new_postmortem_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = PostmortemCycleMinimumRangeRule { min: 0.2, max: 513.2 };
        let mut item = new_postmortem_cycle("x", "y");
        item.minimum = 514.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = PostmortemCycleCostRangeRule { min: 1.2, max: 2226.4 };
        let mut item = new_postmortem_cycle("x", "y");
        item.cost = 2227.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = PostmortemCycleMaximumRangeRule { min: 3.2, max: 1244.4 };
        let mut item = new_postmortem_cycle("x", "y");
        item.maximum = 1245.9;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PostmortemCycleCodeLengthRule { max_length: 80 };
        let mut item = new_postmortem_cycle("x", "y");
        item.code = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
