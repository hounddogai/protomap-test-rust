use super::*;

use acme_kit::text;

/// Where a measurement reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementReservationStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl MeasurementReservationStatus {
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

impl std::fmt::Display for MeasurementReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A measurement reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementReservation {
    pub code: String,
    pub label: String,
    pub ends_at: i64,
    pub checksum: String,
    pub description: String,
    pub score: f64,
    pub capacity: i64,
    pub revision: i64,
    pub namespace: String,
    pub updated_at: i64,
    pub status: MeasurementReservationStatus,
}

impl MeasurementReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementReservationStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementReservationStatus) -> bool {
        match self.status {
            MeasurementReservationStatus::Idle => {
                return next == MeasurementReservationStatus::Stopped;
            }
            MeasurementReservationStatus::Warming => {
                return next == MeasurementReservationStatus::Draining;
            }
            MeasurementReservationStatus::Ready => {
                match next {
                    MeasurementReservationStatus::Draining | MeasurementReservationStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementReservationStatus::Draining => {
                return next == MeasurementReservationStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MeasurementReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement reservation {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.capacity as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MeasurementReservation {
        MeasurementReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            ends_at: self.ends_at,
            checksum: self.checksum.clone(),
            description: self.description.clone(),
            score: self.score,
            capacity: self.capacity,
            revision: self.revision,
            namespace: self.namespace.clone(),
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a measurement reservation in status idle.
pub fn new_measurement_reservation(code: &str, label: &str) -> MeasurementReservation {
    MeasurementReservation {
        code: code.to_string(),
        label: label.to_string(),
        ends_at: 0,
        checksum: "".to_string(),
        description: "".to_string(),
        score: 0.0,
        capacity: 0,
        revision: 0,
        namespace: "".to_string(),
        updated_at: 0,
        status: MeasurementReservationStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_measurement_reservation("code-946", "label-492");
        assert_eq!(item.status, MeasurementReservationStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_measurement_reservation("code-946", "label-492");
        if item.move_to(MeasurementReservationStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MeasurementReservationStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_measurement_reservation("code-946", "label-492");
        item.status = MeasurementReservationStatus::Stopped;
        assert!(
            item.move_to(MeasurementReservationStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
