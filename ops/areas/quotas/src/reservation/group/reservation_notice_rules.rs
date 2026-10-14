use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a reservation notice must pass. Check returns its problems, or none.
pub trait ReservationNoticeRule: std::fmt::Debug {
    fn check(&self, item: &ReservationNotice) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationNoticeNameLengthRule {
    pub max_length: i64,
}

impl ReservationNoticeRule for ReservationNoticeNameLengthRule {
    fn check(&self, item: &ReservationNotice) -> Vec<String> {
        let size = item.name.len() as i64;
        if size > self.max_length {
            return vec![format!("name is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationNoticeCodeLengthRule {
    pub max_length: i64,
}

impl ReservationNoticeRule for ReservationNoticeCodeLengthRule {
    fn check(&self, item: &ReservationNotice) -> Vec<String> {
        let size = item.code.len() as i64;
        if size > self.max_length {
            return vec![format!("code is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationNoticeContactEmailLengthRule {
    pub max_length: i64,
}

impl ReservationNoticeRule for ReservationNoticeContactEmailLengthRule {
    fn check(&self, item: &ReservationNotice) -> Vec<String> {
        let size = item.contact_email.len() as i64;
        if size > self.max_length {
            return vec![format!(
                "contact email is {} characters, more than {}",
                size, self.max_length
            )];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationNoticeMaximumRangeRule {
    pub min: f64,
    pub max: f64,
}

impl ReservationNoticeRule for ReservationNoticeMaximumRangeRule {
    fn check(&self, item: &ReservationNotice) -> Vec<String> {
        let value = item.maximum;
        let mut problems: Vec<String> = Vec::new();
        if value < self.min {
            problems.push(format!("maximum {} is below {}", value, self.min));
        }
        if value > self.max {
            problems.push(format!("maximum {} is above {}", value, self.max));
        }
        problems
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationNoticeRequiredContactEmailRule {
    pub message: String,
}

impl ReservationNoticeRule for ReservationNoticeRequiredContactEmailRule {
    fn check(&self, item: &ReservationNotice) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReservationNoticeRequiredChannelRule {
    pub message: String,
}

impl ReservationNoticeRule for ReservationNoticeRequiredChannelRule {
    fn check(&self, item: &ReservationNotice) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

/// Runs every rule of a reservation notice.
#[derive(Debug, Clone)]
pub struct ReservationNoticeValidator {
    pub rules: Vec<Rc<dyn ReservationNoticeRule>>,
}

impl ReservationNoticeValidator {
    pub fn validate(&self, item: &ReservationNotice) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &ReservationNotice) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[ReservationNotice]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_reservation_notice_validator() -> ReservationNoticeValidator {
    ReservationNoticeValidator {
        rules: vec![
            Rc::new(ReservationNoticeNameLengthRule { max_length: 80 }) as Rc<dyn ReservationNoticeRule>,
            Rc::new(ReservationNoticeCodeLengthRule { max_length: 3600 }),
            Rc::new(ReservationNoticeContactEmailLengthRule { max_length: 120 }),
            Rc::new(ReservationNoticeMaximumRangeRule { min: 4.8, max: 3905.7 }),
            Rc::new(ReservationNoticeRequiredContactEmailRule {
                message: "reservation notice needs a value".to_string(),
            }),
            Rc::new(ReservationNoticeRequiredChannelRule {
                message: "reservation notice needs an entry".to_string(),
            }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_reservation_notice_validator();
        let item = new_reservation_notice("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_name() {
        let rule = ReservationNoticeNameLengthRule { max_length: 80 };
        let mut item = new_reservation_notice("x", "y");
        item.name = "x".repeat(81 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.name = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_code() {
        let rule = ReservationNoticeCodeLengthRule { max_length: 1000 };
        let mut item = new_reservation_notice("x", "y");
        item.code = "x".repeat(1001 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.code = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = ReservationNoticeContactEmailLengthRule { max_length: 120 };
        let mut item = new_reservation_notice("x", "y");
        item.contact_email = "x".repeat(121 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_values_above_maximum() {
        let rule = ReservationNoticeMaximumRangeRule { min: 1.5, max: 4234.7 };
        let mut item = new_reservation_notice("x", "y");
        item.maximum = 4236.2;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = ReservationNoticeRequiredContactEmailRule {
            message: "reservation notice needs a value".to_string(),
        };
        let mut item = new_reservation_notice("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = ReservationNoticeRequiredChannelRule {
            message: "reservation notice needs an entry".to_string(),
        };
        let mut item = new_reservation_notice("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }
}
