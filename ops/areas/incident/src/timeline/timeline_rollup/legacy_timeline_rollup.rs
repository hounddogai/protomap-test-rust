use super::*;

use acme_kit::text;

/// Where a legacy timeline rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyTimelineRollupStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LegacyTimelineRollupStatus {
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

impl std::fmt::Display for LegacyTimelineRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy timeline rollup. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTimelineRollup {
    pub code: String,
    pub title: String,
    pub tags: Vec<String>,
    pub category: String,
    pub expires_at: i64,
    pub created_at: i64,
    pub status: LegacyTimelineRollupStatus,
}

impl LegacyTimelineRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyTimelineRollupStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyTimelineRollupStatus) -> bool {
        match self.status {
            LegacyTimelineRollupStatus::Draft => {
                match next {
                    LegacyTimelineRollupStatus::Pending | LegacyTimelineRollupStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyTimelineRollupStatus::Pending => {
                return next == LegacyTimelineRollupStatus::Active || next == LegacyTimelineRollupStatus::Closed;
            }
            LegacyTimelineRollupStatus::Active => {
                match next {
                    LegacyTimelineRollupStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyTimelineRollupStatus::Suspended => {
                return next == LegacyTimelineRollupStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyTimelineRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy timeline rollup {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "legacy timeline rollup", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_volume(&self) -> f64 {
        ((self.expires_at as f64 - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyTimelineRollup {
        LegacyTimelineRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            tags: self.tags.clone(),
            category: self.category.clone(),
            expires_at: self.expires_at,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a legacy timeline rollup in status draft.
pub fn new_legacy_timeline_rollup(code: &str, title: &str) -> LegacyTimelineRollup {
    LegacyTimelineRollup {
        code: code.to_string(),
        title: title.to_string(),
        tags: Vec::new(),
        category: "".to_string(),
        expires_at: 0,
        created_at: 0,
        status: LegacyTimelineRollupStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_legacy_timeline_rollup("code-674", "title-434");
        assert_eq!(item.status, LegacyTimelineRollupStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_legacy_timeline_rollup("code-674", "title-434");
        if item.move_to(LegacyTimelineRollupStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyTimelineRollupStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_legacy_timeline_rollup("code-674", "title-434");
        item.status = LegacyTimelineRollupStatus::Closed;
        assert!(
            item.move_to(LegacyTimelineRollupStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
