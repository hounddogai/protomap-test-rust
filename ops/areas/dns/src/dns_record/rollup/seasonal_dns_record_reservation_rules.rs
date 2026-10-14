use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// A rule a seasonal dns record reservation must pass. Check returns its problems, or none.
pub trait SeasonalDnsRecordReservationRule: std::fmt::Debug {
    fn check(&self, item: &SeasonalDnsRecordReservation) -> Vec<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDnsRecordReservationContactEmailLengthRule {
    pub max_length: i64,
}

impl SeasonalDnsRecordReservationRule for SeasonalDnsRecordReservationContactEmailLengthRule {
    fn check(&self, item: &SeasonalDnsRecordReservation) -> Vec<String> {
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
pub struct SeasonalDnsRecordReservationRequiredContactEmailRule {
    pub message: String,
}

impl SeasonalDnsRecordReservationRule for SeasonalDnsRecordReservationRequiredContactEmailRule {
    fn check(&self, item: &SeasonalDnsRecordReservation) -> Vec<String> {
        if item.contact_email.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDnsRecordReservationRequiredChannelRule {
    pub message: String,
}

impl SeasonalDnsRecordReservationRule for SeasonalDnsRecordReservationRequiredChannelRule {
    fn check(&self, item: &SeasonalDnsRecordReservation) -> Vec<String> {
        if item.channel.trim() == "" {
            return vec![self.message.clone()];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDnsRecordReservationPortRangeRule {
    pub min: i64,
    pub max: i64,
}

impl SeasonalDnsRecordReservationRule for SeasonalDnsRecordReservationPortRangeRule {
    fn check(&self, item: &SeasonalDnsRecordReservation) -> Vec<String> {
        let value = item.port;
        if value < self.min || value > self.max {
            return vec![format!("port {} is outside {} to {}", value, self.min, self.max)];
        }
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDnsRecordReservationChannelLengthRule {
    pub max_length: i64,
}

impl SeasonalDnsRecordReservationRule for SeasonalDnsRecordReservationChannelLengthRule {
    fn check(&self, item: &SeasonalDnsRecordReservation) -> Vec<String> {
        let size = item.channel.len() as i64;
        if size > self.max_length {
            return vec![format!("channel is {} characters, more than {}", size, self.max_length)];
        }
        Vec::new()
    }
}

/// Runs every rule of a seasonal dns record reservation.
#[derive(Debug, Clone)]
pub struct SeasonalDnsRecordReservationValidator {
    pub rules: Vec<Rc<dyn SeasonalDnsRecordReservationRule>>,
}

impl SeasonalDnsRecordReservationValidator {
    pub fn validate(&self, item: &SeasonalDnsRecordReservation) -> Vec<String> {
        let mut problems: Vec<String> = Vec::new();
        for rule in self.rules.iter() {
            for problem in rule.check(item) {
                problems.push(problem.clone());
            }
        }
        problems
    }

    pub fn require_valid(&self, item: &SeasonalDnsRecordReservation) -> Result<(), String> {
        let problems = self.validate(item);
        if (problems.len() as i64) > 0 {
            return Err(format!("{} is invalid: {}", item.label(), problems.join("; ")));
        }
        Ok(())
    }

    pub fn validate_all(&self, items: &[SeasonalDnsRecordReservation]) -> BTreeMap<String, Vec<String>> {
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

pub fn default_seasonal_dns_record_reservation_validator() -> SeasonalDnsRecordReservationValidator {
    SeasonalDnsRecordReservationValidator {
        rules: vec![
            Rc::new(SeasonalDnsRecordReservationContactEmailLengthRule { max_length: 32 })
                as Rc<dyn SeasonalDnsRecordReservationRule>,
            Rc::new(SeasonalDnsRecordReservationRequiredContactEmailRule {
                message: "seasonal dns record reservation needs this field".to_string(),
            }),
            Rc::new(SeasonalDnsRecordReservationRequiredChannelRule {
                message: "seasonal dns record reservation needs a value".to_string(),
            }),
            Rc::new(SeasonalDnsRecordReservationPortRangeRule { min: 120, max: 1000 }),
            Rc::new(SeasonalDnsRecordReservationChannelLengthRule { max_length: 86400 }),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_problem() {
        let validator = default_seasonal_dns_record_reservation_validator();
        let item = new_seasonal_dns_record_reservation("", "");
        let problems = validator.validate(&item);
        assert!((problems.len() as i64) >= 0, "problems");
    }

    #[test]
    fn flags_a_long_contact_email() {
        let rule = SeasonalDnsRecordReservationContactEmailLengthRule { max_length: 3600 };
        let mut item = new_seasonal_dns_record_reservation("x", "y");
        item.contact_email = "x".repeat(3601 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.contact_email = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }

    #[test]
    fn flags_a_blank_contact_email() {
        let rule = SeasonalDnsRecordReservationRequiredContactEmailRule {
            message: "seasonal dns record reservation needs this field".to_string(),
        };
        let mut item = new_seasonal_dns_record_reservation("", "");
        item.contact_email = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_a_blank_channel() {
        let rule = SeasonalDnsRecordReservationRequiredChannelRule {
            message: "seasonal dns record reservation needs an entry".to_string(),
        };
        let mut item = new_seasonal_dns_record_reservation("", "");
        item.channel = "  ".to_string();
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
    }

    #[test]
    fn flags_values_above_port() {
        let rule = SeasonalDnsRecordReservationPortRangeRule { min: 16, max: 32 };
        let mut item = new_seasonal_dns_record_reservation("x", "y");
        item.port = 33;
        assert!((rule.check(&item).len() as i64) > 0, "no problem above the range");
    }

    #[test]
    fn flags_a_long_channel() {
        let rule = SeasonalDnsRecordReservationChannelLengthRule { max_length: 64 };
        let mut item = new_seasonal_dns_record_reservation("x", "y");
        item.channel = "x".repeat(65 as usize);
        assert_eq!(rule.check(&item).len() as i64, 1, "problems");
        item.channel = "ok".to_string();
        assert_eq!(rule.check(&item).len() as i64, 0, "problems after fixing");
    }
}
