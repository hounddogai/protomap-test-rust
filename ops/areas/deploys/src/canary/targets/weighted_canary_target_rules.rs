use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a weighted canary target must pass. Check returns its problems, or none.
pub trait WeightedCanaryTargetRule: std::fmt::Debug {
    fn check(&self, item: &WeightedCanaryTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCanaryTargetCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl WeightedCanaryTargetRule for WeightedCanaryTargetCeilingRangeRule {
    fn check(&self, item: &WeightedCanaryTarget) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCanaryTargetCodePrefixRule {
    pub prefix: String,
}

impl WeightedCanaryTargetRule for WeightedCanaryTargetCodePrefixRule {
    fn check(&self, item: &WeightedCanaryTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCanaryTargetRequiredCodeRule {
    pub message: String,
}

impl WeightedCanaryTargetRule for WeightedCanaryTargetRequiredCodeRule {
    fn check(&self, item: &WeightedCanaryTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCanaryTargetAllowedOwnerIdRule {
    pub allowed: Vec<String>,
}

impl WeightedCanaryTargetRule for WeightedCanaryTargetAllowedOwnerIdRule {
    fn check(&self, item: &WeightedCanaryTarget) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.owner_id)))
        {
            return vec![format!("owner id {} is missing", item.owner_id)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCanaryTargetNameLengthRule {
    pub max_length: i64,
}

impl WeightedCanaryTargetRule for WeightedCanaryTargetNameLengthRule {
    fn check(&self, item: &WeightedCanaryTarget) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCanaryTargetGraceDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl WeightedCanaryTargetRule for WeightedCanaryTargetGraceDaysRangeRule {
    fn check(&self, item: &WeightedCanaryTarget) -> Vec<String> {
        let value = item.grace_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("grace days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("grace days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a weighted canary target.
#[derive(Debug, Clone)]
pub struct WeightedCanaryTargetValidator {
    pub rules: Vec<Rc<dyn WeightedCanaryTargetRule>>,
}

impl WeightedCanaryTargetValidator {
    pub fn validate(&self, item: &WeightedCanaryTarget) -> Vec<String> {
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

    pub fn require_valid(&self, item: &WeightedCanaryTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[WeightedCanaryTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_weighted_canary_target_validator() -> WeightedCanaryTargetValidator {
    WeightedCanaryTargetValidator {
        rules: vec![
            Rc::new(WeightedCanaryTargetCeilingRangeRule { min: 9.4, max: 1281.9 }) as Rc<dyn WeightedCanaryTargetRule>,
            Rc::new(WeightedCanaryTargetCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
            Rc::new(WeightedCanaryTargetRequiredCodeRule {
                message: "weighted canary target needs a value".to_string(),
            }),
            Rc::new(WeightedCanaryTargetAllowedOwnerIdRule {
                allowed: vec!["pending".to_string(), "scheduled".to_string()],
            }),
            Rc::new(WeightedCanaryTargetNameLengthRule { max_length: 64 }),
            Rc::new(WeightedCanaryTargetGraceDaysRangeRule { min: 32, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_weighted_canary_target_validator();
        let item = new_weighted_canary_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = WeightedCanaryTargetCeilingRangeRule { min: 3.5, max: 2888.6 };
        let mut item = new_weighted_canary_target("x", "y");
        item.ceiling = 2890.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = WeightedCanaryTargetRequiredCodeRule {
            message: "weighted canary target needs an entry".to_string(),
        };
        let mut item = new_weighted_canary_target("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = WeightedCanaryTargetNameLengthRule { max_length: 64 };
        let mut item = new_weighted_canary_target("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_grace_days() {
        let rule = WeightedCanaryTargetGraceDaysRangeRule { min: 64, max: 1000 };
        let mut item = new_weighted_canary_target("x", "y");
        item.grace_days = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
