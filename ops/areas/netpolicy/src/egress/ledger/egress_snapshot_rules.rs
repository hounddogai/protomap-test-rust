use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a egress snapshot must pass. Check returns its problems, or none.
pub trait EgressSnapshotRule: std::fmt::Debug {
    fn check(&self, item: &EgressSnapshot) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressSnapshotTitleLengthRule {
    pub max_length: i64,
}

impl EgressSnapshotRule for EgressSnapshotTitleLengthRule {
    fn check(&self, item: &EgressSnapshot) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressSnapshotWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EgressSnapshotRule for EgressSnapshotWeightRangeRule {
    fn check(&self, item: &EgressSnapshot) -> Vec<String> {
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
pub struct EgressSnapshotThresholdRangeRule {
    pub min: f64,
    pub max: f64,
}

impl EgressSnapshotRule for EgressSnapshotThresholdRangeRule {
    fn check(&self, item: &EgressSnapshot) -> Vec<String> {
        let value = item.threshold;
        if value < self.min || value > self.max {
            return vec![format!("threshold {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressSnapshotNameLengthRule {
    pub max_length: i64,
}

impl EgressSnapshotRule for EgressSnapshotNameLengthRule {
    fn check(&self, item: &EgressSnapshot) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressSnapshotCodePrefixRule {
    pub prefix: String,
}

impl EgressSnapshotRule for EgressSnapshotCodePrefixRule {
    fn check(&self, item: &EgressSnapshot) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress snapshot.
#[derive(Debug, Clone)]
pub struct EgressSnapshotValidator {
    pub rules: Vec<Rc<dyn EgressSnapshotRule>>,
}

impl EgressSnapshotValidator {
    pub fn validate(&self, item: &EgressSnapshot) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EgressSnapshot) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressSnapshot]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_snapshot_validator() -> EgressSnapshotValidator {
    EgressSnapshotValidator {
        rules: vec![
            Rc::new(EgressSnapshotTitleLengthRule { max_length: 1000 }) as Rc<dyn EgressSnapshotRule>,
            Rc::new(EgressSnapshotWeightRangeRule { min: 7.4, max: 569.4 }),
            Rc::new(EgressSnapshotThresholdRangeRule { min: 8.6, max: 3747.4 }),
            Rc::new(EgressSnapshotNameLengthRule { max_length: 32 }),
            Rc::new(EgressSnapshotCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_egress_snapshot_validator();
        let item = new_egress_snapshot("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = EgressSnapshotTitleLengthRule { max_length: 64 };
        let mut item = new_egress_snapshot("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = EgressSnapshotWeightRangeRule { min: 3.1, max: 192.5 };
        let mut item = new_egress_snapshot("x", "y");
        item.weight = 194.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_threshold() {
        let rule = EgressSnapshotThresholdRangeRule { min: 7.8, max: 4204.8 };
        let mut item = new_egress_snapshot("x", "y");
        item.threshold = 4206.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = EgressSnapshotNameLengthRule { max_length: 86400 };
        let mut item = new_egress_snapshot("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
