use super::*;

use acme_kit::text;

/// Where a primary data export quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryDataExportQuotaStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PrimaryDataExportQuotaStatus {
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

impl std::fmt::Display for PrimaryDataExportQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary data export quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDataExportQuota {
    pub code: String,
    pub title: String,
    pub sequence: i64,
    pub burst: i64,
    pub priority: i64,
    pub revision: i64,
    pub batch_size: i64,
    pub key_id: String,
    pub score: f64,
    pub ceiling: f64,
    pub status: PrimaryDataExportQuotaStatus,
}

impl PrimaryDataExportQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryDataExportQuotaStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryDataExportQuotaStatus) -> bool {
        match self.status {
            PrimaryDataExportQuotaStatus::New => {
                return next == PrimaryDataExportQuotaStatus::Queued || next == PrimaryDataExportQuotaStatus::Running;
            }
            PrimaryDataExportQuotaStatus::Queued => {
                match next {
                    PrimaryDataExportQuotaStatus::Running | PrimaryDataExportQuotaStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryDataExportQuotaStatus::Running => {
                return next == PrimaryDataExportQuotaStatus::Failed;
            }
            PrimaryDataExportQuotaStatus::Done => {
                match next {
                    PrimaryDataExportQuotaStatus::Failed => {
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

    pub fn move_to(&mut self, next: PrimaryDataExportQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary data export quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary data export quota", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.sequence as f64 + (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryDataExportQuota {
        PrimaryDataExportQuota {
            code: self.code.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            burst: self.burst,
            priority: self.priority,
            revision: self.revision,
            batch_size: self.batch_size,
            key_id: self.key_id.clone(),
            score: self.score,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a primary data export quota in status new.
pub fn new_primary_data_export_quota(code: &str, title: &str) -> PrimaryDataExportQuota {
    PrimaryDataExportQuota {
        code: code.to_string(),
        title: title.to_string(),
        sequence: 0,
        burst: 0,
        priority: 0,
        revision: 0,
        batch_size: 0,
        key_id: "".to_string(),
        score: 0.0,
        ceiling: 0.0,
        status: PrimaryDataExportQuotaStatus::New,
    }
}
