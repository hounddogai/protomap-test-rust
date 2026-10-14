use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a zone draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneDraftStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ZoneDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ZoneDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One zone draft, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneDraft {
    pub code: String,
    pub title: String,
    pub fingerprint: String,
    pub unit: String,
    pub version: i64,
    pub minimum: f64,
    pub rotation_days: i64,
    pub attributes: BTreeMap<String, String>,
    pub starts_at: i64,
    pub ttl_seconds: i64,
    pub ceiling: f64,
    pub status: ZoneDraftStatus,
}

impl ZoneDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneDraftStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneDraftStatus) -> bool {
        match self.status {
            ZoneDraftStatus::Proposed => {
                return next == ZoneDraftStatus::Approved || next == ZoneDraftStatus::Withdrawn;
            }
            ZoneDraftStatus::Approved => {
                return next == ZoneDraftStatus::Rejected || next == ZoneDraftStatus::Withdrawn;
            }
            ZoneDraftStatus::Rejected => {
                match next {
                    ZoneDraftStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ZoneDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
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

    pub fn weighted_value(&self) -> f64 {
        ((self.ttl_seconds as f64 + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZoneDraft {
        ZoneDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            fingerprint: self.fingerprint.clone(),
            unit: self.unit.clone(),
            version: self.version,
            minimum: self.minimum,
            rotation_days: self.rotation_days,
            attributes: self.attributes.clone(),
            starts_at: self.starts_at,
            ttl_seconds: self.ttl_seconds,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a zone draft in status proposed.
pub fn new_zone_draft(code: &str, title: &str) -> ZoneDraft {
    ZoneDraft {
        code: code.to_string(),
        title: title.to_string(),
        fingerprint: "".to_string(),
        unit: "".to_string(),
        version: 0,
        minimum: 0.0,
        rotation_days: 0,
        attributes: BTreeMap::new(),
        starts_at: 0,
        ttl_seconds: 0,
        ceiling: 0.0,
        status: ZoneDraftStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_zone_draft("code-520", "title-882");
        assert_eq!(item.status, ZoneDraftStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_zone_draft("code-520", "title-882");
        if item.move_to(ZoneDraftStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ZoneDraftStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_zone_draft("code-520", "title-882");
        item.status = ZoneDraftStatus::Withdrawn;
        assert!(
            item.move_to(ZoneDraftStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
