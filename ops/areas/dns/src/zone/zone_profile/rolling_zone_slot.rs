use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a rolling zone slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingZoneSlotStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RollingZoneSlotStatus {
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

impl std::fmt::Display for RollingZoneSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling zone slot as the zone team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingZoneSlot {
    pub code: String,
    pub label: String,
    pub archived: bool,
    pub attributes: BTreeMap<String, String>,
    pub updated_at: i64,
    pub starts_at: i64,
    pub segment: String,
    pub quota_units: i64,
    pub weight: f64,
    pub status: RollingZoneSlotStatus,
}

impl RollingZoneSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingZoneSlotStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingZoneSlotStatus) -> bool {
        match self.status {
            RollingZoneSlotStatus::Open => {
                return next == RollingZoneSlotStatus::Resolved;
            }
            RollingZoneSlotStatus::InReview => {
                match next {
                    RollingZoneSlotStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingZoneSlotStatus::Resolved => {
                match next {
                    RollingZoneSlotStatus::Reopened | RollingZoneSlotStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingZoneSlotStatus::Reopened => {
                match next {
                    RollingZoneSlotStatus::Archived => {
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

    pub fn move_to(&mut self, next: RollingZoneSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling zone slot {} cannot move from {} to {}",
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
        format!("{} {}", "rolling zone slot", self.code)
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

    pub fn net_exposure(&self) -> f64 {
        ((self.weight - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingZoneSlot {
        RollingZoneSlot {
            code: self.code.clone(),
            label: self.label.clone(),
            archived: self.archived,
            attributes: self.attributes.clone(),
            updated_at: self.updated_at,
            starts_at: self.starts_at,
            segment: self.segment.clone(),
            quota_units: self.quota_units,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a rolling zone slot in status open.
pub fn new_rolling_zone_slot(code: &str, label: &str) -> RollingZoneSlot {
    RollingZoneSlot {
        code: code.to_string(),
        label: label.to_string(),
        archived: false,
        attributes: BTreeMap::new(),
        updated_at: 0,
        starts_at: 0,
        segment: "".to_string(),
        quota_units: 0,
        weight: 0.0,
        status: RollingZoneSlotStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_rolling_zone_slot("code-279", "label-861");
        assert_eq!(item.status, RollingZoneSlotStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_rolling_zone_slot("code-279", "label-861");
        if item.move_to(RollingZoneSlotStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingZoneSlotStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_rolling_zone_slot("code-279", "label-861");
        item.status = RollingZoneSlotStatus::Archived;
        assert!(
            item.move_to(RollingZoneSlotStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
