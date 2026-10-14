use super::*;

use acme_kit::text;

/// Where a dns record reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordReservationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DnsRecordReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DnsRecordReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dns record reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordReservation {
    pub code: String,
    pub label: String,
    pub unit: String,
    pub score: f64,
    pub duration_minutes: i64,
    pub region: String,
    pub expires_at: i64,
    pub version: i64,
    pub threshold: f64,
    pub timezone: String,
    pub audience: String,
    pub status: DnsRecordReservationStatus,
}

impl DnsRecordReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordReservationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordReservationStatus) -> bool {
        match self.status {
            DnsRecordReservationStatus::Requested => {
                return next == DnsRecordReservationStatus::Granted;
            }
            DnsRecordReservationStatus::Granted => {
                match next {
                    DnsRecordReservationStatus::Revoked | DnsRecordReservationStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordReservationStatus::Revoked => {
                match next {
                    DnsRecordReservationStatus::Expired => {
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

    pub fn move_to(&mut self, next: DnsRecordReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.threshold - (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordReservation {
        DnsRecordReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            unit: self.unit.clone(),
            score: self.score,
            duration_minutes: self.duration_minutes,
            region: self.region.clone(),
            expires_at: self.expires_at,
            version: self.version,
            threshold: self.threshold,
            timezone: self.timezone.clone(),
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a dns record reservation in status requested.
pub fn new_dns_record_reservation(code: &str, label: &str) -> DnsRecordReservation {
    DnsRecordReservation {
        code: code.to_string(),
        label: label.to_string(),
        unit: "".to_string(),
        score: 0.0,
        duration_minutes: 0,
        region: "".to_string(),
        expires_at: 0,
        version: 0,
        threshold: 0.0,
        timezone: "".to_string(),
        audience: "".to_string(),
        status: DnsRecordReservationStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_dns_record_reservation("code-26", "label-216");
        assert_eq!(item.status, DnsRecordReservationStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_dns_record_reservation("code-26", "label-216");
        if item.move_to(DnsRecordReservationStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordReservationStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_dns_record_reservation("code-26", "label-216");
        item.status = DnsRecordReservationStatus::Expired;
        assert!(
            item.move_to(DnsRecordReservationStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
