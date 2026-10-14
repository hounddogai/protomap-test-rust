use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a sink adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkAdjustmentStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SinkAdjustmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SinkAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One sink adjustment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkAdjustment {
    pub code: String,
    pub title: String,
    pub segment: String,
    pub ttl_seconds: i64,
    pub attributes: BTreeMap<String, String>,
    pub quantity: i64,
    pub grace_days: i64,
    pub name: String,
    pub minimum: f64,
    pub status: SinkAdjustmentStatus,
}

impl SinkAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkAdjustmentStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkAdjustmentStatus) -> bool {
        match self.status {
            SinkAdjustmentStatus::Requested => {
                return next == SinkAdjustmentStatus::Granted || next == SinkAdjustmentStatus::Revoked;
            }
            SinkAdjustmentStatus::Granted => {
                return next == SinkAdjustmentStatus::Revoked;
            }
            SinkAdjustmentStatus::Revoked => {
                match next {
                    SinkAdjustmentStatus::Expired => {
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

    pub fn move_to(&mut self, next: SinkAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
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
        ((self.quantity as f64 - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SinkAdjustment {
        SinkAdjustment {
            code: self.code.clone(),
            title: self.title.clone(),
            segment: self.segment.clone(),
            ttl_seconds: self.ttl_seconds,
            attributes: self.attributes.clone(),
            quantity: self.quantity,
            grace_days: self.grace_days,
            name: self.name.clone(),
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a sink adjustment in status requested.
pub fn new_sink_adjustment(code: &str, title: &str) -> SinkAdjustment {
    SinkAdjustment {
        code: code.to_string(),
        title: title.to_string(),
        segment: "".to_string(),
        ttl_seconds: 0,
        attributes: BTreeMap::new(),
        quantity: 0,
        grace_days: 0,
        name: "".to_string(),
        minimum: 0.0,
        status: SinkAdjustmentStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_sink_adjustment("code-399", "title-173");
        assert_eq!(item.status, SinkAdjustmentStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_sink_adjustment("code-399", "title-173");
        if item.move_to(SinkAdjustmentStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SinkAdjustmentStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_sink_adjustment("code-399", "title-173");
        item.status = SinkAdjustmentStatus::Expired;
        assert!(
            item.move_to(SinkAdjustmentStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
