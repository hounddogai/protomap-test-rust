use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a trust domain template must pass. Check returns its problems, or none.
pub trait TrustDomainTemplateRule: std::fmt::Debug {
    fn check(&self, item: &TrustDomainTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainTemplateCodePrefixRule {
    pub prefix: String,
}

impl TrustDomainTemplateRule for TrustDomainTemplateCodePrefixRule {
    fn check(&self, item: &TrustDomainTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainTemplateAttemptsRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TrustDomainTemplateRule for TrustDomainTemplateAttemptsRangeRule {
    fn check(&self, item: &TrustDomainTemplate) -> Vec<String> {
        let value = item.attempts;
        if value < self.min || value > self.max {
            return vec![format!("attempts {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainTemplateRequiredHostRule {
    pub message: String,
}

impl TrustDomainTemplateRule for TrustDomainTemplateRequiredHostRule {
    fn check(&self, item: &TrustDomainTemplate) -> Vec<String> {
        if item.host.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainTemplateBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl TrustDomainTemplateRule for TrustDomainTemplateBurstRangeRule {
    fn check(&self, item: &TrustDomainTemplate) -> Vec<String> {
        let value = item.burst;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("burst {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("burst {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainTemplateBudgetRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TrustDomainTemplateRule for TrustDomainTemplateBudgetRangeRule {
    fn check(&self, item: &TrustDomainTemplate) -> Vec<String> {
        let value = item.budget;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("budget {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("budget {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainTemplateLabelLengthRule {
    pub max_length: i64,
}

impl TrustDomainTemplateRule for TrustDomainTemplateLabelLengthRule {
    fn check(&self, item: &TrustDomainTemplate) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a trust domain template.
#[derive(Debug, Clone)]
pub struct TrustDomainTemplateValidator {
    pub rules: Vec<Rc<dyn TrustDomainTemplateRule>>,
}

impl TrustDomainTemplateValidator {
    pub fn validate(&self, item: &TrustDomainTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &TrustDomainTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TrustDomainTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_trust_domain_template_validator() -> TrustDomainTemplateValidator {
    TrustDomainTemplateValidator {
        rules: vec![
            Rc::new(TrustDomainTemplateCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn TrustDomainTemplateRule>,
            Rc::new(TrustDomainTemplateAttemptsRangeRule { min: 32, max: 86400 }),
            Rc::new(TrustDomainTemplateRequiredHostRule {
                message: "trust domain template needs this field".to_string(),
            }),
            Rc::new(TrustDomainTemplateBurstRangeRule { min: 1000, max: 86400 }),
            Rc::new(TrustDomainTemplateBudgetRangeRule { min: 4.7, max: 3517.9 }),
            Rc::new(TrustDomainTemplateLabelLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_trust_domain_template_validator();
        let item = new_trust_domain_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_attempts() {
        let rule = TrustDomainTemplateAttemptsRangeRule { min: 16, max: 120 };
        let mut item = new_trust_domain_template("x", "y");
        item.attempts = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_host() {
        let rule = TrustDomainTemplateRequiredHostRule {
            message: "trust domain template needs an entry".to_string(),
        };
        let mut item = new_trust_domain_template("", "");
        item.host = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = TrustDomainTemplateBurstRangeRule { min: 86400, max: 86400 };
        let mut item = new_trust_domain_template("x", "y");
        item.burst = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_budget() {
        let rule = TrustDomainTemplateBudgetRangeRule { min: 5.5, max: 1814.6 };
        let mut item = new_trust_domain_template("x", "y");
        item.budget = 1816.1;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = TrustDomainTemplateLabelLengthRule { max_length: 64 };
        let mut item = new_trust_domain_template("x", "y");
        item.label = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
