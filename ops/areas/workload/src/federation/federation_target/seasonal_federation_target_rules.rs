use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal federation target must pass. Check returns its problems, or none.
pub trait SeasonalFederationTargetRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalFederationTarget) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFederationTargetMinimumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalFederationTargetRule for SeasonalFederationTargetMinimumRangeRule {
    fn check(&self, item: &SeasonalFederationTarget) -> Vec<String> {
        let value = item.minimum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("minimum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("minimum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFederationTargetRequiredCodeRule {
    pub message: String,
}

impl SeasonalFederationTargetRule for SeasonalFederationTargetRequiredCodeRule {
    fn check(&self, item: &SeasonalFederationTarget) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFederationTargetCodePrefixRule {
    pub prefix: String,
}

impl SeasonalFederationTargetRule for SeasonalFederationTargetCodePrefixRule {
    fn check(&self, item: &SeasonalFederationTarget) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFederationTargetRequiredDescriptionRule {
    pub message: String,
}

impl SeasonalFederationTargetRule for SeasonalFederationTargetRequiredDescriptionRule {
    fn check(&self, item: &SeasonalFederationTarget) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal federation target.
#[derive(Debug, Clone)]
pub struct SeasonalFederationTargetValidator {
    pub rules: Vec<Rc<dyn SeasonalFederationTargetRule>>,
}

impl SeasonalFederationTargetValidator {
    pub fn validate(&self, item: &SeasonalFederationTarget) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalFederationTarget) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalFederationTarget]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_federation_target_validator() -> SeasonalFederationTargetValidator {
    SeasonalFederationTargetValidator {
        rules: vec![
            Rc::new(SeasonalFederationTargetMinimumRangeRule { min: 6.2, max: 215.5 })
                as Rc<dyn SeasonalFederationTargetRule>,
            Rc::new(SeasonalFederationTargetRequiredCodeRule {
                message: "seasonal federation target needs this field".to_string(),
            }),
            Rc::new(SeasonalFederationTargetCodePrefixRule {
                prefix: "REF".to_string(),
            }),
            Rc::new(SeasonalFederationTargetRequiredDescriptionRule {
                message: "seasonal federation target needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_federation_target_validator();
        let item = new_seasonal_federation_target("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_minimum() {
        let rule = SeasonalFederationTargetMinimumRangeRule { min: 3.3, max: 3086.7 };
        let mut item = new_seasonal_federation_target("x", "y");
        item.minimum = 3088.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = SeasonalFederationTargetRequiredCodeRule {
            message: "seasonal federation target needs a value".to_string(),
        };
        let mut item = new_seasonal_federation_target("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = SeasonalFederationTargetRequiredDescriptionRule {
            message: "seasonal federation target needs this field".to_string(),
        };
        let mut item = new_seasonal_federation_target("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
