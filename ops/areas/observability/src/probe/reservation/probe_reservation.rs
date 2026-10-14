use super::*;

use acme_kit::text;

/// Where a probe reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeReservationStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ProbeReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ProbeReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a probe reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeReservation {
    pub code: String,
    pub label: String,
    pub version: i64,
    pub batch_size: i64,
    pub country: String,
    pub notes: String,
    pub ends_at: i64,
    pub summary: String,
    pub audience: String,
    pub status: ProbeReservationStatus,
}

impl ProbeReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeReservationStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeReservationStatus) -> bool {
        match self.status {
            ProbeReservationStatus::Created => {
                match next {
                    ProbeReservationStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeReservationStatus::Validated => {
                return next == ProbeReservationStatus::Published || next == ProbeReservationStatus::Retired;
            }
            ProbeReservationStatus::Published => {
                match next {
                    ProbeReservationStatus::Retired => {
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

    pub fn move_to(&mut self, next: ProbeReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.version as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeReservation {
        ProbeReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            version: self.version,
            batch_size: self.batch_size,
            country: self.country.clone(),
            notes: self.notes.clone(),
            ends_at: self.ends_at,
            summary: self.summary.clone(),
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a probe reservation in status created.
pub fn new_probe_reservation(code: &str, label: &str) -> ProbeReservation {
    ProbeReservation {
        code: code.to_string(),
        label: label.to_string(),
        version: 0,
        batch_size: 0,
        country: "".to_string(),
        notes: "".to_string(),
        ends_at: 0,
        summary: "".to_string(),
        audience: "".to_string(),
        status: ProbeReservationStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_probe_reservation("code-401", "label-55");
        assert_eq!(item.status, ProbeReservationStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_retired() {
        let mut item = new_probe_reservation("code-401", "label-55");
        if item.move_to(ProbeReservationStatus::Retired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ProbeReservationStatus::Retired, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_probe_reservation("code-401", "label-55");
        item.status = ProbeReservationStatus::Retired;
        assert!(
            item.move_to(ProbeReservationStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
