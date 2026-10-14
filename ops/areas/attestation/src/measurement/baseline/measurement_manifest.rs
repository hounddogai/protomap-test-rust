use super::*;

use acme_kit::text;

/// Where a measurement manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementManifestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl MeasurementManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for MeasurementManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a measurement manifest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementManifest {
    pub code: String,
    pub title: String,
    pub unit: String,
    pub audience: String,
    pub batch_size: i64,
    pub status: MeasurementManifestStatus,
}

impl MeasurementManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementManifestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementManifestStatus) -> bool {
        match self.status {
            MeasurementManifestStatus::Planned => {
                return next == MeasurementManifestStatus::Completed || next == MeasurementManifestStatus::Cancelled;
            }
            MeasurementManifestStatus::Scheduled => {
                return next == MeasurementManifestStatus::Cancelled;
            }
            MeasurementManifestStatus::InProgress => {
                return next == MeasurementManifestStatus::Completed || next == MeasurementManifestStatus::Cancelled;
            }
            MeasurementManifestStatus::Completed => {
                return next == MeasurementManifestStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MeasurementManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "measurement manifest", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.batch_size as f64
    }

    pub fn copy(&self) -> MeasurementManifest {
        MeasurementManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            unit: self.unit.clone(),
            audience: self.audience.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a measurement manifest in status planned.
pub fn new_measurement_manifest(code: &str, title: &str) -> MeasurementManifest {
    MeasurementManifest {
        code: code.to_string(),
        title: title.to_string(),
        unit: "".to_string(),
        audience: "".to_string(),
        batch_size: 0,
        status: MeasurementManifestStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_measurement_manifest("code-754", "title-466");
        assert_eq!(item.status, MeasurementManifestStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_measurement_manifest("code-754", "title-466");
        if item.move_to(MeasurementManifestStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MeasurementManifestStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_measurement_manifest("code-754", "title-466");
        item.status = MeasurementManifestStatus::Cancelled;
        assert!(
            item.move_to(MeasurementManifestStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
