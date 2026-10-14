use super::*;

use acme_kit::text;

/// Where a zone mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneMappingStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ZoneMappingStatus {
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

impl std::fmt::Display for ZoneMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One zone mapping, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneMapping {
    pub code: String,
    pub title: String,
    pub grace_days: i64,
    pub rotation_days: i64,
    pub archived: bool,
    pub external_ref: String,
    pub burst: i64,
    pub threshold: f64,
    pub label: String,
    pub status: ZoneMappingStatus,
}

impl ZoneMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneMappingStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneMappingStatus) -> bool {
        match self.status {
            ZoneMappingStatus::Planned => {
                return next == ZoneMappingStatus::Completed;
            }
            ZoneMappingStatus::Scheduled => {
                return next == ZoneMappingStatus::Completed;
            }
            ZoneMappingStatus::InProgress => {
                match next {
                    ZoneMappingStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneMappingStatus::Completed => {
                return next == ZoneMappingStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ZoneMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.threshold + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZoneMapping {
        ZoneMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            grace_days: self.grace_days,
            rotation_days: self.rotation_days,
            archived: self.archived,
            external_ref: self.external_ref.clone(),
            burst: self.burst,
            threshold: self.threshold,
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a zone mapping in status planned.
pub fn new_zone_mapping(code: &str, title: &str) -> ZoneMapping {
    ZoneMapping {
        code: code.to_string(),
        title: title.to_string(),
        grace_days: 0,
        rotation_days: 0,
        archived: false,
        external_ref: "".to_string(),
        burst: 0,
        threshold: 0.0,
        label: "".to_string(),
        status: ZoneMappingStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_zone_mapping("code-149", "title-752");
        assert_eq!(item.status, ZoneMappingStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_zone_mapping("code-149", "title-752");
        if item.move_to(ZoneMappingStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ZoneMappingStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_zone_mapping("code-149", "title-752");
        item.status = ZoneMappingStatus::Cancelled;
        assert!(
            item.move_to(ZoneMappingStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
