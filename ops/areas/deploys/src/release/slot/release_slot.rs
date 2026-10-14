use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a release slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseSlotStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ReleaseSlotStatus {
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

impl std::fmt::Display for ReleaseSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A release slot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseSlot {
    pub code: String,
    pub label: String,
    pub visible: bool,
    pub attempts: i64,
    pub cost: f64,
    pub channel: String,
    pub maximum: f64,
    pub attributes: BTreeMap<String, String>,
    pub owner_id: String,
    pub status: ReleaseSlotStatus,
}

impl ReleaseSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseSlotStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseSlotStatus) -> bool {
        match self.status {
            ReleaseSlotStatus::Open => {
                match next {
                    ReleaseSlotStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseSlotStatus::InReview => {
                match next {
                    ReleaseSlotStatus::Resolved | ReleaseSlotStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseSlotStatus::Resolved => {
                return next == ReleaseSlotStatus::Reopened;
            }
            ReleaseSlotStatus::Reopened => {
                match next {
                    ReleaseSlotStatus::Archived => {
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

    pub fn move_to(&mut self, next: ReleaseSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
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

    pub fn weighted_volume(&self) -> f64 {
        ((self.cost - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReleaseSlot {
        ReleaseSlot {
            code: self.code.clone(),
            label: self.label.clone(),
            visible: self.visible,
            attempts: self.attempts,
            cost: self.cost,
            channel: self.channel.clone(),
            maximum: self.maximum,
            attributes: self.attributes.clone(),
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a release slot in status open.
pub fn new_release_slot(code: &str, label: &str) -> ReleaseSlot {
    ReleaseSlot {
        code: code.to_string(),
        label: label.to_string(),
        visible: false,
        attempts: 0,
        cost: 0.0,
        channel: "".to_string(),
        maximum: 0.0,
        attributes: BTreeMap::new(),
        owner_id: "".to_string(),
        status: ReleaseSlotStatus::Open,
    }
}
