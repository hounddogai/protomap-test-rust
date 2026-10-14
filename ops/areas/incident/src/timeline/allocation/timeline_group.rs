use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a timeline group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineGroupStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TimelineGroupStatus {
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

impl std::fmt::Display for TimelineGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One timeline group, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineGroup {
    pub code: String,
    pub label: String,
    pub cost: f64,
    pub attributes: BTreeMap<String, String>,
    pub issuer: String,
    pub expires_at: i64,
    pub quantity: i64,
    pub ttl_seconds: i64,
    pub margin: f64,
    pub status: TimelineGroupStatus,
}

impl TimelineGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineGroupStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineGroupStatus) -> bool {
        match self.status {
            TimelineGroupStatus::Draft => {
                return next == TimelineGroupStatus::Closed;
            }
            TimelineGroupStatus::Pending => {
                match next {
                    TimelineGroupStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineGroupStatus::Active => {
                return next == TimelineGroupStatus::Suspended;
            }
            TimelineGroupStatus::Suspended => {
                return next == TimelineGroupStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TimelineGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
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

    pub fn effective_load(&self) -> f64 {
        ((self.margin + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelineGroup {
        TimelineGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            cost: self.cost,
            attributes: self.attributes.clone(),
            issuer: self.issuer.clone(),
            expires_at: self.expires_at,
            quantity: self.quantity,
            ttl_seconds: self.ttl_seconds,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a timeline group in status draft.
pub fn new_timeline_group(code: &str, label: &str) -> TimelineGroup {
    TimelineGroup {
        code: code.to_string(),
        label: label.to_string(),
        cost: 0.0,
        attributes: BTreeMap::new(),
        issuer: "".to_string(),
        expires_at: 0,
        quantity: 0,
        ttl_seconds: 0,
        margin: 0.0,
        status: TimelineGroupStatus::Draft,
    }
}
