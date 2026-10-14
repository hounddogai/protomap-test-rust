use super::*;

use acme_kit::text;

/// Where a forecast snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastSnapshotStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ForecastSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for ForecastSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a forecast snapshot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastSnapshot {
    pub code: String,
    pub title: String,
    pub reviewer: String,
    pub notes: String,
    pub score: f64,
    pub checksum: String,
    pub status: ForecastSnapshotStatus,
}

impl ForecastSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastSnapshotStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastSnapshotStatus) -> bool {
        match self.status {
            ForecastSnapshotStatus::Draft => {
                match next {
                    ForecastSnapshotStatus::Pending | ForecastSnapshotStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastSnapshotStatus::Pending => {
                match next {
                    ForecastSnapshotStatus::Active | ForecastSnapshotStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastSnapshotStatus::Active => {
                return next == ForecastSnapshotStatus::Suspended || next == ForecastSnapshotStatus::Closed;
            }
            ForecastSnapshotStatus::Suspended => {
                return next == ForecastSnapshotStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ForecastSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> ForecastSnapshot {
        ForecastSnapshot {
            code: self.code.clone(),
            title: self.title.clone(),
            reviewer: self.reviewer.clone(),
            notes: self.notes.clone(),
            score: self.score,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a forecast snapshot in status draft.
pub fn new_forecast_snapshot(code: &str, title: &str) -> ForecastSnapshot {
    ForecastSnapshot {
        code: code.to_string(),
        title: title.to_string(),
        reviewer: "".to_string(),
        notes: "".to_string(),
        score: 0.0,
        checksum: "".to_string(),
        status: ForecastSnapshotStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_forecast_snapshot("code-305", "title-922");
        assert_eq!(item.status, ForecastSnapshotStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_forecast_snapshot("code-305", "title-922");
        if item.move_to(ForecastSnapshotStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ForecastSnapshotStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_forecast_snapshot("code-305", "title-922");
        item.status = ForecastSnapshotStatus::Closed;
        assert!(
            item.move_to(ForecastSnapshotStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
