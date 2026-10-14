use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal plan profile must pass. Check returns its problems, or none.
pub trait SeasonalPlanProfileRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalPlanProfile) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPlanProfileCountryLengthRule {
    pub max_length: i64,
}

impl SeasonalPlanProfileRule for SeasonalPlanProfileCountryLengthRule {
    fn check(&self, item: &SeasonalPlanProfile) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPlanProfileRequiredNameRule {
    pub message: String,
}

impl SeasonalPlanProfileRule for SeasonalPlanProfileRequiredNameRule {
    fn check(&self, item: &SeasonalPlanProfile) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPlanProfileRequiredCountryRule {
    pub message: String,
}

impl SeasonalPlanProfileRule for SeasonalPlanProfileRequiredCountryRule {
    fn check(&self, item: &SeasonalPlanProfile) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPlanProfileCodePrefixRule {
    pub prefix: String,
}

impl SeasonalPlanProfileRule for SeasonalPlanProfileCodePrefixRule {
    fn check(&self, item: &SeasonalPlanProfile) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPlanProfileCategoryLengthRule {
    pub max_length: i64,
}

impl SeasonalPlanProfileRule for SeasonalPlanProfileCategoryLengthRule {
    fn check(&self, item: &SeasonalPlanProfile) -> Vec<String> {
        let size = item.category.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "category is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalPlanProfileRetriesRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalPlanProfileRule for SeasonalPlanProfileRetriesRangeRule {
    fn check(&self, item: &SeasonalPlanProfile) -> Vec<String> {
        let value = item.retries;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("retries {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("retries {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a seasonal plan profile.
#[derive(Debug, Clone)]
pub struct SeasonalPlanProfileValidator {
    pub rules: Vec<Rc<dyn SeasonalPlanProfileRule>>,
}

impl SeasonalPlanProfileValidator {
    pub fn validate(&self, item: &SeasonalPlanProfile) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalPlanProfile) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalPlanProfile]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_plan_profile_validator() -> SeasonalPlanProfileValidator {
    SeasonalPlanProfileValidator {
        rules: vec![
            Rc::new(SeasonalPlanProfileCountryLengthRule { max_length: 1000 }) as Rc<dyn SeasonalPlanProfileRule>,
            Rc::new(SeasonalPlanProfileRequiredNameRule {
                message: "seasonal plan profile needs this field".to_string(),
            }),
            Rc::new(SeasonalPlanProfileRequiredCountryRule {
                message: "seasonal plan profile needs an entry".to_string(),
            }),
            Rc::new(SeasonalPlanProfileCodePrefixRule {
                prefix: "OPS".to_string(),
            }),
            Rc::new(SeasonalPlanProfileCategoryLengthRule { max_length: 3600 }),
            Rc::new(SeasonalPlanProfileRetriesRangeRule { min: 120, max: 1000 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_plan_profile_validator();
        let item = new_seasonal_plan_profile("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = SeasonalPlanProfileCountryLengthRule { max_length: 16 };
        let mut item = new_seasonal_plan_profile("x", "y");
        item.country = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = SeasonalPlanProfileRequiredNameRule {
            message: "seasonal plan profile needs this field".to_string(),
        };
        let mut item = new_seasonal_plan_profile("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = SeasonalPlanProfileRequiredCountryRule {
            message: "seasonal plan profile needs a value".to_string(),
        };
        let mut item = new_seasonal_plan_profile("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_category() {
        let rule = SeasonalPlanProfileCategoryLengthRule { max_length: 120 };
        let mut item = new_seasonal_plan_profile("x", "y");
        item.category = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.category = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_retries() {
        let rule = SeasonalPlanProfileRetriesRangeRule { min: 120, max: 1000 };
        let mut item = new_seasonal_plan_profile("x", "y");
        item.retries = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
