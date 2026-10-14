use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a fallback action item assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackActionItemAssignmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FallbackActionItemAssignmentStatus {
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

impl std::fmt::Display for FallbackActionItemAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback action item assignment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackActionItemAssignment {
    pub code: String,
    pub label: String,
    pub limit: i64,
    pub cipher: String,
    pub reviewer: String,
    pub updated_at: i64,
    pub maximum: f64,
    pub rotation_days: i64,
    pub attributes: BTreeMap<String, String>,
    pub title: String,
    pub status: FallbackActionItemAssignmentStatus,
}

impl FallbackActionItemAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackActionItemAssignmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackActionItemAssignmentStatus) -> bool {
        match self.status {
            FallbackActionItemAssignmentStatus::Open => {
                return next == FallbackActionItemAssignmentStatus::InReview;
            }
            FallbackActionItemAssignmentStatus::InReview => {
                match next {
                    FallbackActionItemAssignmentStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackActionItemAssignmentStatus::Resolved => {
                return next == FallbackActionItemAssignmentStatus::Archived;
            }
            FallbackActionItemAssignmentStatus::Reopened => {
                return next == FallbackActionItemAssignmentStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackActionItemAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback action item assignment {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "fallback action item assignment", self.code)
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

    pub fn net_value(&self) -> f64 {
        ((self.limit as f64 - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackActionItemAssignment {
        FallbackActionItemAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            limit: self.limit,
            cipher: self.cipher.clone(),
            reviewer: self.reviewer.clone(),
            updated_at: self.updated_at,
            maximum: self.maximum,
            rotation_days: self.rotation_days,
            attributes: self.attributes.clone(),
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a fallback action item assignment in status open.
pub fn new_fallback_action_item_assignment(code: &str, label: &str) -> FallbackActionItemAssignment {
    FallbackActionItemAssignment {
        code: code.to_string(),
        label: label.to_string(),
        limit: 0,
        cipher: "".to_string(),
        reviewer: "".to_string(),
        updated_at: 0,
        maximum: 0.0,
        rotation_days: 0,
        attributes: BTreeMap::new(),
        title: "".to_string(),
        status: FallbackActionItemAssignmentStatus::Open,
    }
}
