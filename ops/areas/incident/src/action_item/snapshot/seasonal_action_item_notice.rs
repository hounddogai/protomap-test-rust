use super::*;

use acme_kit::text;

/// Where a seasonal action item notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalActionItemNoticeStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SeasonalActionItemNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SeasonalActionItemNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal action item notice as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalActionItemNotice {
    pub code: String,
    pub title: String,
    pub threshold: f64,
    pub ends_at: i64,
    pub score: f64,
    pub ttl_seconds: i64,
    pub name: String,
    pub tags: Vec<String>,
    pub quota_units: i64,
    pub key_id: String,
    pub status: SeasonalActionItemNoticeStatus,
}

impl SeasonalActionItemNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalActionItemNoticeStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalActionItemNoticeStatus) -> bool {
        match self.status {
            SeasonalActionItemNoticeStatus::Proposed => {
                return next == SeasonalActionItemNoticeStatus::Rejected
                    || next == SeasonalActionItemNoticeStatus::Withdrawn;
            }
            SeasonalActionItemNoticeStatus::Approved => {
                match next {
                    SeasonalActionItemNoticeStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalActionItemNoticeStatus::Rejected => {
                return next == SeasonalActionItemNoticeStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalActionItemNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal action item notice {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_value(&self) -> f64 {
        ((self.threshold - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalActionItemNotice {
        SeasonalActionItemNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            threshold: self.threshold,
            ends_at: self.ends_at,
            score: self.score,
            ttl_seconds: self.ttl_seconds,
            name: self.name.clone(),
            tags: self.tags.clone(),
            quota_units: self.quota_units,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a seasonal action item notice in status proposed.
pub fn new_seasonal_action_item_notice(code: &str, title: &str) -> SeasonalActionItemNotice {
    SeasonalActionItemNotice {
        code: code.to_string(),
        title: title.to_string(),
        threshold: 0.0,
        ends_at: 0,
        score: 0.0,
        ttl_seconds: 0,
        name: "".to_string(),
        tags: Vec::new(),
        quota_units: 0,
        key_id: "".to_string(),
        status: SeasonalActionItemNoticeStatus::Proposed,
    }
}
