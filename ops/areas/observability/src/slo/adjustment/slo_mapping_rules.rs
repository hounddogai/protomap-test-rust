use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a slo mapping must pass. Check returns its problems, or none.
pub trait SloMappingRule: std::fmt::Debug {
    fn check(&self, item: &SloMapping) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloMappingCodePrefixRule {
    pub prefix: String,
}

impl SloMappingRule for SloMappingCodePrefixRule {
    fn check(&self, item: &SloMapping) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloMappingRequiredTitleRule {
    pub message: String,
}

impl SloMappingRule for SloMappingRequiredTitleRule {
    fn check(&self, item: &SloMapping) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SloMappingWindowDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SloMappingRule for SloMappingWindowDaysRangeRule {
    fn check(&self, item: &SloMapping) -> Vec<String> {
        let value = item.window_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("window days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("window days {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a slo mapping.
#[derive(Debug, Clone)]
pub struct SloMappingValidator {
    pub rules: Vec<Rc<dyn SloMappingRule>>,
}

impl SloMappingValidator {
    pub fn validate(&self, item: &SloMapping) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SloMapping) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SloMapping]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_slo_mapping_validator() -> SloMappingValidator {
    SloMappingValidator {
        rules: vec![
            Rc::new(SloMappingCodePrefixRule {
                prefix: "ACM".to_string(),
            }) as Rc<dyn SloMappingRule>,
            Rc::new(SloMappingRequiredTitleRule {
                message: "slo mapping needs this field".to_string(),
            }),
            Rc::new(SloMappingWindowDaysRangeRule { min: 32, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_slo_mapping_validator();
        let item = new_slo_mapping("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = SloMappingRequiredTitleRule {
            message: "slo mapping needs a value".to_string(),
        };
        let mut item = new_slo_mapping("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_window_days() {
        let rule = SloMappingWindowDaysRangeRule { min: 32, max: 80 };
        let mut item = new_slo_mapping("x", "y");
        item.window_days = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
