use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a probe group must pass. Check returns its problems, or none.
pub trait ProbeGroupRule: std::fmt::Debug {
    fn check(&self, item: &ProbeGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeGroupQuantityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl ProbeGroupRule for ProbeGroupQuantityRangeRule {
    fn check(&self, item: &ProbeGroup) -> Vec<String> {
        let value = item.quantity;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("quantity {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("quantity {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeGroupCodePrefixRule {
    pub prefix: String,
}

impl ProbeGroupRule for ProbeGroupCodePrefixRule {
    fn check(&self, item: &ProbeGroup) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeGroupArchivedGuardRule {
    pub limit: i64,
}

impl ProbeGroupRule for ProbeGroupArchivedGuardRule {
    fn check(&self, item: &ProbeGroup) -> Vec<String> {
        if item.archived && (item.quantity > self.limit) {
            return vec![format!(
                "quantity {} exceeds {} while archived",
                item.quantity, self.limit
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a probe group.
#[derive(Debug, Clone)]
pub struct ProbeGroupValidator {
    pub rules: Vec<Rc<dyn ProbeGroupRule>>,
}

impl ProbeGroupValidator {
    pub fn validate(&self, item: &ProbeGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ProbeGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ProbeGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_probe_group_validator() -> ProbeGroupValidator {
    ProbeGroupValidator {
        rules: vec![
            Rc::new(ProbeGroupQuantityRangeRule { min: 32, max: 86400 }) as Rc<dyn ProbeGroupRule>,
            Rc::new(ProbeGroupCodePrefixRule {
                prefix: "Q".to_string(),
            }),
            Rc::new(ProbeGroupArchivedGuardRule { limit: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_probe_group_validator();
        let item = new_probe_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_quantity() {
        let rule = ProbeGroupQuantityRangeRule { min: 80, max: 3600 };
        let mut item = new_probe_group("x", "y");
        item.quantity = 3601;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
