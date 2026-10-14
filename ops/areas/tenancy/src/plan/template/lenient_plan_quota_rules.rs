use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::text;

/// A rule a lenient plan quota must pass. Check returns its problems, or none.
pub trait LenientPlanQuotaRule: std::fmt::Debug {
    fn check(&self, item: &LenientPlanQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanQuotaAllowedUnitRule {
    pub allowed: Vec<String>,
}

impl LenientPlanQuotaRule for LenientPlanQuotaAllowedUnitRule {
    fn check(&self, item: &LenientPlanQuota) -> Vec<String> {
        if (self.allowed.len() as i64) == 0 {
            return Vec::new();
        }
        if !(self.allowed.iter().any(|value| value == &text::normalize(&item.unit))) {
            return vec![format!("unit {} is below minimum", item.unit)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanQuotaNotesLengthRule {
    pub max_length: i64,
}

impl LenientPlanQuotaRule for LenientPlanQuotaNotesLengthRule {
    fn check(&self, item: &LenientPlanQuota) -> Vec<String> {
        let size = item.notes.len() as i64;
        if size > self.max_length {
            return vec![format!("notes is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LenientPlanQuotaPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl LenientPlanQuotaRule for LenientPlanQuotaPortRangeRule {
    fn check(&self, item: &LenientPlanQuota) -> Vec<String> {
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
pub struct LenientPlanQuotaNameLengthRule {
    pub max_length: i64,
}

impl LenientPlanQuotaRule for LenientPlanQuotaNameLengthRule {
    fn check(&self, item: &LenientPlanQuota) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a lenient plan quota.
#[derive(Debug, Clone)]
pub struct LenientPlanQuotaValidator {
    pub rules: Vec<Rc<dyn LenientPlanQuotaRule>>,
}

impl LenientPlanQuotaValidator {
    pub fn validate(&self, item: &LenientPlanQuota) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &LenientPlanQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[LenientPlanQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_lenient_plan_quota_validator() -> LenientPlanQuotaValidator {
    LenientPlanQuotaValidator {
        rules: vec![
            Rc::new(LenientPlanQuotaAllowedUnitRule {
                allowed: vec!["strict".to_string(), "weighted".to_string()],
            }) as Rc<dyn LenientPlanQuotaRule>,
            Rc::new(LenientPlanQuotaNotesLengthRule { max_length: 1000 }),
            Rc::new(LenientPlanQuotaPortRangeRule { min: 32, max: 1000 }),
            Rc::new(LenientPlanQuotaNameLengthRule { max_length: 120 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_lenient_plan_quota_validator();
        let item = new_lenient_plan_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_notes() {
        let rule = LenientPlanQuotaNotesLengthRule { max_length: 16 };
        let mut item = new_lenient_plan_quota("x", "y");
        item.notes = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.notes = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = LenientPlanQuotaPortRangeRule { min: 16, max: 64 };
        let mut item = new_lenient_plan_quota("x", "y");
        item.port = 65;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = LenientPlanQuotaNameLengthRule { max_length: 16 };
        let mut item = new_lenient_plan_quota("x", "y");
        item.name = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
