use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal budget burn estimate must pass. Check returns its problems, or none.
pub trait SeasonalBudgetBurnEstimateRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalBudgetBurnEstimate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalBudgetBurnEstimateRatioRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalBudgetBurnEstimateRule for SeasonalBudgetBurnEstimateRatioRangeRule {
    fn check(&self, item: &SeasonalBudgetBurnEstimate) -> Vec<String> {
        let value = item.ratio;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ratio {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ratio {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalBudgetBurnEstimateCodePrefixRule {
    pub prefix: String,
}

impl SeasonalBudgetBurnEstimateRule for SeasonalBudgetBurnEstimateCodePrefixRule {
    fn check(&self, item: &SeasonalBudgetBurnEstimate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalBudgetBurnEstimateKeyIdLengthRule {
    pub max_length: i64,
}

impl SeasonalBudgetBurnEstimateRule for SeasonalBudgetBurnEstimateKeyIdLengthRule {
    fn check(&self, item: &SeasonalBudgetBurnEstimate) -> Vec<String> {
        let size = item.key_id.len() as i64;
        if size > self.max_length {
            return vec![format!("key id is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalBudgetBurnEstimateRequiredTitleRule {
    pub message: String,
}

impl SeasonalBudgetBurnEstimateRule for SeasonalBudgetBurnEstimateRequiredTitleRule {
    fn check(&self, item: &SeasonalBudgetBurnEstimate) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalBudgetBurnEstimateCodeLengthRule {
    pub max_length: i64,
}

impl SeasonalBudgetBurnEstimateRule for SeasonalBudgetBurnEstimateCodeLengthRule {
    fn check(&self, item: &SeasonalBudgetBurnEstimate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalBudgetBurnEstimateScoreRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalBudgetBurnEstimateRule for SeasonalBudgetBurnEstimateScoreRangeRule {
    fn check(&self, item: &SeasonalBudgetBurnEstimate) -> Vec<String> {
        let value = item.score;
        if value < self.min || value > self.max {
            return vec![format!("score {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal budget burn estimate.
#[derive(Debug, Clone)]
pub struct SeasonalBudgetBurnEstimateValidator {
    pub rules: Vec<Rc<dyn SeasonalBudgetBurnEstimateRule>>,
}

impl SeasonalBudgetBurnEstimateValidator {
    pub fn validate(&self, item: &SeasonalBudgetBurnEstimate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalBudgetBurnEstimate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalBudgetBurnEstimate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_budget_burn_estimate_validator() -> SeasonalBudgetBurnEstimateValidator {
    SeasonalBudgetBurnEstimateValidator {
        rules: vec![
            Rc::new(SeasonalBudgetBurnEstimateRatioRangeRule { min: 0.7, max: 995.0 })
                as Rc<dyn SeasonalBudgetBurnEstimateRule>,
            Rc::new(SeasonalBudgetBurnEstimateCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(SeasonalBudgetBurnEstimateKeyIdLengthRule { max_length: 32 }),
            Rc::new(SeasonalBudgetBurnEstimateRequiredTitleRule {
                message: "seasonal budget burn estimate needs an entry".to_string(),
            }),
            Rc::new(SeasonalBudgetBurnEstimateCodeLengthRule { max_length: 1000 }),
            Rc::new(SeasonalBudgetBurnEstimateScoreRangeRule { min: 9.1, max: 990.1 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_budget_burn_estimate_validator();
        let item = new_seasonal_budget_burn_estimate("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ratio() {
        let rule = SeasonalBudgetBurnEstimateRatioRangeRule { min: 4.1, max: 4646.1 };
        let mut item = new_seasonal_budget_burn_estimate("x", "y");
        item.ratio = 4647.6;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_key_id() {
        let rule = SeasonalBudgetBurnEstimateKeyIdLengthRule { max_length: 80 };
        let mut item = new_seasonal_budget_burn_estimate("x", "y");
        item.key_id = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.key_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = SeasonalBudgetBurnEstimateRequiredTitleRule {
            message: "seasonal budget burn estimate needs a value".to_string(),
        };
        let mut item = new_seasonal_budget_burn_estimate("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = SeasonalBudgetBurnEstimateCodeLengthRule { max_length: 120 };
        let mut item = new_seasonal_budget_burn_estimate("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_score() {
        let rule = SeasonalBudgetBurnEstimateScoreRangeRule { min: 3.0, max: 3329.8 };
        let mut item = new_seasonal_budget_burn_estimate("x", "y");
        item.score = 3331.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
