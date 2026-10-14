use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a isolation segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationSegmentStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl IsolationSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for IsolationSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One isolation segment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationSegment {
    pub code: String,
    pub title: String,
    pub attributes: BTreeMap<String, String>,
    pub ceiling: f64,
    pub ends_at: i64,
    pub status: IsolationSegmentStatus,
}

impl IsolationSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationSegmentStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationSegmentStatus) -> bool {
        match self.status {
            IsolationSegmentStatus::Requested => {
                return next == IsolationSegmentStatus::Expired;
            }
            IsolationSegmentStatus::Granted => {
                return next == IsolationSegmentStatus::Revoked;
            }
            IsolationSegmentStatus::Revoked => {
                match next {
                    IsolationSegmentStatus::Expired => {
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

    pub fn move_to(&mut self, next: IsolationSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation segment {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
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
        (self.ceiling * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IsolationSegment {
        IsolationSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            attributes: self.attributes.clone(),
            ceiling: self.ceiling,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a isolation segment in status requested.
pub fn new_isolation_segment(code: &str, title: &str) -> IsolationSegment {
    IsolationSegment {
        code: code.to_string(),
        title: title.to_string(),
        attributes: BTreeMap::new(),
        ceiling: 0.0,
        ends_at: 0,
        status: IsolationSegmentStatus::Requested,
    }
}
