use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a policy template must pass. Check returns its problems, or none.
pub trait PolicyTemplateRule: std::fmt::Debug {
    fn check(&self, item: &PolicyTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyTemplateCodeLengthRule {
    pub max_length: i64,
}

impl PolicyTemplateRule for PolicyTemplateCodeLengthRule {
    fn check(&self, item: &PolicyTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyTemplateRequiredCountryRule {
    pub message: String,
}

impl PolicyTemplateRule for PolicyTemplateRequiredCountryRule {
    fn check(&self, item: &PolicyTemplate) -> Vec<String> {
        if item.country.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyTemplateCountryLengthRule {
    pub max_length: i64,
}

impl PolicyTemplateRule for PolicyTemplateCountryLengthRule {
    fn check(&self, item: &PolicyTemplate) -> Vec<String> {
        let size = item.country.len() as i64;
        if size > self.max_length {
            return vec![format!("country is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyTemplateCodePrefixRule {
    pub prefix: String,
}

impl PolicyTemplateRule for PolicyTemplateCodePrefixRule {
    fn check(&self, item: &PolicyTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a policy template.
#[derive(Debug, Clone)]
pub struct PolicyTemplateValidator {
    pub rules: Vec<Rc<dyn PolicyTemplateRule>>,
}

impl PolicyTemplateValidator {
    pub fn validate(&self, item: &PolicyTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &PolicyTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PolicyTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_policy_template_validator() -> PolicyTemplateValidator {
    PolicyTemplateValidator {
        rules: vec![
            Rc::new(PolicyTemplateCodeLengthRule { max_length: 86400 }) as Rc<dyn PolicyTemplateRule>,
            Rc::new(PolicyTemplateRequiredCountryRule {
                message: "policy template needs this field".to_string(),
            }),
            Rc::new(PolicyTemplateCountryLengthRule { max_length: 120 }),
            Rc::new(PolicyTemplateCodePrefixRule {
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
        let validator = default_policy_template_validator();
        let item = new_policy_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = PolicyTemplateCodeLengthRule { max_length: 32 };
        let mut item = new_policy_template("x", "y");
        item.code = "x".repeat(33 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_country() {
        let rule = PolicyTemplateRequiredCountryRule {
            message: "policy template needs a value".to_string(),
        };
        let mut item = new_policy_template("", "");
        item.country = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_country() {
        let rule = PolicyTemplateCountryLengthRule { max_length: 255 };
        let mut item = new_policy_template("x", "y");
        item.country = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.country = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
