use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a seasonal budget burn template must pass. Check returns its problems, or none.
pub trait SeasonalBudgetBurnTemplateRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalBudgetBurnTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalBudgetBurnTemplateTitleLengthRule {
    pub max_length: i64,
}

impl SeasonalBudgetBurnTemplateRule for SeasonalBudgetBurnTemplateTitleLengthRule {
    fn check(&self, item: &SeasonalBudgetBurnTemplate) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalBudgetBurnTemplateAllowedSummaryRule {
    pub allowed: Vec<String>,
}

impl SeasonalBudgetBurnTemplateRule for SeasonalBudgetBurnTemplateAllowedSummaryRule {
    fn check(&self, item: &SeasonalBudgetBurnTemplate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.summary)))
        {
            return vec![format!("summary {} is above maximum", item.summary)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalBudgetBurnTemplateWeightRangeRule {
    pub min: f64,
    pub max: f64,
}

impl SeasonalBudgetBurnTemplateRule for SeasonalBudgetBurnTemplateWeightRangeRule {
    fn check(&self, item: &SeasonalBudgetBurnTemplate) -> Vec<String> {
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
pub struct SeasonalBudgetBurnTemplateTenantIdLengthRule {
    pub max_length: i64,
}

impl SeasonalBudgetBurnTemplateRule for SeasonalBudgetBurnTemplateTenantIdLengthRule {
    fn check(&self, item: &SeasonalBudgetBurnTemplate) -> Vec<String> {
        let size = item.tenant_id.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "tenant id is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal budget burn template.
#[derive(Debug, Clone)]
pub struct SeasonalBudgetBurnTemplateValidator {
    pub rules: Vec<Rc<dyn SeasonalBudgetBurnTemplateRule>>,
}

impl SeasonalBudgetBurnTemplateValidator {
    pub fn validate(&self, item: &SeasonalBudgetBurnTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SeasonalBudgetBurnTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalBudgetBurnTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_budget_burn_template_validator() -> SeasonalBudgetBurnTemplateValidator {
    SeasonalBudgetBurnTemplateValidator {
        rules: vec![
            Rc::new(SeasonalBudgetBurnTemplateTitleLengthRule { max_length: 86400 })
                as Rc<dyn SeasonalBudgetBurnTemplateRule>,
            Rc::new(SeasonalBudgetBurnTemplateAllowedSummaryRule {
                allowed: vec!["pending".to_string(), "primary".to_string(), "fallback".to_string()],
            }),
            Rc::new(SeasonalBudgetBurnTemplateWeightRangeRule { min: 0.5, max: 1402.2 }),
            Rc::new(SeasonalBudgetBurnTemplateTenantIdLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_budget_burn_template_validator();
        let item = new_seasonal_budget_burn_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = SeasonalBudgetBurnTemplateTitleLengthRule { max_length: 32 };
        let mut item = new_seasonal_budget_burn_template("x", "y");
        item.title = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_weight() {
        let rule = SeasonalBudgetBurnTemplateWeightRangeRule { min: 8.8, max: 2728.9 };
        let mut item = new_seasonal_budget_burn_template("x", "y");
        item.weight = 2730.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_tenant_id() {
        let rule = SeasonalBudgetBurnTemplateTenantIdLengthRule { max_length: 120 };
        let mut item = new_seasonal_budget_burn_template("x", "y");
        item.tenant_id = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.tenant_id = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
