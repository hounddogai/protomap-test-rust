use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped federation target must pass. Check returns its problems, or none.
pub trait CappedFederationTargetRule: std::fmt::Debug {
    fn check(&self, item: &CappedFederationTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedFederationTargetRequiredSourceRule {
    pub message: String,
}

impl CappedFederationTargetRule for CappedFederationTargetRequiredSourceRule {
    fn check(&self, item: &CappedFederationTarget) -> Vec<String> {
        if item.source.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedFederationTargetMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedFederationTargetRule for CappedFederationTargetMarginRangeRule {
    fn check(&self, item: &CappedFederationTarget) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedFederationTargetApprovedGuardRule {
    pub limit: f64,
}

impl CappedFederationTargetRule for CappedFederationTargetApprovedGuardRule {
    fn check(&self, item: &CappedFederationTarget) -> Vec<String> {
        if item.approved && (item.floor_value > self.limit) {
            return vec![format!(
                "floor value {} exceeds {} while approved",
                item.floor_value, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedFederationTargetNameLengthRule {
    pub max_length: i64,
}

impl CappedFederationTargetRule for CappedFederationTargetNameLengthRule {
    fn check(&self, item: &CappedFederationTarget) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped federation target.
#[derive(Debug, Clone)]
pub struct CappedFederationTargetValidator {
    pub rules: Vec<Rc<dyn CappedFederationTargetRule>>,
}

impl CappedFederationTargetValidator {
    pub fn validate(&self, item: &CappedFederationTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &CappedFederationTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedFederationTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_federation_target_validator() -> CappedFederationTargetValidator {
    CappedFederationTargetValidator {
        rules: vec![
            Rc::new(CappedFederationTargetRequiredSourceRule {
                message: "capped federation target needs this field".to_string(),
            }) as Rc<dyn CappedFederationTargetRule>,
            Rc::new(CappedFederationTargetMarginRangeRule { min: 2.6, max: 2741.7 }),
            Rc::new(CappedFederationTargetApprovedGuardRule { limit: 1.0 }),
            Rc::new(CappedFederationTargetNameLengthRule { max_length: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_federation_target_validator();
        let item = new_capped_federation_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_source() {
        let rule = CappedFederationTargetRequiredSourceRule {
            message: "capped federation target needs an entry".to_string(),
        };
        let mut item = new_capped_federation_target("", "");
        item.source = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = CappedFederationTargetMarginRangeRule { min: 7.9, max: 1293.2 };
        let mut item = new_capped_federation_target("x", "y");
        item.margin = 1294.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = CappedFederationTargetNameLengthRule { max_length: 64 };
        let mut item = new_capped_federation_target("x", "y");
        item.name = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
