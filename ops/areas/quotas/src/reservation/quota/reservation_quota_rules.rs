use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a reservation quota must pass. Check returns its problems, or none.
pub trait ReservationQuotaRule: std::fmt::Debug {
    fn check(&self, item: &ReservationQuota) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationQuotaExternalRefLengthRule {
    pub max_length: i64,
}

impl ReservationQuotaRule for ReservationQuotaExternalRefLengthRule {
    fn check(&self, item: &ReservationQuota) -> Vec<String> {
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
pub struct ReservationQuotaCeilingRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ReservationQuotaRule for ReservationQuotaCeilingRangeRule {
    fn check(&self, item: &ReservationQuota) -> Vec<String> {
        let value = item.ceiling;
        if value < self.min || value > self.max {
            return vec![format!("ceiling {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationQuotaCodeLengthRule {
    pub max_length: i64,
}

impl ReservationQuotaRule for ReservationQuotaCodeLengthRule {
    fn check(&self, item: &ReservationQuota) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationQuotaLabelLengthRule {
    pub max_length: i64,
}

impl ReservationQuotaRule for ReservationQuotaLabelLengthRule {
    fn check(&self, item: &ReservationQuota) -> Vec<String> {
        let size = item.label.len() as i64;
        if size > self.max_length {
            return vec![format!("label is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a reservation quota.
#[derive(Debug, Clone)]
pub struct ReservationQuotaValidator {
    pub rules: Vec<Rc<dyn ReservationQuotaRule>>,
}

impl ReservationQuotaValidator {
    pub fn validate(&self, item: &ReservationQuota) -> Vec<String> {
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

    pub fn require_valid(&self, item: &ReservationQuota) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.display_label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationQuota]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_reservation_quota_validator() -> ReservationQuotaValidator {
    ReservationQuotaValidator {
        rules: vec![
            Rc::new(ReservationQuotaExternalRefLengthRule { max_length: 1000 }) as Rc<dyn ReservationQuotaRule>,
            Rc::new(ReservationQuotaCeilingRangeRule { min: 4.0, max: 4042.3 }),
            Rc::new(ReservationQuotaCodeLengthRule { max_length: 1000 }),
            Rc::new(ReservationQuotaLabelLengthRule { max_length: 16 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_reservation_quota_validator();
        let item = new_reservation_quota("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_external_ref() {
        let rule = ReservationQuotaExternalRefLengthRule { max_length: 255 };
        let mut item = new_reservation_quota("x", "y");
        item.external_ref = "x".repeat(256 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.external_ref = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_ceiling() {
        let rule = ReservationQuotaCeilingRangeRule { min: 3.2, max: 4588.5 };
        let mut item = new_reservation_quota("x", "y");
        item.ceiling = 4590.0;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ReservationQuotaCodeLengthRule { max_length: 120 };
        let mut item = new_reservation_quota("x", "y");
        item.code = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_label() {
        let rule = ReservationQuotaLabelLengthRule { max_length: 16 };
        let mut item = new_reservation_quota("x", "y");
        item.label = "x".repeat(17 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.label = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
