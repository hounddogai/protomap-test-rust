use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a lenient action item override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientActionItemOverrideStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LenientActionItemOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for LenientActionItemOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient action item override, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientActionItemOverride {
    pub code: String,
    pub title: String,
    pub pinned: bool,
    pub cipher: String,
    pub tags: Vec<String>,
    pub visible: bool,
    pub owner_id: String,
    pub phone: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub channel: String,
    pub amount: f64,
    pub status: LenientActionItemOverrideStatus,
}

impl LenientActionItemOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientActionItemOverrideStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientActionItemOverrideStatus) -> bool {
        match self.status {
            LenientActionItemOverrideStatus::Planned => {
                return next == LenientActionItemOverrideStatus::Scheduled
                    || next == LenientActionItemOverrideStatus::Cancelled;
            }
            LenientActionItemOverrideStatus::Scheduled => {
                match next {
                    LenientActionItemOverrideStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientActionItemOverrideStatus::InProgress => {
                match next {
                    LenientActionItemOverrideStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientActionItemOverrideStatus::Completed => {
                match next {
                    LenientActionItemOverrideStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: LenientActionItemOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient action item override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
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

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> LenientActionItemOverride {
        LenientActionItemOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            pinned: self.pinned,
            cipher: self.cipher.clone(),
            tags: self.tags.clone(),
            visible: self.visible,
            owner_id: self.owner_id.clone(),
            phone: self.phone.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            channel: self.channel.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a lenient action item override in status planned.
pub fn new_lenient_action_item_override(code: &str, title: &str) -> LenientActionItemOverride {
    LenientActionItemOverride {
        code: code.to_string(),
        title: title.to_string(),
        pinned: false,
        cipher: "".to_string(),
        tags: Vec::new(),
        visible: false,
        owner_id: "".to_string(),
        phone: "".to_string(),
        label: "".to_string(),
        attributes: BTreeMap::new(),
        channel: "".to_string(),
        amount: 0.0,
        status: LenientActionItemOverrideStatus::Planned,
    }
}
