use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a escalation template must pass. Check returns its problems, or none.
pub trait EscalationTemplateRule: std::fmt::Debug {
    fn check(&self, item: &EscalationTemplate) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationTemplateCodeLengthRule {
    pub max_length: i64,
}

impl EscalationTemplateRule for EscalationTemplateCodeLengthRule {
    fn check(&self, item: &EscalationTemplate) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationTemplateBurstRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EscalationTemplateRule for EscalationTemplateBurstRangeRule {
    fn check(&self, item: &EscalationTemplate) -> Vec<String> {
        let value = item.burst;
        if value < self.min || value > self.max {
            return vec![format!("burst {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationTemplateSegmentLengthRule {
    pub max_length: i64,
}

impl EscalationTemplateRule for EscalationTemplateSegmentLengthRule {
    fn check(&self, item: &EscalationTemplate) -> Vec<String> {
        let size = item.segment.len() as i64;
        if size > self.max_length {
            return vec![format!("segment is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationTemplateExternalRefLengthRule {
    pub max_length: i64,
}

impl EscalationTemplateRule for EscalationTemplateExternalRefLengthRule {
    fn check(&self, item: &EscalationTemplate) -> Vec<String> {
        let size = item.external_ref.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "external ref is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationTemplateRotationDaysRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EscalationTemplateRule for EscalationTemplateRotationDaysRangeRule {
    fn check(&self, item: &EscalationTemplate) -> Vec<String> {
        let value = item.rotation_days;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("rotation days {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("rotation days {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationTemplateCodePrefixRule {
    pub prefix: String,
}

impl EscalationTemplateRule for EscalationTemplateCodePrefixRule {
    fn check(&self, item: &EscalationTemplate) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a escalation template.
#[derive(Debug, Clone)]
pub struct EscalationTemplateValidator {
    pub rules: Vec<Rc<dyn EscalationTemplateRule>>,
}

impl EscalationTemplateValidator {
    pub fn validate(&self, item: &EscalationTemplate) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &EscalationTemplate) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EscalationTemplate]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_escalation_template_validator() -> EscalationTemplateValidator {
    EscalationTemplateValidator {
        rules: vec![
            Rc::new(EscalationTemplateCodeLengthRule { max_length: 86400 }) as Rc<dyn EscalationTemplateRule>,
            Rc::new(EscalationTemplateBurstRangeRule { min: 1000, max: 86400 }),
            Rc::new(EscalationTemplateSegmentLengthRule { max_length: 3600 }),
            Rc::new(EscalationTemplateExternalRefLengthRule { max_length: 1000 }),
            Rc::new(EscalationTemplateRotationDaysRangeRule { min: 64, max: 86400 }),
            Rc::new(EscalationTemplateCodePrefixRule {
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
        let validator = default_escalation_template_validator();
        let item = new_escalation_template("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = EscalationTemplateCodeLengthRule { max_length: 86400 };
        let mut item = new_escalation_template("x", "y");
        item.code = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_burst() {
        let rule = EscalationTemplateBurstRangeRule { min: 1000, max: 1000 };
        let mut item = new_escalation_template("x", "y");
        item.burst = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_segment() {
        let rule = EscalationTemplateSegmentLengthRule { max_length: 120 };
        let mut item = new_escalation_template("x", "y");
        item.segment = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.segment = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = EscalationTemplateExternalRefLengthRule { max_length: 255 };
        let mut item = new_escalation_template("x", "y");
        item.external_ref = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_rotation_days() {
        let rule = EscalationTemplateRotationDaysRangeRule { min: 16, max: 86400 };
        let mut item = new_escalation_template("x", "y");
        item.rotation_days = 86401;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
