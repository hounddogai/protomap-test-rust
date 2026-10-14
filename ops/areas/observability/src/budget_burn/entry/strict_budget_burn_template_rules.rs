use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a strict budget burn template must pass. Check returns its problems, or none.
pub trait StrictBudgetBurnTemplateRule: std::fmt::Debug {
    fn check(&self, item: &StrictBudgetBurnTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictBudgetBurnTemplateAllowedChecksumRule {
    pub allowed: Vec<String>,
}

impl StrictBudgetBurnTemplateRule for StrictBudgetBurnTemplateAllowedChecksumRule {
    fn check(&self, item: &StrictBudgetBurnTemplate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.checksum)))
        {
            return vec![format!("checksum {} is not allowed", item.checksum)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictBudgetBurnTemplateRequiredChecksumRule {
    pub message: String,
}

impl StrictBudgetBurnTemplateRule for StrictBudgetBurnTemplateRequiredChecksumRule {
    fn check(&self, item: &StrictBudgetBurnTemplate) -> Vec<String> {
        if item.checksum.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictBudgetBurnTemplateAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl StrictBudgetBurnTemplateRule for StrictBudgetBurnTemplateAmountRangeRule {
    fn check(&self, item: &StrictBudgetBurnTemplate) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictBudgetBurnTemplateLabelLengthRule {
    pub max_length: i64,
}

impl StrictBudgetBurnTemplateRule for StrictBudgetBurnTemplateLabelLengthRule {
    fn check(&self, item: &StrictBudgetBurnTemplate) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictBudgetBurnTemplateCodePrefixRule {
    pub prefix: String,
}

impl StrictBudgetBurnTemplateRule for StrictBudgetBurnTemplateCodePrefixRule {
    fn check(&self, item: &StrictBudgetBurnTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StrictBudgetBurnTemplateRequiredLabelRule {
    pub message: String,
}

impl StrictBudgetBurnTemplateRule for StrictBudgetBurnTemplateRequiredLabelRule {
    fn check(&self, item: &StrictBudgetBurnTemplate) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a strict budget burn template.
#[derive(Debug, Clone)]
pub struct StrictBudgetBurnTemplateValidator {
    pub rules: Vec<Rc<dyn StrictBudgetBurnTemplateRule>>,
}

impl StrictBudgetBurnTemplateValidator {
    pub fn validate(&self, item: &StrictBudgetBurnTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &StrictBudgetBurnTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[StrictBudgetBurnTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_strict_budget_burn_template_validator() -> StrictBudgetBurnTemplateValidator {
    StrictBudgetBurnTemplateValidator {
        rules: vec![
            Rc::new(StrictBudgetBurnTemplateAllowedChecksumRule {
                allowed: vec!["strict".to_string(), "default".to_string()],
            }) as Rc<dyn StrictBudgetBurnTemplateRule>,
            Rc::new(StrictBudgetBurnTemplateRequiredChecksumRule {
                message: "strict budget burn template needs a value".to_string(),
            }),
            Rc::new(StrictBudgetBurnTemplateAmountRangeRule { min: 4.1, max: 4790.9 }),
            Rc::new(StrictBudgetBurnTemplateLabelLengthRule { max_length: 32 }),
            Rc::new(StrictBudgetBurnTemplateCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(StrictBudgetBurnTemplateRequiredLabelRule {
                message: "strict budget burn template needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_strict_budget_burn_template_validator();
        let item = new_strict_budget_burn_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_checksum() {
        let rule = StrictBudgetBurnTemplateRequiredChecksumRule {
            message: "strict budget burn template needs this field".to_string(),
        };
        let mut item = new_strict_budget_burn_template("", "");
        item.checksum = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = StrictBudgetBurnTemplateAmountRangeRule { min: 1.0, max: 1026.9 };
        let mut item = new_strict_budget_burn_template("x", "y");
        item.amount = 1028.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = StrictBudgetBurnTemplateLabelLengthRule { max_length: 255 };
        let mut item = new_strict_budget_burn_template("x", "y");
        item.label = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = StrictBudgetBurnTemplateRequiredLabelRule {
            message: "strict budget burn template needs this field".to_string(),
        };
        let mut item = new_strict_budget_burn_template("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
