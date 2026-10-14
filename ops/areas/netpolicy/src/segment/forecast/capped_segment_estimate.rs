use super::*;

use acme_kit::text;

/// Where a capped segment estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedSegmentEstimateStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl CappedSegmentEstimateStatus {
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

impl std::fmt::Display for CappedSegmentEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped segment estimate as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedSegmentEstimate {
    pub code: String,
    pub label: String,
    pub ends_at: i64,
    pub retries: i64,
    pub region: String,
    pub margin: f64,
    pub scope: String,
    pub unit: String,
    pub phone: String,
    pub country: String,
    pub status: CappedSegmentEstimateStatus,
}

impl CappedSegmentEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedSegmentEstimateStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedSegmentEstimateStatus) -> bool {
        match self.status {
            CappedSegmentEstimateStatus::New => {
                return next == CappedSegmentEstimateStatus::Done;
            }
            CappedSegmentEstimateStatus::Queued => {
                match next {
                    CappedSegmentEstimateStatus::Running | CappedSegmentEstimateStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedSegmentEstimateStatus::Running => {
                match next {
                    CappedSegmentEstimateStatus::Done | CappedSegmentEstimateStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedSegmentEstimateStatus::Done => {
                return next == CappedSegmentEstimateStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedSegmentEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped segment estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.margin - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedSegmentEstimate {
        CappedSegmentEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            ends_at: self.ends_at,
            retries: self.retries,
            region: self.region.clone(),
            margin: self.margin,
            scope: self.scope.clone(),
            unit: self.unit.clone(),
            phone: self.phone.clone(),
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a capped segment estimate in status new.
pub fn new_capped_segment_estimate(code: &str, label: &str) -> CappedSegmentEstimate {
    CappedSegmentEstimate {
        code: code.to_string(),
        label: label.to_string(),
        ends_at: 0,
        retries: 0,
        region: "".to_string(),
        margin: 0.0,
        scope: "".to_string(),
        unit: "".to_string(),
        phone: "".to_string(),
        country: "".to_string(),
        status: CappedSegmentEstimateStatus::New,
    }
}
