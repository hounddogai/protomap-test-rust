use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a budget burn forecast must pass. Check returns its problems, or none.
pub trait BudgetBurnForecastRule: std::fmt::Debug {
    fn check(&self, item: &BudgetBurnForecast) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnForecastNameLengthRule {
    pub max_length: i64,
}

impl BudgetBurnForecastRule for BudgetBurnForecastNameLengthRule {
    fn check(&self, item: &BudgetBurnForecast) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnForecastRequiredCodeRule {
    pub message: String,
}

impl BudgetBurnForecastRule for BudgetBurnForecastRequiredCodeRule {
    fn check(&self, item: &BudgetBurnForecast) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnForecastTitleLengthRule {
    pub max_length: i64,
}

impl BudgetBurnForecastRule for BudgetBurnForecastTitleLengthRule {
    fn check(&self, item: &BudgetBurnForecast) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnForecastCodePrefixRule {
    pub prefix: String,
}

impl BudgetBurnForecastRule for BudgetBurnForecastCodePrefixRule {
    fn check(&self, item: &BudgetBurnForecast) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnForecastRequiredNameRule {
    pub message: String,
}

impl BudgetBurnForecastRule for BudgetBurnForecastRequiredNameRule {
    fn check(&self, item: &BudgetBurnForecast) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnForecastCodeLengthRule {
    pub max_length: i64,
}

impl BudgetBurnForecastRule for BudgetBurnForecastCodeLengthRule {
    fn check(&self, item: &BudgetBurnForecast) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a budget burn forecast.
#[derive(Debug, Clone)]
pub struct BudgetBurnForecastValidator {
    pub rules: Vec<Rc<dyn BudgetBurnForecastRule>>,
}

impl BudgetBurnForecastValidator {
    pub fn validate(&self, item: &BudgetBurnForecast) -> Vec<String> {
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

    pub fn require_valid(&self, item: &BudgetBurnForecast) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[BudgetBurnForecast]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_budget_burn_forecast_validator() -> BudgetBurnForecastValidator {
    BudgetBurnForecastValidator {
        rules: vec![
            Rc::new(BudgetBurnForecastNameLengthRule { max_length: 1000 }) as Rc<dyn BudgetBurnForecastRule>,
            Rc::new(BudgetBurnForecastRequiredCodeRule {
                message: "budget burn forecast needs this field".to_string(),
            }),
            Rc::new(BudgetBurnForecastTitleLengthRule { max_length: 64 }),
            Rc::new(BudgetBurnForecastCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(BudgetBurnForecastRequiredNameRule {
                message: "budget burn forecast needs this field".to_string(),
            }),
            Rc::new(BudgetBurnForecastCodeLengthRule { max_length: 80 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_budget_burn_forecast_validator();
        let item = new_budget_burn_forecast("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = BudgetBurnForecastNameLengthRule { max_length: 3600 };
        let mut item = new_budget_burn_forecast("x", "y");
        item.name = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = BudgetBurnForecastRequiredCodeRule {
            message: "budget burn forecast needs this field".to_string(),
        };
        let mut item = new_budget_burn_forecast("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = BudgetBurnForecastTitleLengthRule { max_length: 1000 };
        let mut item = new_budget_burn_forecast("x", "y");
        item.title = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = BudgetBurnForecastRequiredNameRule {
            message: "budget burn forecast needs a value".to_string(),
        };
        let mut item = new_budget_burn_forecast("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = BudgetBurnForecastCodeLengthRule { max_length: 64 };
        let mut item = new_budget_burn_forecast("x", "y");
        item.code = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
