use super::*;

use acme_kit::text;

/// Where a zone slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneSlotStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ZoneSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for ZoneSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A zone slot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneSlot {
    pub code: String,
    pub name: String,
    pub label: String,
    pub ttl_seconds: i64,
    pub audience: String,
    pub weight: f64,
    pub quota_units: i64,
    pub rotation_days: i64,
    pub rate: f64,
    pub archived: bool,
    pub tags: Vec<String>,
    pub status: ZoneSlotStatus,
}

impl ZoneSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneSlotStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneSlotStatus) -> bool {
        match self.status {
            ZoneSlotStatus::New => {
                return next == ZoneSlotStatus::Running || next == ZoneSlotStatus::Done;
            }
            ZoneSlotStatus::Queued => {
                match next {
                    ZoneSlotStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneSlotStatus::Running => {
                match next {
                    ZoneSlotStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneSlotStatus::Done => {
                match next {
                    ZoneSlotStatus::Failed => {
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

    pub fn move_to(&mut self, next: ZoneSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.rotation_days as f64 + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZoneSlot {
        ZoneSlot {
            code: self.code.clone(),
            name: self.name.clone(),
            label: self.label.clone(),
            ttl_seconds: self.ttl_seconds,
            audience: self.audience.clone(),
            weight: self.weight,
            quota_units: self.quota_units,
            rotation_days: self.rotation_days,
            rate: self.rate,
            archived: self.archived,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a zone slot in status new.
pub fn new_zone_slot(code: &str, name: &str) -> ZoneSlot {
    ZoneSlot {
        code: code.to_string(),
        name: name.to_string(),
        label: "".to_string(),
        ttl_seconds: 0,
        audience: "".to_string(),
        weight: 0.0,
        quota_units: 0,
        rotation_days: 0,
        rate: 0.0,
        archived: false,
        tags: Vec::new(),
        status: ZoneSlotStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_zone_slot("code-391", "name-518");
        assert_eq!(item.status, ZoneSlotStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_zone_slot("code-391", "name-518");
        if item.move_to(ZoneSlotStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ZoneSlotStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_zone_slot("code-391", "name-518");
        item.status = ZoneSlotStatus::Failed;
        assert!(item.move_to(ZoneSlotStatus::New).is_err(), "expected move to fail");
    }
}
