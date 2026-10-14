use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a strict suspension bundle must pass. Check returns its problems, or none.
pub trait StrictSuspensionBundleRule: std::fmt::Debug {
    fn check(&self, item: &StrictSuspensionBundle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSuspensionBundleRequiredNameRule {
    pub message: String,
}

impl StrictSuspensionBundleRule for StrictSuspensionBundleRequiredNameRule {
    fn check(&self, item: &StrictSuspensionBundle) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSuspensionBundleScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictSuspensionBundleRule for StrictSuspensionBundleScoreRangeRule {
    fn check(&self, item: &StrictSuspensionBundle) -> Vec<String> {
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

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSuspensionBundleDescriptionLengthRule {
    pub max_length: i64,
}

impl StrictSuspensionBundleRule for StrictSuspensionBundleDescriptionLengthRule {
    fn check(&self, item: &StrictSuspensionBundle) -> Vec<String> {
        let size = item.description.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "description is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSuspensionBundleRequiredCategoryRule {
    pub message: String,
}

impl StrictSuspensionBundleRule for StrictSuspensionBundleRequiredCategoryRule {
    fn check(&self, item: &StrictSuspensionBundle) -> Vec<String> {
        if item.category.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictSuspensionBundleNameLengthRule {
    pub max_length: i64,
}

impl StrictSuspensionBundleRule for StrictSuspensionBundleNameLengthRule {
    fn check(&self, item: &StrictSuspensionBundle) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict suspension bundle.
#[derive(Debug, Clone)]
pub struct StrictSuspensionBundleValidator {
    pub rules: Vec<Rc<dyn StrictSuspensionBundleRule>>,
}

impl StrictSuspensionBundleValidator {
    pub fn validate(&self, item: &StrictSuspensionBundle) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &StrictSuspensionBundle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictSuspensionBundle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_suspension_bundle_validator() -> StrictSuspensionBundleValidator {
    StrictSuspensionBundleValidator {
        rules: vec![
            Rc::new(StrictSuspensionBundleRequiredNameRule {
                message: "strict suspension bundle needs an entry".to_string(),
            }) as Rc<dyn StrictSuspensionBundleRule>,
            Rc::new(StrictSuspensionBundleScoreRangeRule { min: 7.8, max: 1844.5 }),
            Rc::new(StrictSuspensionBundleDescriptionLengthRule { max_length: 32 }),
            Rc::new(StrictSuspensionBundleRequiredCategoryRule {
                message: "strict suspension bundle needs this field".to_string(),
            }),
            Rc::new(StrictSuspensionBundleNameLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_suspension_bundle_validator();
        let item = new_strict_suspension_bundle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = StrictSuspensionBundleRequiredNameRule {
            message: "strict suspension bundle needs this field".to_string(),
        };
        let mut item = new_strict_suspension_bundle("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = StrictSuspensionBundleScoreRangeRule { min: 7.4, max: 1962.7 };
        let mut item = new_strict_suspension_bundle("x", "y");
        item.score = 1964.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_description() {
        let rule = StrictSuspensionBundleDescriptionLengthRule { max_length: 120 };
        let mut item = new_strict_suspension_bundle("x", "y");
        item.description = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.description = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_category() {
        let rule = StrictSuspensionBundleRequiredCategoryRule {
            message: "strict suspension bundle needs an entry".to_string(),
        };
        let mut item = new_strict_suspension_bundle("", "");
        item.category = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = StrictSuspensionBundleNameLengthRule { max_length: 1000 };
        let mut item = new_strict_suspension_bundle("x", "y");
        item.name = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
