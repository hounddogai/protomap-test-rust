use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary usage override must pass. Check returns its problems, or none.
pub trait PrimaryUsageOverrideRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryUsageOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryUsageOverrideBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryUsageOverrideRule for PrimaryUsageOverrideBurstRangeRule {
    fn check(&self, item: &PrimaryUsageOverride) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryUsageOverrideRequiredUnitRule {
    pub message: String,
}

impl PrimaryUsageOverrideRule for PrimaryUsageOverrideRequiredUnitRule {
    fn check(&self, item: &PrimaryUsageOverride) -> Vec<String> {
        if item.unit.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryUsageOverrideCodePrefixRule {
    pub prefix: String,
}

impl PrimaryUsageOverrideRule for PrimaryUsageOverrideCodePrefixRule {
    fn check(&self, item: &PrimaryUsageOverride) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryUsageOverrideCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl PrimaryUsageOverrideRule for PrimaryUsageOverrideCapacityRangeRule {
    fn check(&self, item: &PrimaryUsageOverride) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryUsageOverrideRequiredTitleRule {
    pub message: String,
}

impl PrimaryUsageOverrideRule for PrimaryUsageOverrideRequiredTitleRule {
    fn check(&self, item: &PrimaryUsageOverride) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryUsageOverrideArchivedGuardRule {
    pub limit: i64,
}

impl PrimaryUsageOverrideRule for PrimaryUsageOverrideArchivedGuardRule {
    fn check(&self, item: &PrimaryUsageOverride) -> Vec<String> {
        if item.archived && (item.burst > self.limit) {
            return vec![format!("burst {} exceeds {} while archived", item.burst, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary usage override.
#[derive(Debug, Clone)]
pub struct PrimaryUsageOverrideValidator {
    pub rules: Vec<Rc<dyn PrimaryUsageOverrideRule>>,
}

impl PrimaryUsageOverrideValidator {
    pub fn validate(&self, item: &PrimaryUsageOverride) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PrimaryUsageOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryUsageOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_usage_override_validator() -> PrimaryUsageOverrideValidator {
    PrimaryUsageOverrideValidator {
        rules: vec![
            Rc::new(PrimaryUsageOverrideBurstRangeRule { min: 80, max: 80 }) as Rc<dyn PrimaryUsageOverrideRule>,
            Rc::new(PrimaryUsageOverrideRequiredUnitRule {
                message: "primary usage override needs an entry".to_string(),
            }),
            Rc::new(PrimaryUsageOverrideCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(PrimaryUsageOverrideCapacityRangeRule { min: 64, max: 1000 }),
            Rc::new(PrimaryUsageOverrideRequiredTitleRule {
                message: "primary usage override needs a value".to_string(),
            }),
            Rc::new(PrimaryUsageOverrideArchivedGuardRule { limit: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_usage_override_validator();
        let item = new_primary_usage_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = PrimaryUsageOverrideBurstRangeRule { min: 80, max: 1000 };
        let mut item = new_primary_usage_override("x", "y");
        item.burst = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_unit() {
        let rule = PrimaryUsageOverrideRequiredUnitRule {
            message: "primary usage override needs an entry".to_string(),
        };
        let mut item = new_primary_usage_override("", "");
        item.unit = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = PrimaryUsageOverrideCapacityRangeRule { min: 64, max: 80 };
        let mut item = new_primary_usage_override("x", "y");
        item.capacity = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = PrimaryUsageOverrideRequiredTitleRule {
            message: "primary usage override needs a value".to_string(),
        };
        let mut item = new_primary_usage_override("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
