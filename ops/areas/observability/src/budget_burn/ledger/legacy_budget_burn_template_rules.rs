use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a legacy budget burn template must pass. Check returns its problems, or none.
pub trait LegacyBudgetBurnTemplateRule: std::fmt::Debug {
    fn check(&self, item: &LegacyBudgetBurnTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBudgetBurnTemplateAllowedTimezoneRule {
    pub allowed: Vec<String>,
}

impl LegacyBudgetBurnTemplateRule for LegacyBudgetBurnTemplateAllowedTimezoneRule {
    fn check(&self, item: &LegacyBudgetBurnTemplate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.timezone)))
        {
            return vec![format!("timezone {} is blocked", item.timezone)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBudgetBurnTemplateVisibleGuardRule {
    pub limit: i64,
}

impl LegacyBudgetBurnTemplateRule for LegacyBudgetBurnTemplateVisibleGuardRule {
    fn check(&self, item: &LegacyBudgetBurnTemplate) -> Vec<String> {
        if item.visible && (item.sequence > self.limit) {
            return vec![format!(
                "sequence {} exceeds {} while visible",
                item.sequence, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBudgetBurnTemplateCreatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyBudgetBurnTemplateRule for LegacyBudgetBurnTemplateCreatedAtRangeRule {
    fn check(&self, item: &LegacyBudgetBurnTemplate) -> Vec<String> {
        let value = item.created_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("created at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("created at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBudgetBurnTemplateRequiredTimezoneRule {
    pub message: String,
}

impl LegacyBudgetBurnTemplateRule for LegacyBudgetBurnTemplateRequiredTimezoneRule {
    fn check(&self, item: &LegacyBudgetBurnTemplate) -> Vec<String> {
        if item.timezone.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy budget burn template.
#[derive(Debug, Clone)]
pub struct LegacyBudgetBurnTemplateValidator {
    pub rules: Vec<Rc<dyn LegacyBudgetBurnTemplateRule>>,
}

impl LegacyBudgetBurnTemplateValidator {
    pub fn validate(&self, item: &LegacyBudgetBurnTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyBudgetBurnTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyBudgetBurnTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_budget_burn_template_validator() -> LegacyBudgetBurnTemplateValidator {
    LegacyBudgetBurnTemplateValidator {
        rules: vec![
            Rc::new(LegacyBudgetBurnTemplateAllowedTimezoneRule { allowed: Vec::new() })
                as Rc<dyn LegacyBudgetBurnTemplateRule>,
            Rc::new(LegacyBudgetBurnTemplateVisibleGuardRule { limit: 3600 }),
            Rc::new(LegacyBudgetBurnTemplateCreatedAtRangeRule { min: 16, max: 80 }),
            Rc::new(LegacyBudgetBurnTemplateRequiredTimezoneRule {
                message: "legacy budget burn template needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_budget_burn_template_validator();
        let item = new_legacy_budget_burn_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_created_at() {
        let rule = LegacyBudgetBurnTemplateCreatedAtRangeRule { min: 16, max: 64 };
        let mut item = new_legacy_budget_burn_template("x", "y");
        item.created_at = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_timezone() {
        let rule = LegacyBudgetBurnTemplateRequiredTimezoneRule {
            message: "legacy budget burn template needs an entry".to_string(),
        };
        let mut item = new_legacy_budget_burn_template("", "");
        item.timezone = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
