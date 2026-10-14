use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a legacy evidence assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyEvidenceAssignmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LegacyEvidenceAssignmentStatus {
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

impl std::fmt::Display for LegacyEvidenceAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One legacy evidence assignment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyEvidenceAssignment {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub ceiling: f64,
    pub attributes: BTreeMap<String, String>,
    pub updated_at: i64,
    pub description: String,
    pub status: LegacyEvidenceAssignmentStatus,
}

impl LegacyEvidenceAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyEvidenceAssignmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyEvidenceAssignmentStatus) -> bool {
        match self.status {
            LegacyEvidenceAssignmentStatus::Open => {
                return next == LegacyEvidenceAssignmentStatus::Archived;
            }
            LegacyEvidenceAssignmentStatus::InReview => {
                return next == LegacyEvidenceAssignmentStatus::Reopened;
            }
            LegacyEvidenceAssignmentStatus::Resolved => {
                return next == LegacyEvidenceAssignmentStatus::Archived;
            }
            LegacyEvidenceAssignmentStatus::Reopened => {
                return next == LegacyEvidenceAssignmentStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyEvidenceAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy evidence assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.ceiling - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyEvidenceAssignment {
        LegacyEvidenceAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            ceiling: self.ceiling,
            attributes: self.attributes.clone(),
            updated_at: self.updated_at,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a legacy evidence assignment in status open.
pub fn new_legacy_evidence_assignment(code: &str, name: &str) -> LegacyEvidenceAssignment {
    LegacyEvidenceAssignment {
        code: code.to_string(),
        name: name.to_string(),
        summary: "".to_string(),
        ceiling: 0.0,
        attributes: BTreeMap::new(),
        updated_at: 0,
        description: "".to_string(),
        status: LegacyEvidenceAssignmentStatus::Open,
    }
}
