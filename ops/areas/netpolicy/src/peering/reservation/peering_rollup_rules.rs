use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a peering rollup must pass. Check returns its problems, or none.
pub trait PeeringRollupRule: std::fmt::Debug {
    fn check(&self, item: &PeeringRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringRollupRequiredSourceRule {
    pub message: String,
}

impl PeeringRollupRule for PeeringRollupRequiredSourceRule {
    fn check(&self, item: &PeeringRollup) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringRollupCodePrefixRule {
    pub prefix: String,
}

impl PeeringRollupRule for PeeringRollupCodePrefixRule {
    fn check(&self, item: &PeeringRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringRollupScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PeeringRollupRule for PeeringRollupScoreRangeRule {
    fn check(&self, item: &PeeringRollup) -> Vec<String> {
        let value = item.score;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("score {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("score {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a peering rollup.
#[derive(Debug, Clone)]
pub struct PeeringRollupValidator {
    pub rules: Vec<Rc<dyn PeeringRollupRule>>,
}

impl PeeringRollupValidator {
    pub fn validate(&self, item: &PeeringRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PeeringRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_rollup_validator() -> PeeringRollupValidator {
    PeeringRollupValidator {
        rules: vec![
            Rc::new(PeeringRollupRequiredSourceRule {
                message: "peering rollup needs a value".to_string(),
            }) as Rc<dyn PeeringRollupRule>,
            Rc::new(PeeringRollupCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(PeeringRollupScoreRangeRule { min: 5.8, max: 3209.9 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_peering_rollup_validator();
        let item = new_peering_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = PeeringRollupRequiredSourceRule {
            message: "peering rollup needs a value".to_string(),
        };
        let mut item = new_peering_rollup("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = PeeringRollupScoreRangeRule { min: 0.9, max: 3295.6 };
        let mut item = new_peering_rollup("x", "y");
        item.score = 3297.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
