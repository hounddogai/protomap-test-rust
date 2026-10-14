use super::*;

use acme_kit::text;

/// Where a zone milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneMilestoneStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ZoneMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ZoneMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A zone milestone as the zone team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneMilestone {
    pub code: String,
    pub title: String,
    pub active: bool,
    pub notes: String,
    pub created_at: i64,
    pub quota_units: i64,
    pub floor_value: f64,
    pub audience: String,
    pub expires_at: i64,
    pub status: ZoneMilestoneStatus,
}

impl ZoneMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneMilestoneStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneMilestoneStatus) -> bool {
        match self.status {
            ZoneMilestoneStatus::Created => {
                return next == ZoneMilestoneStatus::Validated || next == ZoneMilestoneStatus::Published;
            }
            ZoneMilestoneStatus::Validated => {
                return next == ZoneMilestoneStatus::Published || next == ZoneMilestoneStatus::Retired;
            }
            ZoneMilestoneStatus::Published => {
                return next == ZoneMilestoneStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ZoneMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.quota_units as f64 + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZoneMilestone {
        ZoneMilestone {
            code: self.code.clone(),
            title: self.title.clone(),
            active: self.active,
            notes: self.notes.clone(),
            created_at: self.created_at,
            quota_units: self.quota_units,
            floor_value: self.floor_value,
            audience: self.audience.clone(),
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a zone milestone in status created.
pub fn new_zone_milestone(code: &str, title: &str) -> ZoneMilestone {
    ZoneMilestone {
        code: code.to_string(),
        title: title.to_string(),
        active: false,
        notes: "".to_string(),
        created_at: 0,
        quota_units: 0,
        floor_value: 0.0,
        audience: "".to_string(),
        expires_at: 0,
        status: ZoneMilestoneStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_zone_milestone("code-393", "title-37");
        assert_eq!(item.status, ZoneMilestoneStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_zone_milestone("code-393", "title-37");
        if item.move_to(ZoneMilestoneStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ZoneMilestoneStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_zone_milestone("code-393", "title-37");
        item.status = ZoneMilestoneStatus::Retired;
        assert!(
            item.move_to(ZoneMilestoneStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
