use super::*;

use acme_kit::text;

/// Where a primary data export reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryDataExportReservationStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PrimaryDataExportReservationStatus {
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

impl std::fmt::Display for PrimaryDataExportReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary data export reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDataExportReservation {
    pub code: String,
    pub title: String,
    pub principal: String,
    pub source: String,
    pub revision: i64,
    pub ttl_seconds: i64,
    pub summary: String,
    pub ratio: f64,
    pub status: PrimaryDataExportReservationStatus,
}

impl PrimaryDataExportReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryDataExportReservationStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryDataExportReservationStatus) -> bool {
        match self.status {
            PrimaryDataExportReservationStatus::New => {
                return next == PrimaryDataExportReservationStatus::Queued;
            }
            PrimaryDataExportReservationStatus::Queued => {
                match next {
                    PrimaryDataExportReservationStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryDataExportReservationStatus::Running => {
                return next == PrimaryDataExportReservationStatus::Done
                    || next == PrimaryDataExportReservationStatus::Failed;
            }
            PrimaryDataExportReservationStatus::Done => {
                return next == PrimaryDataExportReservationStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryDataExportReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary data export reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.revision as f64 + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryDataExportReservation {
        PrimaryDataExportReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            principal: self.principal.clone(),
            source: self.source.clone(),
            revision: self.revision,
            ttl_seconds: self.ttl_seconds,
            summary: self.summary.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a primary data export reservation in status new.
pub fn new_primary_data_export_reservation(code: &str, title: &str) -> PrimaryDataExportReservation {
    PrimaryDataExportReservation {
        code: code.to_string(),
        title: title.to_string(),
        principal: "".to_string(),
        source: "".to_string(),
        revision: 0,
        ttl_seconds: 0,
        summary: "".to_string(),
        ratio: 0.0,
        status: PrimaryDataExportReservationStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_primary_data_export_reservation("code-264", "title-400");
        assert_eq!(item.status, PrimaryDataExportReservationStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_primary_data_export_reservation("code-264", "title-400");
        if item.move_to(PrimaryDataExportReservationStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryDataExportReservationStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_primary_data_export_reservation("code-264", "title-400");
        item.status = PrimaryDataExportReservationStatus::Failed;
        assert!(
            item.move_to(PrimaryDataExportReservationStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
