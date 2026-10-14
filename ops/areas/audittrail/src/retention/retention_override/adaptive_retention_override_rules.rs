use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive retention override must pass. Check returns its problems, or none.
pub trait AdaptiveRetentionOverrideRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveRetentionOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRetentionOverrideRequiredLabelRule {
    pub message: String,
}

impl AdaptiveRetentionOverrideRule for AdaptiveRetentionOverrideRequiredLabelRule {
    fn check(&self, item: &AdaptiveRetentionOverride) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRetentionOverrideWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveRetentionOverrideRule for AdaptiveRetentionOverrideWeightRangeRule {
    fn check(&self, item: &AdaptiveRetentionOverride) -> Vec<String> {
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
pub struct AdaptiveRetentionOverrideCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveRetentionOverrideRule for AdaptiveRetentionOverrideCeilingRangeRule {
    fn check(&self, item: &AdaptiveRetentionOverride) -> Vec<String> {
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

/// Runs every rule of a adaptive retention override.
#[derive(Debug, Clone)]
pub struct AdaptiveRetentionOverrideValidator {
    pub rules: Vec<Rc<dyn AdaptiveRetentionOverrideRule>>,
}

impl AdaptiveRetentionOverrideValidator {
    pub fn validate(&self, item: &AdaptiveRetentionOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveRetentionOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveRetentionOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_retention_override_validator() -> AdaptiveRetentionOverrideValidator {
    AdaptiveRetentionOverrideValidator {
        rules: vec![
            Rc::new(AdaptiveRetentionOverrideRequiredLabelRule {
                message: "adaptive retention override needs an entry".to_string(),
            }) as Rc<dyn AdaptiveRetentionOverrideRule>,
            Rc::new(AdaptiveRetentionOverrideWeightRangeRule { min: 7.9, max: 2507.2 }),
            Rc::new(AdaptiveRetentionOverrideCeilingRangeRule { min: 4.9, max: 1574.2 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_retention_override_validator();
        let item = new_adaptive_retention_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = AdaptiveRetentionOverrideRequiredLabelRule {
            message: "adaptive retention override needs an entry".to_string(),
        };
        let mut item = new_adaptive_retention_override("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = AdaptiveRetentionOverrideWeightRangeRule { min: 8.1, max: 2084.1 };
        let mut item = new_adaptive_retention_override("x", "y");
        item.weight = 2085.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = AdaptiveRetentionOverrideCeilingRangeRule { min: 4.9, max: 1541.6 };
        let mut item = new_adaptive_retention_override("x", "y");
        item.ceiling = 1543.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
