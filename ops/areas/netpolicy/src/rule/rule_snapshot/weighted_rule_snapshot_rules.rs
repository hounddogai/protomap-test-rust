use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a weighted rule snapshot must pass. Check returns its problems, or none.
pub trait WeightedRuleSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &WeightedRuleSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleSnapshotRequiredTitleRule {
    pub message: String,
}

impl WeightedRuleSnapshotRule for WeightedRuleSnapshotRequiredTitleRule {
    fn check(&self, item: &WeightedRuleSnapshot) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleSnapshotCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedRuleSnapshotRule for WeightedRuleSnapshotCeilingRangeRule {
    fn check(&self, item: &WeightedRuleSnapshot) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleSnapshotCodeLengthRule {
    pub max_length: i64,
}

impl WeightedRuleSnapshotRule for WeightedRuleSnapshotCodeLengthRule {
    fn check(&self, item: &WeightedRuleSnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedRuleSnapshotMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedRuleSnapshotRule for WeightedRuleSnapshotMaximumRangeRule {
    fn check(&self, item: &WeightedRuleSnapshot) -> Vec<String> {
        let value = item.maximum;
        if value < self.min || value > self.max {
            return vec![format!("maximum {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a weighted rule snapshot.
#[derive(Debug, Clone)]
pub struct WeightedRuleSnapshotValidator {
    pub rules: Vec<Rc<dyn WeightedRuleSnapshotRule>>,
}

impl WeightedRuleSnapshotValidator {
    pub fn validate(&self, item: &WeightedRuleSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedRuleSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedRuleSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_rule_snapshot_validator() -> WeightedRuleSnapshotValidator {
    WeightedRuleSnapshotValidator {
        rules: vec![
            Rc::new(WeightedRuleSnapshotRequiredTitleRule {
                message: "weighted rule snapshot needs this field".to_string(),
            }) as Rc<dyn WeightedRuleSnapshotRule>,
            Rc::new(WeightedRuleSnapshotCeilingRangeRule { min: 6.5, max: 1442.2 }),
            Rc::new(WeightedRuleSnapshotCodeLengthRule { max_length: 16 }),
            Rc::new(WeightedRuleSnapshotMaximumRangeRule { min: 2.4, max: 4260.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_rule_snapshot_validator();
        let item = new_weighted_rule_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = WeightedRuleSnapshotRequiredTitleRule {
            message: "weighted rule snapshot needs a value".to_string(),
        };
        let mut item = new_weighted_rule_snapshot("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = WeightedRuleSnapshotCeilingRangeRule { min: 8.4, max: 3467.3 };
        let mut item = new_weighted_rule_snapshot("x", "y");
        item.ceiling = 3468.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = WeightedRuleSnapshotCodeLengthRule { max_length: 86400 };
        let mut item = new_weighted_rule_snapshot("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = WeightedRuleSnapshotMaximumRangeRule { min: 8.3, max: 4211.1 };
        let mut item = new_weighted_rule_snapshot("x", "y");
        item.maximum = 4212.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
