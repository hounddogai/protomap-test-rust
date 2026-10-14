use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a global zone digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalZoneDigestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl GlobalZoneDigestStatus {
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

impl std::fmt::Display for GlobalZoneDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global zone digest as the zone team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalZoneDigest {
    pub code: String,
    pub label: String,
    pub channel: String,
    pub tags: Vec<String>,
    pub floor_value: f64,
    pub grace_days: i64,
    pub segment: String,
    pub summary: String,
    pub attributes: BTreeMap<String, String>,
    pub owner_id: String,
    pub status: GlobalZoneDigestStatus,
}

impl GlobalZoneDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalZoneDigestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalZoneDigestStatus) -> bool {
        match self.status {
            GlobalZoneDigestStatus::Draft => {
                match next {
                    GlobalZoneDigestStatus::Pending | GlobalZoneDigestStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalZoneDigestStatus::Pending => {
                match next {
                    GlobalZoneDigestStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalZoneDigestStatus::Active => {
                return next == GlobalZoneDigestStatus::Suspended;
            }
            GlobalZoneDigestStatus::Suspended => {
                return next == GlobalZoneDigestStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalZoneDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global zone digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
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

    pub fn projected_value(&self) -> f64 {
        ((self.floor_value + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalZoneDigest {
        GlobalZoneDigest {
            code: self.code.clone(),
            label: self.label.clone(),
            channel: self.channel.clone(),
            tags: self.tags.clone(),
            floor_value: self.floor_value,
            grace_days: self.grace_days,
            segment: self.segment.clone(),
            summary: self.summary.clone(),
            attributes: self.attributes.clone(),
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a global zone digest in status draft.
pub fn new_global_zone_digest(code: &str, label: &str) -> GlobalZoneDigest {
    GlobalZoneDigest {
        code: code.to_string(),
        label: label.to_string(),
        channel: "".to_string(),
        tags: Vec::new(),
        floor_value: 0.0,
        grace_days: 0,
        segment: "".to_string(),
        summary: "".to_string(),
        attributes: BTreeMap::new(),
        owner_id: "".to_string(),
        status: GlobalZoneDigestStatus::Draft,
    }
}
