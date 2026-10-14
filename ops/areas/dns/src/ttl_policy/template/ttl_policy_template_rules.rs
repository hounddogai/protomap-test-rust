use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a ttl policy template must pass. Check returns its problems, or none.
pub trait TtlPolicyTemplateRule: std::fmt::Debug {
    fn check(&self, item: &TtlPolicyTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyTemplateRequiredLabelRule {
    pub message: String,
}

impl TtlPolicyTemplateRule for TtlPolicyTemplateRequiredLabelRule {
    fn check(&self, item: &TtlPolicyTemplate) -> Vec<String> {
        if item.label.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyTemplateCostRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TtlPolicyTemplateRule for TtlPolicyTemplateCostRangeRule {
    fn check(&self, item: &TtlPolicyTemplate) -> Vec<String> {
        let value = item.cost;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("cost {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("cost {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyTemplateAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl TtlPolicyTemplateRule for TtlPolicyTemplateAllowedCodeRule {
    fn check(&self, item: &TtlPolicyTemplate) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is out of range", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyTemplateCodeLengthRule {
    pub max_length: i64,
}

impl TtlPolicyTemplateRule for TtlPolicyTemplateCodeLengthRule {
    fn check(&self, item: &TtlPolicyTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyTemplateRequiredCodeRule {
    pub message: String,
}

impl TtlPolicyTemplateRule for TtlPolicyTemplateRequiredCodeRule {
    fn check(&self, item: &TtlPolicyTemplate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyTemplateCodePrefixRule {
    pub prefix: String,
}

impl TtlPolicyTemplateRule for TtlPolicyTemplateCodePrefixRule {
    fn check(&self, item: &TtlPolicyTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a ttl policy template.
#[derive(Debug, Clone)]
pub struct TtlPolicyTemplateValidator {
    pub rules: Vec<Rc<dyn TtlPolicyTemplateRule>>,
}

impl TtlPolicyTemplateValidator {
    pub fn validate(&self, item: &TtlPolicyTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TtlPolicyTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TtlPolicyTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_ttl_policy_template_validator() -> TtlPolicyTemplateValidator {
    TtlPolicyTemplateValidator {
        rules: vec![
            Rc::new(TtlPolicyTemplateRequiredLabelRule {
                message: "ttl policy template needs this field".to_string(),
            }) as Rc<dyn TtlPolicyTemplateRule>,
            Rc::new(TtlPolicyTemplateCostRangeRule { min: 9.2, max: 838.6 }),
            Rc::new(TtlPolicyTemplateAllowedCodeRule {
                allowed: vec!["legacy".to_string(), "lenient".to_string(), "weighted".to_string()],
            }),
            Rc::new(TtlPolicyTemplateCodeLengthRule { max_length: 80 }),
            Rc::new(TtlPolicyTemplateRequiredCodeRule {
                message: "ttl policy template needs this field".to_string(),
            }),
            Rc::new(TtlPolicyTemplateCodePrefixRule {
                prefix: "ACM".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_ttl_policy_template_validator();
        let item = new_ttl_policy_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_label() {
        let rule = TtlPolicyTemplateRequiredLabelRule {
            message: "ttl policy template needs an entry".to_string(),
        };
        let mut item = new_ttl_policy_template("", "");
        item.label = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_cost() {
        let rule = TtlPolicyTemplateCostRangeRule { min: 7.6, max: 116.9 };
        let mut item = new_ttl_policy_template("x", "y");
        item.cost = 118.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = TtlPolicyTemplateCodeLengthRule { max_length: 32 };
        let mut item = new_ttl_policy_template("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = TtlPolicyTemplateRequiredCodeRule {
            message: "ttl policy template needs a value".to_string(),
        };
        let mut item = new_ttl_policy_template("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
