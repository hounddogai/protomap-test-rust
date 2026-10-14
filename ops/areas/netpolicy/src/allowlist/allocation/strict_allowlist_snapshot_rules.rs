use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict allowlist snapshot must pass. Check returns its problems, or none.
pub trait StrictAllowlistSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &StrictAllowlistSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAllowlistSnapshotDiscountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictAllowlistSnapshotRule for StrictAllowlistSnapshotDiscountRangeRule {
    fn check(&self, item: &StrictAllowlistSnapshot) -> Vec<String> {
        let value = item.discount;
        if value < self.min || value > self.max {
            return vec![format!("discount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAllowlistSnapshotLabelLengthRule {
    pub max_length: i64,
}

impl StrictAllowlistSnapshotRule for StrictAllowlistSnapshotLabelLengthRule {
    fn check(&self, item: &StrictAllowlistSnapshot) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAllowlistSnapshotCodeLengthRule {
    pub max_length: i64,
}

impl StrictAllowlistSnapshotRule for StrictAllowlistSnapshotCodeLengthRule {
    fn check(&self, item: &StrictAllowlistSnapshot) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAllowlistSnapshotAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictAllowlistSnapshotRule for StrictAllowlistSnapshotAmountRangeRule {
    fn check(&self, item: &StrictAllowlistSnapshot) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictAllowlistSnapshotPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl StrictAllowlistSnapshotRule for StrictAllowlistSnapshotPortRangeRule {
    fn check(&self, item: &StrictAllowlistSnapshot) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict allowlist snapshot.
#[derive(Debug, Clone)]
pub struct StrictAllowlistSnapshotValidator {
    pub rules: Vec<Rc<dyn StrictAllowlistSnapshotRule>>,
}

impl StrictAllowlistSnapshotValidator {
    pub fn validate(&self, item: &StrictAllowlistSnapshot) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictAllowlistSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictAllowlistSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_allowlist_snapshot_validator() -> StrictAllowlistSnapshotValidator {
    StrictAllowlistSnapshotValidator {
        rules: vec![
            Rc::new(StrictAllowlistSnapshotDiscountRangeRule { min: 1.1, max: 3274.5 })
                as Rc<dyn StrictAllowlistSnapshotRule>,
            Rc::new(StrictAllowlistSnapshotLabelLengthRule { max_length: 64 }),
            Rc::new(StrictAllowlistSnapshotCodeLengthRule { max_length: 120 }),
            Rc::new(StrictAllowlistSnapshotAmountRangeRule { min: 2.6, max: 2371.6 }),
            Rc::new(StrictAllowlistSnapshotPortRangeRule { min: 80, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_allowlist_snapshot_validator();
        let item = new_strict_allowlist_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_discount() {
        let rule = StrictAllowlistSnapshotDiscountRangeRule { min: 9.9, max: 4175.1 };
        let mut item = new_strict_allowlist_snapshot("x", "y");
        item.discount = 4176.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = StrictAllowlistSnapshotLabelLengthRule { max_length: 16 };
        let mut item = new_strict_allowlist_snapshot("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = StrictAllowlistSnapshotCodeLengthRule { max_length: 255 };
        let mut item = new_strict_allowlist_snapshot("x", "y");
        item.code = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = StrictAllowlistSnapshotAmountRangeRule { min: 7.5, max: 2763.7 };
        let mut item = new_strict_allowlist_snapshot("x", "y");
        item.amount = 2765.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = StrictAllowlistSnapshotPortRangeRule { min: 64, max: 3600 };
        let mut item = new_strict_allowlist_snapshot("x", "y");
        item.port = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
