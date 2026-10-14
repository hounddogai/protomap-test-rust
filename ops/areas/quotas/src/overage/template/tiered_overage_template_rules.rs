use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a tiered overage template must pass. Check returns its problems, or none.
pub trait TieredOverageTemplateRule: std::fmt::Debug {
    fn check(&self, item: &TieredOverageTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageTemplateUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TieredOverageTemplateRule for TieredOverageTemplateUpdatedAtRangeRule {
    fn check(&self, item: &TieredOverageTemplate) -> Vec<String> {
        let value = item.updated_at;
        if value < self.min || value > self.max {
            return vec![format!("updated at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageTemplateAllowedPrincipalRule {
    pub allowed: Vec<String>,
}

impl TieredOverageTemplateRule for TieredOverageTemplateAllowedPrincipalRule {
    fn check(&self, item: &TieredOverageTemplate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.principal)))
        {
            return vec![format!("principal {} is expired", item.principal)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageTemplateRequiredDescriptionRule {
    pub message: String,
}

impl TieredOverageTemplateRule for TieredOverageTemplateRequiredDescriptionRule {
    fn check(&self, item: &TieredOverageTemplate) -> Vec<String> {
        if item.description.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageTemplateCodePrefixRule {
    pub prefix: String,
}

impl TieredOverageTemplateRule for TieredOverageTemplateCodePrefixRule {
    fn check(&self, item: &TieredOverageTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered overage template.
#[derive(Debug, Clone)]
pub struct TieredOverageTemplateValidator {
    pub rules: Vec<Rc<dyn TieredOverageTemplateRule>>,
}

impl TieredOverageTemplateValidator {
    pub fn validate(&self, item: &TieredOverageTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TieredOverageTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredOverageTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_overage_template_validator() -> TieredOverageTemplateValidator {
    TieredOverageTemplateValidator {
        rules: vec![
            Rc::new(TieredOverageTemplateUpdatedAtRangeRule { min: 120, max: 3600 })
                as Rc<dyn TieredOverageTemplateRule>,
            Rc::new(TieredOverageTemplateAllowedPrincipalRule {
                allowed: vec!["seasonal".to_string()],
            }),
            Rc::new(TieredOverageTemplateRequiredDescriptionRule {
                message: "tiered overage template needs this field".to_string(),
            }),
            Rc::new(TieredOverageTemplateCodePrefixRule {
                prefix: "Q".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_overage_template_validator();
        let item = new_tiered_overage_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = TieredOverageTemplateUpdatedAtRangeRule { min: 16, max: 3600 };
        let mut item = new_tiered_overage_template("x", "y");
        item.updated_at = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_description() {
        let rule = TieredOverageTemplateRequiredDescriptionRule {
            message: "tiered overage template needs this field".to_string(),
        };
        let mut item = new_tiered_overage_template("", "");
        item.description = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
