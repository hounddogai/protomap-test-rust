use super::*;

use acme_kit::text;

/// Where a seasonal dns record reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalDnsRecordReservationStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SeasonalDnsRecordReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SeasonalDnsRecordReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal dns record reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDnsRecordReservation {
    pub code: String,
    pub name: String,
    pub channel: String,
    pub threshold: f64,
    pub budget: f64,
    pub port: i64,
    pub contact_email: String,
    pub tenant_id: String,
    pub sequence: i64,
    pub status: SeasonalDnsRecordReservationStatus,
}

impl SeasonalDnsRecordReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalDnsRecordReservationStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalDnsRecordReservationStatus) -> bool {
        match self.status {
            SeasonalDnsRecordReservationStatus::Proposed => {
                match next {
                    SeasonalDnsRecordReservationStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalDnsRecordReservationStatus::Approved => {
                match next {
                    SeasonalDnsRecordReservationStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalDnsRecordReservationStatus::Rejected => {
                match next {
                    SeasonalDnsRecordReservationStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: SeasonalDnsRecordReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal dns record reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.threshold + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalDnsRecordReservation {
        SeasonalDnsRecordReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            channel: self.channel.clone(),
            threshold: self.threshold,
            budget: self.budget,
            port: self.port,
            contact_email: self.contact_email.clone(),
            tenant_id: self.tenant_id.clone(),
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a seasonal dns record reservation in status proposed.
pub fn new_seasonal_dns_record_reservation(code: &str, name: &str) -> SeasonalDnsRecordReservation {
    SeasonalDnsRecordReservation {
        code: code.to_string(),
        name: name.to_string(),
        channel: "".to_string(),
        threshold: 0.0,
        budget: 0.0,
        port: 0,
        contact_email: "".to_string(),
        tenant_id: "".to_string(),
        sequence: 0,
        status: SeasonalDnsRecordReservationStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_seasonal_dns_record_reservation("code-395", "name-880");
        assert_eq!(item.status, SeasonalDnsRecordReservationStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_seasonal_dns_record_reservation("code-395", "name-880");
        if item.move_to(SeasonalDnsRecordReservationStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalDnsRecordReservationStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_seasonal_dns_record_reservation("code-395", "name-880");
        item.status = SeasonalDnsRecordReservationStatus::Withdrawn;
        assert!(
            item.move_to(SeasonalDnsRecordReservationStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
