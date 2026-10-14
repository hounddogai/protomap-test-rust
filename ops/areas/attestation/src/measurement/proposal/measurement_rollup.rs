use super::*;

use acme_kit::text;

/// Where a measurement rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementRollupStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl MeasurementRollupStatus {
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

impl std::fmt::Display for MeasurementRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A measurement rollup. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementRollup {
    pub code: String,
    pub title: String,
    pub margin: f64,
    pub rate: f64,
    pub window_days: i64,
    pub currency: String,
    pub batch_size: i64,
    pub contact_email: String,
    pub status: MeasurementRollupStatus,
}

impl MeasurementRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementRollupStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementRollupStatus) -> bool {
        match self.status {
            MeasurementRollupStatus::New => {
                match next {
                    MeasurementRollupStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementRollupStatus::Queued => {
                return next == MeasurementRollupStatus::Failed;
            }
            MeasurementRollupStatus::Running => {
                return next == MeasurementRollupStatus::Done || next == MeasurementRollupStatus::Failed;
            }
            MeasurementRollupStatus::Done => {
                match next {
                    MeasurementRollupStatus::Failed => {
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

    pub fn move_to(&mut self, next: MeasurementRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.batch_size as f64 + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MeasurementRollup {
        MeasurementRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            margin: self.margin,
            rate: self.rate,
            window_days: self.window_days,
            currency: self.currency.clone(),
            batch_size: self.batch_size,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a measurement rollup in status new.
pub fn new_measurement_rollup(code: &str, title: &str) -> MeasurementRollup {
    MeasurementRollup {
        code: code.to_string(),
        title: title.to_string(),
        margin: 0.0,
        rate: 0.0,
        window_days: 0,
        currency: "".to_string(),
        batch_size: 0,
        contact_email: "".to_string(),
        status: MeasurementRollupStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_measurement_rollup("code-107", "title-844");
        assert_eq!(item.status, MeasurementRollupStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_measurement_rollup("code-107", "title-844");
        if item.move_to(MeasurementRollupStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MeasurementRollupStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_measurement_rollup("code-107", "title-844");
        item.status = MeasurementRollupStatus::Failed;
        assert!(
            item.move_to(MeasurementRollupStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
