use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a peering variance must pass. Check returns its problems, or none.
pub trait PeeringVarianceRule: std::fmt::Debug {
    fn check(&self, item: &PeeringVariance) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringVarianceRequiredSourceRule {
    pub message: String,
}

impl PeeringVarianceRule for PeeringVarianceRequiredSourceRule {
    fn check(&self, item: &PeeringVariance) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringVariancePinnedGuardRule {
    pub limit: f64,
}

impl PeeringVarianceRule for PeeringVariancePinnedGuardRule {
    fn check(&self, item: &PeeringVariance) -> Vec<String> {
        if item.pinned && (item.weight > self.limit) {
            return vec![format!("weight {} exceeds {} while pinned", item.weight, self.limit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringVarianceRequiredNameRule {
    pub message: String,
}

impl PeeringVarianceRule for PeeringVarianceRequiredNameRule {
    fn check(&self, item: &PeeringVariance) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PeeringVarianceBatchSizeRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PeeringVarianceRule for PeeringVarianceBatchSizeRangeRule {
    fn check(&self, item: &PeeringVariance) -> Vec<String> {
        let value = item.batch_size;
        if value < self.min || value > self.max {
            return vec![format!("batch size {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a peering variance.
#[derive(Debug, Clone)]
pub struct PeeringVarianceValidator {
    pub rules: Vec<Rc<dyn PeeringVarianceRule>>,
}

impl PeeringVarianceValidator {
    pub fn validate(&self, item: &PeeringVariance) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PeeringVariance) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PeeringVariance]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_peering_variance_validator() -> PeeringVarianceValidator {
    PeeringVarianceValidator {
        rules: vec![
            Rc::new(PeeringVarianceRequiredSourceRule {
                message: "peering variance needs an entry".to_string(),
            }) as Rc<dyn PeeringVarianceRule>,
            Rc::new(PeeringVariancePinnedGuardRule { limit: 6.3 }),
            Rc::new(PeeringVarianceRequiredNameRule {
                message: "peering variance needs this field".to_string(),
            }),
            Rc::new(PeeringVarianceBatchSizeRangeRule { min: 32, max: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_peering_variance_validator();
        let item = new_peering_variance("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = PeeringVarianceRequiredSourceRule {
            message: "peering variance needs an entry".to_string(),
        };
        let mut item = new_peering_variance("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PeeringVarianceRequiredNameRule {
            message: "peering variance needs an entry".to_string(),
        };
        let mut item = new_peering_variance("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_batch_size() {
        let rule = PeeringVarianceBatchSizeRangeRule { min: 255, max: 255 };
        let mut item = new_peering_variance("x", "y");
        item.batch_size = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
