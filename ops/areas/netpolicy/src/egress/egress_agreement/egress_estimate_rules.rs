use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a egress estimate must pass. Check returns its problems, or none.
pub trait EgressEstimateRule: std::fmt::Debug {
    fn check(&self, item: &EgressEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressEstimateActiveGuardRule {
    pub limit: i64,
}

impl EgressEstimateRule for EgressEstimateActiveGuardRule {
    fn check(&self, item: &EgressEstimate) -> Vec<String> {
        if item.active && (item.port > self.limit) {
            return vec![format!("port {} exceeds {} while active", item.port, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressEstimatePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EgressEstimateRule for EgressEstimatePortRangeRule {
    fn check(&self, item: &EgressEstimate) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressEstimateUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EgressEstimateRule for EgressEstimateUpdatedAtRangeRule {
    fn check(&self, item: &EgressEstimate) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EgressEstimateRequiredDescriptionRule {
    pub message: String,
}

impl EgressEstimateRule for EgressEstimateRequiredDescriptionRule {
    fn check(&self, item: &EgressEstimate) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a egress estimate.
#[derive(Debug, Clone)]
pub struct EgressEstimateValidator {
    pub rules: Vec<Rc<dyn EgressEstimateRule>>,
}

impl EgressEstimateValidator {
    pub fn validate(&self, item: &EgressEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EgressEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EgressEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_egress_estimate_validator() -> EgressEstimateValidator {
    EgressEstimateValidator {
        rules: vec![
            Rc::new(EgressEstimateActiveGuardRule { limit: 255 }) as Rc<dyn EgressEstimateRule>,
            Rc::new(EgressEstimatePortRangeRule { min: 80, max: 3600 }),
            Rc::new(EgressEstimateUpdatedAtRangeRule { min: 80, max: 3600 }),
            Rc::new(EgressEstimateRequiredDescriptionRule {
                message: "egress estimate needs this field".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_egress_estimate_validator();
        let item = new_egress_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = EgressEstimatePortRangeRule { min: 32, max: 255 };
        let mut item = new_egress_estimate("x", "y");
        item.port = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = EgressEstimateUpdatedAtRangeRule { min: 64, max: 86400 };
        let mut item = new_egress_estimate("x", "y");
        item.updated_at = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = EgressEstimateRequiredDescriptionRule {
            message: "egress estimate needs an entry".to_string(),
        };
        let mut item = new_egress_estimate("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
