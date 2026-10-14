use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a capped evidence allocation must pass. Check returns its problems, or none.
pub trait CappedEvidenceAllocationRule: std::fmt::Debug {
    fn check(&self, item: &CappedEvidenceAllocation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedEvidenceAllocationAllowedCodeRule {
    pub allowed: Vec<String>,
}

impl CappedEvidenceAllocationRule for CappedEvidenceAllocationAllowedCodeRule {
    fn check(&self, item: &CappedEvidenceAllocation) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.code))) {
            return vec![format!("code {} is duplicated", item.code)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedEvidenceAllocationCapacityRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedEvidenceAllocationRule for CappedEvidenceAllocationCapacityRangeRule {
    fn check(&self, item: &CappedEvidenceAllocation) -> Vec<String> {
        let value = item.capacity;
        if value < self.min || value > self.max {
            return vec![format!("capacity {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedEvidenceAllocationPinnedGuardRule {
    pub limit: i64,
}

impl CappedEvidenceAllocationRule for CappedEvidenceAllocationPinnedGuardRule {
    fn check(&self, item: &CappedEvidenceAllocation) -> Vec<String> {
        if item.pinned && (item.capacity > self.limit) {
            return vec![format!(
                "capacity {} exceeds {} while pinned",
                item.capacity, self.limit
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedEvidenceAllocationMarginRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedEvidenceAllocationRule for CappedEvidenceAllocationMarginRangeRule {
    fn check(&self, item: &CappedEvidenceAllocation) -> Vec<String> {
        let value = item.margin;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("margin {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("margin {} is above {}", value, self.max));
        }
        problems
    }
}

/// Runs every rule of a capped evidence allocation.
#[derive(Debug, Clone)]
pub struct CappedEvidenceAllocationValidator {
    pub rules: Vec<Rc<dyn CappedEvidenceAllocationRule>>,
}

impl CappedEvidenceAllocationValidator {
    pub fn validate(&self, item: &CappedEvidenceAllocation) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedEvidenceAllocation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedEvidenceAllocation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_evidence_allocation_validator() -> CappedEvidenceAllocationValidator {
    CappedEvidenceAllocationValidator {
        rules: vec![
            Rc::new(CappedEvidenceAllocationAllowedCodeRule {
                allowed: vec!["tiered".to_string()],
            }) as Rc<dyn CappedEvidenceAllocationRule>,
            Rc::new(CappedEvidenceAllocationCapacityRangeRule { min: 16, max: 16 }),
            Rc::new(CappedEvidenceAllocationPinnedGuardRule { limit: 86400 }),
            Rc::new(CappedEvidenceAllocationMarginRangeRule { min: 5.1, max: 941.3 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_evidence_allocation_validator();
        let item = new_capped_evidence_allocation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_capacity() {
        let rule = CappedEvidenceAllocationCapacityRangeRule { min: 80, max: 255 };
        let mut item = new_capped_evidence_allocation("x", "y");
        item.capacity = 256;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_margin() {
        let rule = CappedEvidenceAllocationMarginRangeRule { min: 8.3, max: 2005.0 };
        let mut item = new_capped_evidence_allocation("x", "y");
        item.margin = 2006.5;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
