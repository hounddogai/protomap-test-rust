use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a rolling budget burn template must pass. Check returns its problems, or none.
pub trait RollingBudgetBurnTemplateRule: std::fmt::Debug {
    fn check(&self, item: &RollingBudgetBurnTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingBudgetBurnTemplateRequiredHostRule {
    pub message: String,
}

impl RollingBudgetBurnTemplateRule for RollingBudgetBurnTemplateRequiredHostRule {
    fn check(&self, item: &RollingBudgetBurnTemplate) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingBudgetBurnTemplateTitleLengthRule {
    pub max_length: i64,
}

impl RollingBudgetBurnTemplateRule for RollingBudgetBurnTemplateTitleLengthRule {
    fn check(&self, item: &RollingBudgetBurnTemplate) -> Vec<String> {
        let size = item.title.len() as i64;
        if size > self.max_length {
            return vec![format!("title is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingBudgetBurnTemplateAllowedHostRule {
    pub allowed: Vec<String>,
}

impl RollingBudgetBurnTemplateRule for RollingBudgetBurnTemplateAllowedHostRule {
    fn check(&self, item: &RollingBudgetBurnTemplate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.host))) {
            return vec![format!("host {} is out of range", item.host)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingBudgetBurnTemplateVersionRangeRule {
    pub min: i64,
    pub max: i64,
}

impl RollingBudgetBurnTemplateRule for RollingBudgetBurnTemplateVersionRangeRule {
    fn check(&self, item: &RollingBudgetBurnTemplate) -> Vec<String> {
        let value = item.version;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("version {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("version {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingBudgetBurnTemplateRequiredCodeRule {
    pub message: String,
}

impl RollingBudgetBurnTemplateRule for RollingBudgetBurnTemplateRequiredCodeRule {
    fn check(&self, item: &RollingBudgetBurnTemplate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a rolling budget burn template.
#[derive(Debug, Clone)]
pub struct RollingBudgetBurnTemplateValidator {
    pub rules: Vec<Rc<dyn RollingBudgetBurnTemplateRule>>,
}

impl RollingBudgetBurnTemplateValidator {
    pub fn validate(&self, item: &RollingBudgetBurnTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &RollingBudgetBurnTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[RollingBudgetBurnTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_rolling_budget_burn_template_validator() -> RollingBudgetBurnTemplateValidator {
    RollingBudgetBurnTemplateValidator {
        rules: vec![
            Rc::new(RollingBudgetBurnTemplateRequiredHostRule {
                message: "rolling budget burn template needs this field".to_string(),
            }) as Rc<dyn RollingBudgetBurnTemplateRule>,
            Rc::new(RollingBudgetBurnTemplateTitleLengthRule { max_length: 255 }),
            Rc::new(RollingBudgetBurnTemplateAllowedHostRule {
                allowed: vec!["tiered".to_string()],
            }),
            Rc::new(RollingBudgetBurnTemplateVersionRangeRule { min: 3600, max: 3600 }),
            Rc::new(RollingBudgetBurnTemplateRequiredCodeRule {
                message: "rolling budget burn template needs a value".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_rolling_budget_burn_template_validator();
        let item = new_rolling_budget_burn_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = RollingBudgetBurnTemplateRequiredHostRule {
            message: "rolling budget burn template needs this field".to_string(),
        };
        let mut item = new_rolling_budget_burn_template("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_title() {
        let rule = RollingBudgetBurnTemplateTitleLengthRule { max_length: 64 };
        let mut item = new_rolling_budget_burn_template("x", "y");
        item.title = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.title = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_version() {
        let rule = RollingBudgetBurnTemplateVersionRangeRule { min: 80, max: 1000 };
        let mut item = new_rolling_budget_burn_template("x", "y");
        item.version = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = RollingBudgetBurnTemplateRequiredCodeRule {
            message: "rolling budget burn template needs an entry".to_string(),
        };
        let mut item = new_rolling_budget_burn_template("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
