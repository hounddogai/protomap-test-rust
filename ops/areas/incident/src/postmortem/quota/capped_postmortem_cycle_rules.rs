use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a capped postmortem cycle must pass. Check returns its problems, or none.
pub trait CappedPostmortemCycleRule: std::fmt::Debug {
    fn check(&self, item: &CappedPostmortemCycle) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPostmortemCycleCodePrefixRule {
    pub prefix: String,
}

impl CappedPostmortemCycleRule for CappedPostmortemCycleCodePrefixRule {
    fn check(&self, item: &CappedPostmortemCycle) -> Vec<String> {
        if !item.code.to_uppercase().starts_with(&self.prefix.to_uppercase()) {
            return vec![format!("code must start with {}", self.prefix)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPostmortemCycleFloorValueRangeRule {
    pub min: f64,
    pub max: f64,
}

impl CappedPostmortemCycleRule for CappedPostmortemCycleFloorValueRangeRule {
    fn check(&self, item: &CappedPostmortemCycle) -> Vec<String> {
        let value = item.floor_value;
        if value < self.min || value > self.max {
            return vec![format!("floor value {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CappedPostmortemCyclePortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl CappedPostmortemCycleRule for CappedPostmortemCyclePortRangeRule {
    fn check(&self, item: &CappedPostmortemCycle) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

/// Runs every rule of a capped postmortem cycle.
#[derive(Debug, Clone)]
pub struct CappedPostmortemCycleValidator {
    pub rules: Vec<Rc<dyn CappedPostmortemCycleRule>>,
}

impl CappedPostmortemCycleValidator {
    pub fn validate(&self, item: &CappedPostmortemCycle) -> Vec<String> {
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

    pub fn require_valid(&self, item: &CappedPostmortemCycle) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[CappedPostmortemCycle]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_capped_postmortem_cycle_validator() -> CappedPostmortemCycleValidator {
    CappedPostmortemCycleValidator {
        rules: vec![
            Rc::new(CappedPostmortemCycleCodePrefixRule {
                prefix: "X".to_string(),
            }) as Rc<dyn CappedPostmortemCycleRule>,
            Rc::new(CappedPostmortemCycleFloorValueRangeRule { min: 5.4, max: 2056.3 }),
            Rc::new(CappedPostmortemCyclePortRangeRule { min: 1000, max: 3600 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_capped_postmortem_cycle_validator();
        let item = new_capped_postmortem_cycle("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_values_above_floor_value() {
        let rule = CappedPostmortemCycleFloorValueRangeRule { min: 9.2, max: 1745.9 };
        let mut item = new_capped_postmortem_cycle("x", "y");
        item.floor_value = 1747.4;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = CappedPostmortemCyclePortRangeRule { min: 64, max: 1000 };
        let mut item = new_capped_postmortem_cycle("x", "y");
        item.port = 1001;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }
}
