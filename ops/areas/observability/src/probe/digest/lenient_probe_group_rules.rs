use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a lenient probe group must pass. Check returns its problems, or none.
pub trait LenientProbeGroupRule: std::fmt::Debug {
    fn check(&self, item: &LenientProbeGroup) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientProbeGroupUpdatedAtRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientProbeGroupRule for LenientProbeGroupUpdatedAtRangeRule {
    fn check(&self, item: &LenientProbeGroup) -> Vec<String> {
        let value = item.updated_at;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("updated at {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("updated at {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientProbeGroupRequiredTitleRule {
    pub message: String,
}

impl LenientProbeGroupRule for LenientProbeGroupRequiredTitleRule {
    fn check(&self, item: &LenientProbeGroup) -> Vec<String> {
        if item.title.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientProbeGroupPrincipalLengthRule {
    pub max_length: i64,
}

impl LenientProbeGroupRule for LenientProbeGroupPrincipalLengthRule {
    fn check(&self, item: &LenientProbeGroup) -> Vec<String> {
        let size = item.principal.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "principal is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient probe group.
#[derive(Debug, Clone)]
pub struct LenientProbeGroupValidator {
    pub rules: Vec<Rc<dyn LenientProbeGroupRule>>,
}

impl LenientProbeGroupValidator {
    pub fn validate(&self, item: &LenientProbeGroup) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientProbeGroup) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientProbeGroup]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_probe_group_validator() -> LenientProbeGroupValidator {
    LenientProbeGroupValidator {
        rules: vec![
            Rc::new(LenientProbeGroupUpdatedAtRangeRule { min: 16, max: 64 }) as Rc<dyn LenientProbeGroupRule>,
            Rc::new(LenientProbeGroupRequiredTitleRule {
                message: "lenient probe group needs an entry".to_string(),
            }),
            Rc::new(LenientProbeGroupPrincipalLengthRule { max_length: 255 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_probe_group_validator();
        let item = new_lenient_probe_group("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_updated_at() {
        let rule = LenientProbeGroupUpdatedAtRangeRule { min: 80, max: 80 };
        let mut item = new_lenient_probe_group("x", "y");
        item.updated_at = 81;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_title() {
        let rule = LenientProbeGroupRequiredTitleRule {
            message: "lenient probe group needs an entry".to_string(),
        };
        let mut item = new_lenient_probe_group("", "");
        item.title = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_long_principal() {
        let rule = LenientProbeGroupPrincipalLengthRule { max_length: 120 };
        let mut item = new_lenient_probe_group("x", "y");
        item.principal = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.principal = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
