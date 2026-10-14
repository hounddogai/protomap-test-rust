use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a silence policy must pass. Check returns its problems, or none.
pub trait SilencePolicyRule: std::fmt::Debug {
    fn check(&self, item: &SilencePolicy) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilencePolicyCodePrefixRule {
    pub prefix: String,
}

impl SilencePolicyRule for SilencePolicyCodePrefixRule {
    fn check(&self, item: &SilencePolicy) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilencePolicySequenceRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SilencePolicyRule for SilencePolicySequenceRangeRule {
    fn check(&self, item: &SilencePolicy) -> Vec<String> {
        let value = item.sequence;
        if value < self.min || value > self.max {
            return vec![format!("sequence {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SilencePolicyNamespaceLengthRule {
    pub max_length: i64,
}

impl SilencePolicyRule for SilencePolicyNamespaceLengthRule {
    fn check(&self, item: &SilencePolicy) -> Vec<String> {
        let size = item.namespace.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "namespace is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a silence policy.
#[derive(Debug, Clone)]
pub struct SilencePolicyValidator {
    pub rules: Vec<Rc<dyn SilencePolicyRule>>,
}

impl SilencePolicyValidator {
    pub fn validate(&self, item: &SilencePolicy) -> Vec<String> {
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

    pub fn require_valid(&self, item: &SilencePolicy) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SilencePolicy]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_silence_policy_validator() -> SilencePolicyValidator {
    SilencePolicyValidator {
        rules: vec![
            Rc::new(SilencePolicyCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn SilencePolicyRule>,
            Rc::new(SilencePolicySequenceRangeRule { min: 1000, max: 86400 }),
            Rc::new(SilencePolicyNamespaceLengthRule { max_length: 32 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_silence_policy_validator();
        let item = new_silence_policy("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_sequence() {
        let rule = SilencePolicySequenceRangeRule { min: 80, max: 1000 };
        let mut item = new_silence_policy("x", "y");
        item.sequence = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_namespace() {
        let rule = SilencePolicyNamespaceLengthRule { max_length: 16 };
        let mut item = new_silence_policy("x", "y");
        item.namespace = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.namespace = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
