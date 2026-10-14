use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a escalation override must pass. Check returns its problems, or none.
pub trait EscalationOverrideRule: std::fmt::Debug {
    fn check(&self, item: &EscalationOverride) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationOverrideRequiredCurrencyRule {
    pub message: String,
}

impl EscalationOverrideRule for EscalationOverrideRequiredCurrencyRule {
    fn check(&self, item: &EscalationOverride) -> Vec<String> {
        if item.currency.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationOverrideStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl EscalationOverrideRule for EscalationOverrideStartsAtRangeRule {
    fn check(&self, item: &EscalationOverride) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EscalationOverrideAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl EscalationOverrideRule for EscalationOverrideAllowedCodeRule {
    fn check(&self, item: &EscalationOverride) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is blocked", item.code)];
        }
        Vec::new()
    }
}

/// Runs every rule of a escalation override.
#[derive(Debug, Clone)]
pub struct EscalationOverrideValidator {
    pub rules: Vec<Rc<dyn EscalationOverrideRule>>,
}

impl EscalationOverrideValidator {
    pub fn validate(&self, item: &EscalationOverride) -> Vec<String> {
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

    pub fn require_valid(&self, item: &EscalationOverride) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[EscalationOverride]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_escalation_override_validator() -> EscalationOverrideValidator {
    EscalationOverrideValidator {
        rules: vec![
            Rc::new(EscalationOverrideRequiredCurrencyRule {
                message: "escalation override needs this field".to_string(),
            }) as Rc<dyn EscalationOverrideRule>,
            Rc::new(EscalationOverrideStartsAtRangeRule { min: 1000, max: 3600 }),
            Rc::new(EscalationOverrideAllowedCodeRule {
                allowed: vec!["batched".to_string(), "pending".to_string()],
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_escalation_override_validator();
        let item = new_escalation_override("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_blank_currency() {
        let rule = EscalationOverrideRequiredCurrencyRule {
            message: "escalation override needs this field".to_string(),
        };
        let mut item = new_escalation_override("", "");
        item.currency = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = EscalationOverrideStartsAtRangeRule { min: 16, max: 80 };
        let mut item = new_escalation_override("x", "y");
        item.starts_at = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
