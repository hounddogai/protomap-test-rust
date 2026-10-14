use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a grant entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantEntryStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl GrantEntryStatus {
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

impl std::fmt::Display for GrantEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a grant entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantEntry {
    pub code: String,
    pub name: String,
    pub title: String,
    pub owner_id: String,
    pub score: f64,
    pub ttl_seconds: i64,
    pub pinned: bool,
    pub grace_days: i64,
    pub attributes: BTreeMap<String, String>,
    pub currency: String,
    pub status: GrantEntryStatus,
}

impl GrantEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantEntryStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantEntryStatus) -> bool {
        match self.status {
            GrantEntryStatus::Draft => {
                return next == GrantEntryStatus::Suspended;
            }
            GrantEntryStatus::Pending => {
                return next == GrantEntryStatus::Suspended || next == GrantEntryStatus::Closed;
            }
            GrantEntryStatus::Active => {
                return next == GrantEntryStatus::Closed;
            }
            GrantEntryStatus::Suspended => {
                return next == GrantEntryStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GrantEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
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

    pub fn net_load(&self) -> f64 {
        ((self.ttl_seconds as f64) * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantEntry {
        GrantEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            title: self.title.clone(),
            owner_id: self.owner_id.clone(),
            score: self.score,
            ttl_seconds: self.ttl_seconds,
            pinned: self.pinned,
            grace_days: self.grace_days,
            attributes: self.attributes.clone(),
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a grant entry in status draft.
pub fn new_grant_entry(code: &str, name: &str) -> GrantEntry {
    GrantEntry {
        code: code.to_string(),
        name: name.to_string(),
        title: "".to_string(),
        owner_id: "".to_string(),
        score: 0.0,
        ttl_seconds: 0,
        pinned: false,
        grace_days: 0,
        attributes: BTreeMap::new(),
        currency: "".to_string(),
        status: GrantEntryStatus::Draft,
    }
}
