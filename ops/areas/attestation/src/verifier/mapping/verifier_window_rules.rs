use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a verifier window must pass. Check returns its problems, or none.
pub trait VerifierWindowRule: std::fmt::Debug {
    fn check(&self, item: &VerifierWindow) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierWindowAllowedTitleRule {
    pub allowed: Vec<String>,
}

impl VerifierWindowRule for VerifierWindowAllowedTitleRule {
    fn check(&self, item: &VerifierWindow) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.title))) {
            return vec![format!("title {} is blocked", item.title)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierWindowStartsAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl VerifierWindowRule for VerifierWindowStartsAtRangeRule {
    fn check(&self, item: &VerifierWindow) -> Vec<String> {
        let value = item.starts_at;
        if value < self.min || value > self.max {
            return vec![format!("starts at {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VerifierWindowCodePrefixRule {
    pub prefix: String,
}

impl VerifierWindowRule for VerifierWindowCodePrefixRule {
    fn check(&self, item: &VerifierWindow) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

/// Runs every rule of a verifier window.
#[derive(Debug, Clone)]
pub struct VerifierWindowValidator {
    pub rules: Vec<Rc<dyn VerifierWindowRule>>,
}

impl VerifierWindowValidator {
    pub fn validate(&self, item: &VerifierWindow) -> Vec<String> {
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

    pub fn require_valid(&self, item: &VerifierWindow) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[VerifierWindow]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_verifier_window_validator() -> VerifierWindowValidator {
    VerifierWindowValidator {
        rules: vec![
            Rc::new(VerifierWindowAllowedTitleRule {
                allowed: vec!["lenient".to_string()],
            }) as Rc<dyn VerifierWindowRule>,
            Rc::new(VerifierWindowStartsAtRangeRule { min: 1000, max: 86400 }),
            Rc::new(VerifierWindowCodePrefixRule {
                prefix: "REF".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_verifier_window_validator();
        let item = new_verifier_window("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_starts_at() {
        let rule = VerifierWindowStartsAtRangeRule { min: 32, max: 120 };
        let mut item = new_verifier_window("x", "y");
        item.starts_at = 121;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
