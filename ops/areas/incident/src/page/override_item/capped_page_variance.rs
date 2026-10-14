use super::*;

use acme_kit::text;

/// Where a capped page variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedPageVarianceStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CappedPageVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CappedPageVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a capped page variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedPageVariance {
    pub code: String,
    pub label: String,
    pub ttl_seconds: i64,
    pub maximum: f64,
    pub fingerprint: String,
    pub owner_id: String,
    pub notes: String,
    pub status: CappedPageVarianceStatus,
}

impl CappedPageVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedPageVarianceStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedPageVarianceStatus) -> bool {
        match self.status {
            CappedPageVarianceStatus::Proposed => {
                match next {
                    CappedPageVarianceStatus::Rejected | CappedPageVarianceStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedPageVarianceStatus::Approved => {
                match next {
                    CappedPageVarianceStatus::Rejected | CappedPageVarianceStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedPageVarianceStatus::Rejected => {
                return next == CappedPageVarianceStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedPageVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped page variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.ttl_seconds as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedPageVariance {
        CappedPageVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            ttl_seconds: self.ttl_seconds,
            maximum: self.maximum,
            fingerprint: self.fingerprint.clone(),
            owner_id: self.owner_id.clone(),
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a capped page variance in status proposed.
pub fn new_capped_page_variance(code: &str, label: &str) -> CappedPageVariance {
    CappedPageVariance {
        code: code.to_string(),
        label: label.to_string(),
        ttl_seconds: 0,
        maximum: 0.0,
        fingerprint: "".to_string(),
        owner_id: "".to_string(),
        notes: "".to_string(),
        status: CappedPageVarianceStatus::Proposed,
    }
}
