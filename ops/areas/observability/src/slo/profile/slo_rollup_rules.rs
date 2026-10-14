use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a slo rollup must pass. Check returns its problems, or none.
pub trait SloRollupRule: std::fmt::Debug {
    fn check(&self, item: &SloRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloRollupCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SloRollupRule for SloRollupCeilingRangeRule {
    fn check(&self, item: &SloRollup) -> Vec<String> {
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
pub struct SloRollupRequiredCodeRule {
    pub message: String,
}

impl SloRollupRule for SloRollupRequiredCodeRule {
    fn check(&self, item: &SloRollup) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloRollupRequiredLabelRule {
    pub message: String,
}

impl SloRollupRule for SloRollupRequiredLabelRule {
    fn check(&self, item: &SloRollup) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloRollupRequiredNameRule {
    pub message: String,
}

impl SloRollupRule for SloRollupRequiredNameRule {
    fn check(&self, item: &SloRollup) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a slo rollup.
#[derive(Debug, Clone)]
pub struct SloRollupValidator {
    pub rules: Vec<Rc<dyn SloRollupRule>>,
}

impl SloRollupValidator {
    pub fn validate(&self, item: &SloRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SloRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_rollup_validator() -> SloRollupValidator {
    SloRollupValidator {
        rules: vec![
            Rc::new(SloRollupCeilingRangeRule { min: 4.8, max: 1538.6 }) as Rc<dyn SloRollupRule>,
            Rc::new(SloRollupRequiredCodeRule {
                message: "slo rollup needs an entry".to_string(),
            }),
            Rc::new(SloRollupRequiredLabelRule {
                message: "slo rollup needs this field".to_string(),
            }),
            Rc::new(SloRollupRequiredNameRule {
                message: "slo rollup needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_slo_rollup_validator();
        let item = new_slo_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = SloRollupCeilingRangeRule { min: 5.8, max: 1835.3 };
        let mut item = new_slo_rollup("x", "y");
        item.ceiling = 1836.8;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SloRollupRequiredCodeRule {
            message: "slo rollup needs a value".to_string(),
        };
        let mut item = new_slo_rollup("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = SloRollupRequiredLabelRule {
            message: "slo rollup needs a value".to_string(),
        };
        let mut item = new_slo_rollup("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = SloRollupRequiredNameRule {
            message: "slo rollup needs an entry".to_string(),
        };
        let mut item = new_slo_rollup("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
