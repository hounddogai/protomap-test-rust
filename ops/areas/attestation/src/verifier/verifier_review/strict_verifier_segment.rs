use super::*;

use acme_kit::text;

/// Where a strict verifier segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictVerifierSegmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl StrictVerifierSegmentStatus {
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

impl std::fmt::Display for StrictVerifierSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict verifier segment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictVerifierSegment {
    pub code: String,
    pub name: String,
    pub retries: i64,
    pub score: f64,
    pub floor_value: f64,
    pub rotation_days: i64,
    pub active: bool,
    pub starts_at: i64,
    pub reviewer: String,
    pub summary: String,
    pub checksum: String,
    pub status: StrictVerifierSegmentStatus,
}

impl StrictVerifierSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictVerifierSegmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictVerifierSegmentStatus) -> bool {
        match self.status {
            StrictVerifierSegmentStatus::Draft => {
                return next == StrictVerifierSegmentStatus::Active;
            }
            StrictVerifierSegmentStatus::Pending => {
                match next {
                    StrictVerifierSegmentStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictVerifierSegmentStatus::Active => {
                match next {
                    StrictVerifierSegmentStatus::Suspended | StrictVerifierSegmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictVerifierSegmentStatus::Suspended => {
                match next {
                    StrictVerifierSegmentStatus::Closed => {
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

    pub fn move_to(&mut self, next: StrictVerifierSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict verifier segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.rotation_days as f64 + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictVerifierSegment {
        StrictVerifierSegment {
            code: self.code.clone(),
            name: self.name.clone(),
            retries: self.retries,
            score: self.score,
            floor_value: self.floor_value,
            rotation_days: self.rotation_days,
            active: self.active,
            starts_at: self.starts_at,
            reviewer: self.reviewer.clone(),
            summary: self.summary.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a strict verifier segment in status draft.
pub fn new_strict_verifier_segment(code: &str, name: &str) -> StrictVerifierSegment {
    StrictVerifierSegment {
        code: code.to_string(),
        name: name.to_string(),
        retries: 0,
        score: 0.0,
        floor_value: 0.0,
        rotation_days: 0,
        active: false,
        starts_at: 0,
        reviewer: "".to_string(),
        summary: "".to_string(),
        checksum: "".to_string(),
        status: StrictVerifierSegmentStatus::Draft,
    }
}
