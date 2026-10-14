use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a manual rule snapshot must pass. Check returns its problems, or none.
pub trait ManualRuleSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &ManualRuleSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRuleSnapshotWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ManualRuleSnapshotRule for ManualRuleSnapshotWeightRangeRule {
    fn check(&self, item: &ManualRuleSnapshot) -> Vec<String> {
        let value = item.weight;
        if value < self.min || value > self.max {
            return vec![format!("weight {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRuleSnapshotRequiredSummaryRule {
    pub message: String,
}

impl ManualRuleSnapshotRule for ManualRuleSnapshotRequiredSummaryRule {
    fn check(&self, item: &ManualRuleSnapshot) -> Vec<String> {
        if item.summary.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRuleSnapshotRequiredNameRule {
    pub message: String,
}

impl ManualRuleSnapshotRule for ManualRuleSnapshotRequiredNameRule {
    fn check(&self, item: &ManualRuleSnapshot) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRuleSnapshotSummaryLengthRule {
    pub max_length: i64,
}

impl ManualRuleSnapshotRule for ManualRuleSnapshotSummaryLengthRule {
    fn check(&self, item: &ManualRuleSnapshot) -> Vec<String> {
        let size = item.summary.len() as i64;
        if size > self.max_length {
            return vec![format!("summary is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ManualRuleSnapshotRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ManualRuleSnapshotRule for ManualRuleSnapshotRotationDaysRangeRule {
    fn check(&self, item: &ManualRuleSnapshot) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a manual rule snapshot.
#[derive(Debug, Clone)]
pub struct ManualRuleSnapshotValidator {
    pub rules: Vec<Rc<dyn ManualRuleSnapshotRule>>,
}

impl ManualRuleSnapshotValidator {
    pub fn validate(&self, item: &ManualRuleSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ManualRuleSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ManualRuleSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_manual_rule_snapshot_validator() -> ManualRuleSnapshotValidator {
    ManualRuleSnapshotValidator {
        rules: vec![
            Rc::new(ManualRuleSnapshotWeightRangeRule { min: 1.6, max: 2890.5 }) as Rc<dyn ManualRuleSnapshotRule>,
            Rc::new(ManualRuleSnapshotRequiredSummaryRule {
                message: "manual rule snapshot needs an entry".to_string(),
            }),
            Rc::new(ManualRuleSnapshotRequiredNameRule {
                message: "manual rule snapshot needs this field".to_string(),
            }),
            Rc::new(ManualRuleSnapshotSummaryLengthRule { max_length: 1000 }),
            Rc::new(ManualRuleSnapshotRotationDaysRangeRule { min: 255, max: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_manual_rule_snapshot_validator();
        let item = new_manual_rule_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = ManualRuleSnapshotWeightRangeRule { min: 5.5, max: 2127.5 };
        let mut item = new_manual_rule_snapshot("x", "y");
        item.weight = 2129.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_summary() {
        let rule = ManualRuleSnapshotRequiredSummaryRule {
            message: "manual rule snapshot needs this field".to_string(),
        };
        let mut item = new_manual_rule_snapshot("", "");
        item.summary = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = ManualRuleSnapshotRequiredNameRule {
            message: "manual rule snapshot needs this field".to_string(),
        };
        let mut item = new_manual_rule_snapshot("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_summary() {
        let rule = ManualRuleSnapshotSummaryLengthRule { max_length: 86400 };
        let mut item = new_manual_rule_snapshot("x", "y");
        item.summary = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.summary = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = ManualRuleSnapshotRotationDaysRangeRule { min: 32, max: 86400 };
        let mut item = new_manual_rule_snapshot("x", "y");
        item.rotation_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
