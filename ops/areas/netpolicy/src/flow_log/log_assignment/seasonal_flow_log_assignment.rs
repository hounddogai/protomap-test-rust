use super::*;

use acme_kit::text;

/// Where a seasonal flow log assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalFlowLogAssignmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SeasonalFlowLogAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for SeasonalFlowLogAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal flow log assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFlowLogAssignment {
    pub code: String,
    pub label: String,
    pub tags: Vec<String>,
    pub threshold: f64,
    pub notes: String,
    pub title: String,
    pub ends_at: i64,
    pub sequence: i64,
    pub quantity: i64,
    pub region: String,
    pub ttl_seconds: i64,
    pub status: SeasonalFlowLogAssignmentStatus,
}

impl SeasonalFlowLogAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalFlowLogAssignmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalFlowLogAssignmentStatus) -> bool {
        match self.status {
            SeasonalFlowLogAssignmentStatus::Open => {
                return next == SeasonalFlowLogAssignmentStatus::Reopened
                    || next == SeasonalFlowLogAssignmentStatus::Archived;
            }
            SeasonalFlowLogAssignmentStatus::InReview => {
                return next == SeasonalFlowLogAssignmentStatus::Resolved;
            }
            SeasonalFlowLogAssignmentStatus::Resolved => {
                return next == SeasonalFlowLogAssignmentStatus::Reopened;
            }
            SeasonalFlowLogAssignmentStatus::Reopened => {
                match next {
                    SeasonalFlowLogAssignmentStatus::Archived => {
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

    pub fn move_to(&mut self, next: SeasonalFlowLogAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal flow log assignment {} cannot move from {} to {}",
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
        format!("{} {}", "seasonal flow log assignment", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.ends_at as f64 + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalFlowLogAssignment {
        SeasonalFlowLogAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            tags: self.tags.clone(),
            threshold: self.threshold,
            notes: self.notes.clone(),
            title: self.title.clone(),
            ends_at: self.ends_at,
            sequence: self.sequence,
            quantity: self.quantity,
            region: self.region.clone(),
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a seasonal flow log assignment in status open.
pub fn new_seasonal_flow_log_assignment(code: &str, label: &str) -> SeasonalFlowLogAssignment {
    SeasonalFlowLogAssignment {
        code: code.to_string(),
        label: label.to_string(),
        tags: Vec::new(),
        threshold: 0.0,
        notes: "".to_string(),
        title: "".to_string(),
        ends_at: 0,
        sequence: 0,
        quantity: 0,
        region: "".to_string(),
        ttl_seconds: 0,
        status: SeasonalFlowLogAssignmentStatus::Open,
    }
}
