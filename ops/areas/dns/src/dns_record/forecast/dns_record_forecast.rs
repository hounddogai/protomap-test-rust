use super::*;

use acme_kit::text;

/// Where a dns record forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordForecastStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DnsRecordForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DnsRecordForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a dns record forecast and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordForecast {
    pub code: String,
    pub name: String,
    pub priority: i64,
    pub tags: Vec<String>,
    pub quantity: i64,
    pub contact_email: String,
    pub status: DnsRecordForecastStatus,
}

impl DnsRecordForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordForecastStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordForecastStatus) -> bool {
        match self.status {
            DnsRecordForecastStatus::Proposed => {
                match next {
                    DnsRecordForecastStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordForecastStatus::Approved => {
                match next {
                    DnsRecordForecastStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordForecastStatus::Rejected => {
                match next {
                    DnsRecordForecastStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DnsRecordForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "dns record forecast", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.quantity as f64) * (self.priority as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordForecast {
        DnsRecordForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            priority: self.priority,
            tags: self.tags.clone(),
            quantity: self.quantity,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a dns record forecast in status proposed.
pub fn new_dns_record_forecast(code: &str, name: &str) -> DnsRecordForecast {
    DnsRecordForecast {
        code: code.to_string(),
        name: name.to_string(),
        priority: 0,
        tags: Vec::new(),
        quantity: 0,
        contact_email: "".to_string(),
        status: DnsRecordForecastStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_dns_record_forecast("code-276", "name-571");
        assert_eq!(item.status, DnsRecordForecastStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_dns_record_forecast("code-276", "name-571");
        if item.move_to(DnsRecordForecastStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordForecastStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_dns_record_forecast("code-276", "name-571");
        item.status = DnsRecordForecastStatus::Withdrawn;
        assert!(
            item.move_to(DnsRecordForecastStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
