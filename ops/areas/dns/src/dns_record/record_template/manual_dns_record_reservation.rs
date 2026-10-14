use super::*;

use acme_kit::text;

/// Where a manual dns record reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualDnsRecordReservationStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ManualDnsRecordReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ManualDnsRecordReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual dns record reservation as the dns record team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualDnsRecordReservation {
    pub code: String,
    pub title: String,
    pub threshold: f64,
    pub margin: f64,
    pub version: i64,
    pub label: String,
    pub starts_at: i64,
    pub revision: i64,
    pub status: ManualDnsRecordReservationStatus,
}

impl ManualDnsRecordReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualDnsRecordReservationStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualDnsRecordReservationStatus) -> bool {
        match self.status {
            ManualDnsRecordReservationStatus::Idle => {
                match next {
                    ManualDnsRecordReservationStatus::Warming | ManualDnsRecordReservationStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualDnsRecordReservationStatus::Warming => {
                return next == ManualDnsRecordReservationStatus::Draining
                    || next == ManualDnsRecordReservationStatus::Stopped;
            }
            ManualDnsRecordReservationStatus::Ready => {
                return next == ManualDnsRecordReservationStatus::Draining
                    || next == ManualDnsRecordReservationStatus::Stopped;
            }
            ManualDnsRecordReservationStatus::Draining => {
                return next == ManualDnsRecordReservationStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualDnsRecordReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual dns record reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.margin - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualDnsRecordReservation {
        ManualDnsRecordReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            threshold: self.threshold,
            margin: self.margin,
            version: self.version,
            label: self.label.clone(),
            starts_at: self.starts_at,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a manual dns record reservation in status idle.
pub fn new_manual_dns_record_reservation(code: &str, title: &str) -> ManualDnsRecordReservation {
    ManualDnsRecordReservation {
        code: code.to_string(),
        title: title.to_string(),
        threshold: 0.0,
        margin: 0.0,
        version: 0,
        label: "".to_string(),
        starts_at: 0,
        revision: 0,
        status: ManualDnsRecordReservationStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_manual_dns_record_reservation("code-683", "title-964");
        assert_eq!(item.status, ManualDnsRecordReservationStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_manual_dns_record_reservation("code-683", "title-964");
        if item.move_to(ManualDnsRecordReservationStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualDnsRecordReservationStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_manual_dns_record_reservation("code-683", "title-964");
        item.status = ManualDnsRecordReservationStatus::Stopped;
        assert!(
            item.move_to(ManualDnsRecordReservationStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
