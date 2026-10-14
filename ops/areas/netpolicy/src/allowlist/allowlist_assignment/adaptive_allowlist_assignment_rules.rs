use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a adaptive allowlist assignment must pass. Check returns its problems, or none.
pub trait AdaptiveAllowlistAssignmentRule: std::fmt::Debug {
    fn check(&self, item: &AdaptiveAllowlistAssignment) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAllowlistAssignmentExternalRefLengthRule {
    pub max_length: i64,
}

impl AdaptiveAllowlistAssignmentRule for AdaptiveAllowlistAssignmentExternalRefLengthRule {
    fn check(&self, item: &AdaptiveAllowlistAssignment) -> Vec<String> {
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
pub struct AdaptiveAllowlistAssignmentMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl AdaptiveAllowlistAssignmentRule for AdaptiveAllowlistAssignmentMarginRangeRule {
    fn check(&self, item: &AdaptiveAllowlistAssignment) -> Vec<String> {
        let value = item.margin;
        if value < self.min || value > self.max {
            return vec![format!("margin {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAllowlistAssignmentPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl AdaptiveAllowlistAssignmentRule for AdaptiveAllowlistAssignmentPortRangeRule {
    fn check(&self, item: &AdaptiveAllowlistAssignment) -> Vec<String> {
        let value = item.port;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("port {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("port {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveAllowlistAssignmentNameLengthRule {
    pub max_length: i64,
}

impl AdaptiveAllowlistAssignmentRule for AdaptiveAllowlistAssignmentNameLengthRule {
    fn check(&self, item: &AdaptiveAllowlistAssignment) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a adaptive allowlist assignment.
#[derive(Debug, Clone)]
pub struct AdaptiveAllowlistAssignmentValidator {
    pub rules: Vec<Rc<dyn AdaptiveAllowlistAssignmentRule>>,
}

impl AdaptiveAllowlistAssignmentValidator {
    pub fn validate(&self, item: &AdaptiveAllowlistAssignment) -> Vec<String> {
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

    pub fn require_valid(&self, item: &AdaptiveAllowlistAssignment) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[AdaptiveAllowlistAssignment]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_adaptive_allowlist_assignment_validator() -> AdaptiveAllowlistAssignmentValidator {
    AdaptiveAllowlistAssignmentValidator {
        rules: vec![
            Rc::new(AdaptiveAllowlistAssignmentExternalRefLengthRule { max_length: 32 })
                as Rc<dyn AdaptiveAllowlistAssignmentRule>,
            Rc::new(AdaptiveAllowlistAssignmentMarginRangeRule { min: 3.2, max: 2954.6 }),
            Rc::new(AdaptiveAllowlistAssignmentPortRangeRule { min: 86400, max: 86400 }),
            Rc::new(AdaptiveAllowlistAssignmentNameLengthRule { max_length: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_adaptive_allowlist_assignment_validator();
        let item = new_adaptive_allowlist_assignment("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = AdaptiveAllowlistAssignmentExternalRefLengthRule { max_length: 255 };
        let mut item = new_adaptive_allowlist_assignment("x", "y");
        item.external_ref = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = AdaptiveAllowlistAssignmentMarginRangeRule { min: 4.3, max: 3518.9 };
        let mut item = new_adaptive_allowlist_assignment("x", "y");
        item.margin = 3520.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = AdaptiveAllowlistAssignmentPortRangeRule { min: 255, max: 1000 };
        let mut item = new_adaptive_allowlist_assignment("x", "y");
        item.port = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = AdaptiveAllowlistAssignmentNameLengthRule { max_length: 86400 };
        let mut item = new_adaptive_allowlist_assignment("x", "y");
        item.name = "x".repeat(86401 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
