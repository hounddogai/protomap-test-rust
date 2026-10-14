use super::*;

use acme_kit::text;

/// Where a legacy zone override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyZoneOverrideStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LegacyZoneOverrideStatus {
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

impl std::fmt::Display for LegacyZoneOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy zone override and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyZoneOverride {
    pub code: String,
    pub title: String,
    pub approved: bool,
    pub attempts: i64,
    pub threshold: f64,
    pub tags: Vec<String>,
    pub notes: String,
    pub host: String,
    pub quota_units: i64,
    pub created_at: i64,
    pub status: LegacyZoneOverrideStatus,
}

impl LegacyZoneOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyZoneOverrideStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyZoneOverrideStatus) -> bool {
        match self.status {
            LegacyZoneOverrideStatus::Draft => {
                match next {
                    LegacyZoneOverrideStatus::Pending | LegacyZoneOverrideStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyZoneOverrideStatus::Pending => {
                match next {
                    LegacyZoneOverrideStatus::Active | LegacyZoneOverrideStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyZoneOverrideStatus::Active => {
                return next == LegacyZoneOverrideStatus::Suspended;
            }
            LegacyZoneOverrideStatus::Suspended => {
                return next == LegacyZoneOverrideStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyZoneOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy zone override {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.quota_units as f64) * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyZoneOverride {
        LegacyZoneOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            approved: self.approved,
            attempts: self.attempts,
            threshold: self.threshold,
            tags: self.tags.clone(),
            notes: self.notes.clone(),
            host: self.host.clone(),
            quota_units: self.quota_units,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a legacy zone override in status draft.
pub fn new_legacy_zone_override(code: &str, title: &str) -> LegacyZoneOverride {
    LegacyZoneOverride {
        code: code.to_string(),
        title: title.to_string(),
        approved: false,
        attempts: 0,
        threshold: 0.0,
        tags: Vec::new(),
        notes: "".to_string(),
        host: "".to_string(),
        quota_units: 0,
        created_at: 0,
        status: LegacyZoneOverrideStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_legacy_zone_override("code-994", "title-512");
        assert_eq!(item.status, LegacyZoneOverrideStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_legacy_zone_override("code-994", "title-512");
        if item.move_to(LegacyZoneOverrideStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyZoneOverrideStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_legacy_zone_override("code-994", "title-512");
        item.status = LegacyZoneOverrideStatus::Closed;
        assert!(
            item.move_to(LegacyZoneOverrideStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
