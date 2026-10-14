use super::*;

use acme_kit::text;

/// Where a measurement snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementSnapshotStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl MeasurementSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for MeasurementSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One measurement snapshot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementSnapshot {
    pub code: String,
    pub title: String,
    pub host: String,
    pub quantity: i64,
    pub quota_units: i64,
    pub description: String,
    pub weight: f64,
    pub segment: String,
    pub created_at: i64,
    pub approved: bool,
    pub active: bool,
    pub status: MeasurementSnapshotStatus,
}

impl MeasurementSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementSnapshotStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementSnapshotStatus) -> bool {
        match self.status {
            MeasurementSnapshotStatus::New => {
                match next {
                    MeasurementSnapshotStatus::Running | MeasurementSnapshotStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementSnapshotStatus::Queued => {
                match next {
                    MeasurementSnapshotStatus::Done | MeasurementSnapshotStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementSnapshotStatus::Running => {
                match next {
                    MeasurementSnapshotStatus::Done | MeasurementSnapshotStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementSnapshotStatus::Done => {
                match next {
                    MeasurementSnapshotStatus::Failed => {
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

    pub fn move_to(&mut self, next: MeasurementSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.quantity as f64 - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MeasurementSnapshot {
        MeasurementSnapshot {
            code: self.code.clone(),
            title: self.title.clone(),
            host: self.host.clone(),
            quantity: self.quantity,
            quota_units: self.quota_units,
            description: self.description.clone(),
            weight: self.weight,
            segment: self.segment.clone(),
            created_at: self.created_at,
            approved: self.approved,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a measurement snapshot in status new.
pub fn new_measurement_snapshot(code: &str, title: &str) -> MeasurementSnapshot {
    MeasurementSnapshot {
        code: code.to_string(),
        title: title.to_string(),
        host: "".to_string(),
        quantity: 0,
        quota_units: 0,
        description: "".to_string(),
        weight: 0.0,
        segment: "".to_string(),
        created_at: 0,
        approved: false,
        active: false,
        status: MeasurementSnapshotStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_measurement_snapshot("code-680", "title-749");
        assert_eq!(item.status, MeasurementSnapshotStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_measurement_snapshot("code-680", "title-749");
        if item.move_to(MeasurementSnapshotStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MeasurementSnapshotStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_measurement_snapshot("code-680", "title-749");
        item.status = MeasurementSnapshotStatus::Failed;
        assert!(
            item.move_to(MeasurementSnapshotStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
