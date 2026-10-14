use super::*;

use acme_kit::text;

/// Where a regional measurement reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalMeasurementReservationStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RegionalMeasurementReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for RegionalMeasurementReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional measurement reservation as the measurement team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalMeasurementReservation {
    pub code: String,
    pub title: String,
    pub revision: i64,
    pub amount: f64,
    pub ends_at: i64,
    pub fingerprint: String,
    pub channel: String,
    pub contact_email: String,
    pub margin: f64,
    pub timezone: String,
    pub status: RegionalMeasurementReservationStatus,
}

impl RegionalMeasurementReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalMeasurementReservationStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalMeasurementReservationStatus) -> bool {
        match self.status {
            RegionalMeasurementReservationStatus::Open => {
                match next {
                    RegionalMeasurementReservationStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalMeasurementReservationStatus::InReview => {
                match next {
                    RegionalMeasurementReservationStatus::Reopened | RegionalMeasurementReservationStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalMeasurementReservationStatus::Resolved => {
                match next {
                    RegionalMeasurementReservationStatus::Reopened | RegionalMeasurementReservationStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalMeasurementReservationStatus::Reopened => {
                match next {
                    RegionalMeasurementReservationStatus::Archived => {
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

    pub fn move_to(&mut self, next: RegionalMeasurementReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional measurement reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.revision as f64) * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalMeasurementReservation {
        RegionalMeasurementReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            revision: self.revision,
            amount: self.amount,
            ends_at: self.ends_at,
            fingerprint: self.fingerprint.clone(),
            channel: self.channel.clone(),
            contact_email: self.contact_email.clone(),
            margin: self.margin,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a regional measurement reservation in status open.
pub fn new_regional_measurement_reservation(code: &str, title: &str) -> RegionalMeasurementReservation {
    RegionalMeasurementReservation {
        code: code.to_string(),
        title: title.to_string(),
        revision: 0,
        amount: 0.0,
        ends_at: 0,
        fingerprint: "".to_string(),
        channel: "".to_string(),
        contact_email: "".to_string(),
        margin: 0.0,
        timezone: "".to_string(),
        status: RegionalMeasurementReservationStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_regional_measurement_reservation("code-828", "title-503");
        assert_eq!(item.status, RegionalMeasurementReservationStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_regional_measurement_reservation("code-828", "title-503");
        if item.move_to(RegionalMeasurementReservationStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalMeasurementReservationStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_regional_measurement_reservation("code-828", "title-503");
        item.status = RegionalMeasurementReservationStatus::Archived;
        assert!(
            item.move_to(RegionalMeasurementReservationStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
