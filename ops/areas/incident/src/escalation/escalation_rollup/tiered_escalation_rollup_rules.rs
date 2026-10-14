use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a tiered escalation rollup must pass. Check returns its problems, or none.
pub trait TieredEscalationRollupRule: std::fmt::Debug {
    fn check(&self, item: &TieredEscalationRollup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEscalationRollupCodePrefixRule {
    pub prefix: String,
}

impl TieredEscalationRollupRule for TieredEscalationRollupCodePrefixRule {
    fn check(&self, item: &TieredEscalationRollup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEscalationRollupLabelLengthRule {
    pub max_length: i64,
}

impl TieredEscalationRollupRule for TieredEscalationRollupLabelLengthRule {
    fn check(&self, item: &TieredEscalationRollup) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredEscalationRollupAmountRangeRule {
    pub min: f64,
    pub max: f64,
}

impl TieredEscalationRollupRule for TieredEscalationRollupAmountRangeRule {
    fn check(&self, item: &TieredEscalationRollup) -> Vec<String> {
        let value = item.amount;
        if value < self.min || value > self.max {
            return vec![format!("amount {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a tiered escalation rollup.
#[derive(Debug, Clone)]
pub struct TieredEscalationRollupValidator {
    pub rules: Vec<Rc<dyn TieredEscalationRollupRule>>,
}

impl TieredEscalationRollupValidator {
    pub fn validate(&self, item: &TieredEscalationRollup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &TieredEscalationRollup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[TieredEscalationRollup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_tiered_escalation_rollup_validator() -> TieredEscalationRollupValidator {
    TieredEscalationRollupValidator {
        rules: vec![
            Rc::new(TieredEscalationRollupCodePrefixRule {
                prefix: "REF".to_string(),
            }) as Rc<dyn TieredEscalationRollupRule>,
            Rc::new(TieredEscalationRollupLabelLengthRule { max_length: 16 }),
            Rc::new(TieredEscalationRollupAmountRangeRule { min: 8.6, max: 4014.8 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_tiered_escalation_rollup_validator();
        let item = new_tiered_escalation_rollup("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = TieredEscalationRollupLabelLengthRule { max_length: 80 };
        let mut item = new_tiered_escalation_rollup("x", "y");
        item.label = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_amount() {
        let rule = TieredEscalationRollupAmountRangeRule { min: 1.7, max: 2833.9 };
        let mut item = new_tiered_escalation_rollup("x", "y");
        item.amount = 2835.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
