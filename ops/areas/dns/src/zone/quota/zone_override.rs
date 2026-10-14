use super::*;

use acme_kit::text;

/// Where a zone override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneOverrideStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ZoneOverrideStatus {
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

impl std::fmt::Display for ZoneOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A zone override as the zone team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneOverride {
    pub code: String,
    pub title: String,
    pub label: String,
    pub category: String,
    pub limit: i64,
    pub pinned: bool,
    pub timezone: String,
    pub threshold: f64,
    pub tenant_id: String,
    pub status: ZoneOverrideStatus,
}

impl ZoneOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneOverrideStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneOverrideStatus) -> bool {
        match self.status {
            ZoneOverrideStatus::Draft => {
                match next {
                    ZoneOverrideStatus::Pending | ZoneOverrideStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneOverrideStatus::Pending => {
                return next == ZoneOverrideStatus::Suspended || next == ZoneOverrideStatus::Closed;
            }
            ZoneOverrideStatus::Active => {
                match next {
                    ZoneOverrideStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneOverrideStatus::Suspended => {
                match next {
                    ZoneOverrideStatus::Closed => {
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

    pub fn move_to(&mut self, next: ZoneOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "zone override", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.limit as f64 + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ZoneOverride {
        ZoneOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            label: self.label.clone(),
            category: self.category.clone(),
            limit: self.limit,
            pinned: self.pinned,
            timezone: self.timezone.clone(),
            threshold: self.threshold,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a zone override in status draft.
pub fn new_zone_override(code: &str, title: &str) -> ZoneOverride {
    ZoneOverride {
        code: code.to_string(),
        title: title.to_string(),
        label: "".to_string(),
        category: "".to_string(),
        limit: 0,
        pinned: false,
        timezone: "".to_string(),
        threshold: 0.0,
        tenant_id: "".to_string(),
        status: ZoneOverrideStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_zone_override("code-23", "title-769");
        assert_eq!(item.status, ZoneOverrideStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_zone_override("code-23", "title-769");
        if item.move_to(ZoneOverrideStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ZoneOverrideStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_zone_override("code-23", "title-769");
        item.status = ZoneOverrideStatus::Closed;
        assert!(
            item.move_to(ZoneOverrideStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
