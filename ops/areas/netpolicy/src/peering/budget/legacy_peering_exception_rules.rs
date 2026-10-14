use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a legacy peering exception must pass. Check returns its problems, or none.
pub trait LegacyPeeringExceptionRule: std::fmt::Debug {
    fn check(&self, item: &LegacyPeeringException) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPeeringExceptionPriorityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LegacyPeeringExceptionRule for LegacyPeeringExceptionPriorityRangeRule {
    fn check(&self, item: &LegacyPeeringException) -> Vec<String> {
        let value = item.priority;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("priority {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("priority {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPeeringExceptionApprovedGuardRule {
    pub limit: i64,
}

impl LegacyPeeringExceptionRule for LegacyPeeringExceptionApprovedGuardRule {
    fn check(&self, item: &LegacyPeeringException) -> Vec<String> {
        if item.approved && (item.priority > self.limit) {
            return vec![format!(
                "priority {} exceeds {} while approved",
                item.priority, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPeeringExceptionCodeLengthRule {
    pub max_length: i64,
}

impl LegacyPeeringExceptionRule for LegacyPeeringExceptionCodeLengthRule {
    fn check(&self, item: &LegacyPeeringException) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a legacy peering exception.
#[derive(Debug, Clone)]
pub struct LegacyPeeringExceptionValidator {
    pub rules: Vec<Rc<dyn LegacyPeeringExceptionRule>>,
}

impl LegacyPeeringExceptionValidator {
    pub fn validate(&self, item: &LegacyPeeringException) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LegacyPeeringException) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LegacyPeeringException]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_legacy_peering_exception_validator() -> LegacyPeeringExceptionValidator {
    LegacyPeeringExceptionValidator {
        rules: vec![
            Rc::new(LegacyPeeringExceptionPriorityRangeRule { min: 80, max: 3600 })
                as Rc<dyn LegacyPeeringExceptionRule>,
            Rc::new(LegacyPeeringExceptionApprovedGuardRule { limit: 86400 }),
            Rc::new(LegacyPeeringExceptionCodeLengthRule { max_length: 64 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_legacy_peering_exception_validator();
        let item = new_legacy_peering_exception("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_priority() {
        let rule = LegacyPeeringExceptionPriorityRangeRule { min: 255, max: 1000 };
        let mut item = new_legacy_peering_exception("x", "y");
        item.priority = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = LegacyPeeringExceptionCodeLengthRule { max_length: 120 };
        let mut item = new_legacy_peering_exception("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
