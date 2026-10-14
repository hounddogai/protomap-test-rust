use super::*;

use acme_kit::text;

/// Where a tiered egress cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredEgressCycleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TieredEgressCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TieredEgressCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered egress cycle. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredEgressCycle {
    pub code: String,
    pub name: String,
    pub minimum: f64,
    pub revision: i64,
    pub capacity: i64,
    pub segment: String,
    pub archived: bool,
    pub tags: Vec<String>,
    pub threshold: f64,
    pub notes: String,
    pub quota_units: i64,
    pub status: TieredEgressCycleStatus,
}

impl TieredEgressCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredEgressCycleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredEgressCycleStatus) -> bool {
        match self.status {
            TieredEgressCycleStatus::Proposed => {
                match next {
                    TieredEgressCycleStatus::Approved | TieredEgressCycleStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredEgressCycleStatus::Approved => {
                return next == TieredEgressCycleStatus::Rejected || next == TieredEgressCycleStatus::Withdrawn;
            }
            TieredEgressCycleStatus::Rejected => {
                match next {
                    TieredEgressCycleStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: TieredEgressCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered egress cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.capacity as f64 - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredEgressCycle {
        TieredEgressCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            minimum: self.minimum,
            revision: self.revision,
            capacity: self.capacity,
            segment: self.segment.clone(),
            archived: self.archived,
            tags: self.tags.clone(),
            threshold: self.threshold,
            notes: self.notes.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a tiered egress cycle in status proposed.
pub fn new_tiered_egress_cycle(code: &str, name: &str) -> TieredEgressCycle {
    TieredEgressCycle {
        code: code.to_string(),
        name: name.to_string(),
        minimum: 0.0,
        revision: 0,
        capacity: 0,
        segment: "".to_string(),
        archived: false,
        tags: Vec::new(),
        threshold: 0.0,
        notes: "".to_string(),
        quota_units: 0,
        status: TieredEgressCycleStatus::Proposed,
    }
}
