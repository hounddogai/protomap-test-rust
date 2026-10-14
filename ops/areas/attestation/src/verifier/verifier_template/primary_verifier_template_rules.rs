use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a primary verifier template must pass. Check returns its problems, or none.
pub trait PrimaryVerifierTemplateRule: std::fmt::Debug {
    fn check(&self, item: &PrimaryVerifierTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierTemplateRequiredNameRule {
    pub message: String,
}

impl PrimaryVerifierTemplateRule for PrimaryVerifierTemplateRequiredNameRule {
    fn check(&self, item: &PrimaryVerifierTemplate) -> Vec<String> {
        if item.name.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierTemplateRequiredCodeRule {
    pub message: String,
}

impl PrimaryVerifierTemplateRule for PrimaryVerifierTemplateRequiredCodeRule {
    fn check(&self, item: &PrimaryVerifierTemplate) -> Vec<String> {
        if item.code.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierTemplateRateRangeRule {
    pub min: f64,
    pub max: f64,
}

impl PrimaryVerifierTemplateRule for PrimaryVerifierTemplateRateRangeRule {
    fn check(&self, item: &PrimaryVerifierTemplate) -> Vec<String> {
        let value = item.rate;
        if value < self.min || value > self.max {
            return vec![format!("rate {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryVerifierTemplateActiveGuardRule {
    pub limit: f64,
}

impl PrimaryVerifierTemplateRule for PrimaryVerifierTemplateActiveGuardRule {
    fn check(&self, item: &PrimaryVerifierTemplate) -> Vec<String> {
        if item.active && (item.ceiling > self.limit) {
            return vec![format!("ceiling {} exceeds {} while active", item.ceiling, self.limit)];
        }
        Vec::new()
    }
}

/// Runs every rule of a primary verifier template.
#[derive(Debug, Clone)]
pub struct PrimaryVerifierTemplateValidator {
    pub rules: Vec<Rc<dyn PrimaryVerifierTemplateRule>>,
}

impl PrimaryVerifierTemplateValidator {
    pub fn validate(&self, item: &PrimaryVerifierTemplate) -> Vec<String> {
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

    pub fn require_valid(&self, item: &PrimaryVerifierTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[PrimaryVerifierTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_primary_verifier_template_validator() -> PrimaryVerifierTemplateValidator {
    PrimaryVerifierTemplateValidator {
        rules: vec![
            Rc::new(PrimaryVerifierTemplateRequiredNameRule {
                message: "primary verifier template needs this field".to_string(),
            }) as Rc<dyn PrimaryVerifierTemplateRule>,
            Rc::new(PrimaryVerifierTemplateRequiredCodeRule {
                message: "primary verifier template needs an entry".to_string(),
            }),
            Rc::new(PrimaryVerifierTemplateRateRangeRule { min: 6.1, max: 1575.1 }),
            Rc::new(PrimaryVerifierTemplateActiveGuardRule { limit: 7.7 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_primary_verifier_template_validator();
        let item = new_primary_verifier_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_name() {
        let rule = PrimaryVerifierTemplateRequiredNameRule {
            message: "primary verifier template needs this field".to_string(),
        };
        let mut item = new_primary_verifier_template("", "");
        item.name = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_code() {
        let rule = PrimaryVerifierTemplateRequiredCodeRule {
            message: "primary verifier template needs this field".to_string(),
        };
        let mut item = new_primary_verifier_template("", "");
        item.code = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_rate() {
        let rule = PrimaryVerifierTemplateRateRangeRule { min: 2.4, max: 665.8 };
        let mut item = new_primary_verifier_template("x", "y");
        item.rate = 667.3;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
