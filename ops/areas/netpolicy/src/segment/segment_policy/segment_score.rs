use super::*;

use acme_kit::text;

/// Where a segment score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentScoreStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SegmentScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SegmentScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One segment score, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentScore {
    pub code: String,
    pub label: String,
    pub window_days: i64,
    pub cost: f64,
    pub maximum: f64,
    pub unit: String,
    pub version: i64,
    pub status: SegmentScoreStatus,
}

impl SegmentScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentScoreStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentScoreStatus) -> bool {
        match self.status {
            SegmentScoreStatus::Proposed => {
                return next == SegmentScoreStatus::Rejected;
            }
            SegmentScoreStatus::Approved => {
                match next {
                    SegmentScoreStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentScoreStatus::Rejected => {
                return next == SegmentScoreStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SegmentScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.version as f64 - self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SegmentScore {
        SegmentScore {
            code: self.code.clone(),
            label: self.label.clone(),
            window_days: self.window_days,
            cost: self.cost,
            maximum: self.maximum,
            unit: self.unit.clone(),
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a segment score in status proposed.
pub fn new_segment_score(code: &str, label: &str) -> SegmentScore {
    SegmentScore {
        code: code.to_string(),
        label: label.to_string(),
        window_days: 0,
        cost: 0.0,
        maximum: 0.0,
        unit: "".to_string(),
        version: 0,
        status: SegmentScoreStatus::Proposed,
    }
}
