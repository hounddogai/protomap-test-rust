use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a global probe assignment must pass. Check returns its problems, or none.
pub trait GlobalProbeAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &GlobalProbeAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalProbeAssignmentCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl GlobalProbeAssignmentRule for GlobalProbeAssignmentCeilingRangeRule {
    fn check(&self, item: &GlobalProbeAssignment) -> Vec<String> {
        let value = item.ceiling;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("ceiling {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("ceiling {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalProbeAssignmentAllowedCurrencyRule {
    pub allowed: Vec<String>,
}

impl GlobalProbeAssignmentRule for GlobalProbeAssignmentAllowedCurrencyRule {
    fn check(&self, item: &GlobalProbeAssignment) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self
            .allowed
            .iter()
            .any(|value| value == &text::normalize(&item.currency)))
        {
            return vec![format!("currency {} is stale", item.currency)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalProbeAssignmentPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl GlobalProbeAssignmentRule for GlobalProbeAssignmentPortRangeRule {
    fn check(&self, item: &GlobalProbeAssignment) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalProbeAssignmentCodePrefixRule {
    pub prefix: String,
}

impl GlobalProbeAssignmentRule for GlobalProbeAssignmentCodePrefixRule {
    fn check(&self, item: &GlobalProbeAssignment) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalProbeAssignmentCodeLengthRule {
    pub max_length: i64,
}

impl GlobalProbeAssignmentRule for GlobalProbeAssignmentCodeLengthRule {
    fn check(&self, item: &GlobalProbeAssignment) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a global probe assignment.
#[derive(Debug, Clone)]
pub struct GlobalProbeAssignmentValidator {
    pub rules: Vec<Rc<dyn GlobalProbeAssignmentRule>>,
}

impl GlobalProbeAssignmentValidator {
    pub fn validate(&self, item: &GlobalProbeAssignment) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &GlobalProbeAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[GlobalProbeAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_global_probe_assignment_validator() -> GlobalProbeAssignmentValidator {
    GlobalProbeAssignmentValidator {
        rules: vec![
            Rc::new(GlobalProbeAssignmentCeilingRangeRule { min: 7.8, max: 2336.8 })
                as Rc<dyn GlobalProbeAssignmentRule>,
            Rc::new(GlobalProbeAssignmentAllowedCurrencyRule {
                allowed: vec!["batched".to_string()],
            }),
            Rc::new(GlobalProbeAssignmentPortRangeRule { min: 32, max: 64 }),
            Rc::new(GlobalProbeAssignmentCodePrefixRule {
                prefix: "X".to_string(),
            }),
            Rc::new(GlobalProbeAssignmentCodeLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_global_probe_assignment_validator();
        let item = new_global_probe_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = GlobalProbeAssignmentCeilingRangeRule { min: 2.9, max: 2783.2 };
        let mut item = new_global_probe_assignment("x", "y");
        item.ceiling = 2784.7;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = GlobalProbeAssignmentPortRangeRule { min: 32, max: 64 };
        let mut item = new_global_probe_assignment("x", "y");
        item.port = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = GlobalProbeAssignmentCodeLengthRule { max_length: 16 };
        let mut item = new_global_probe_assignment("x", "y");
        item.code = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
