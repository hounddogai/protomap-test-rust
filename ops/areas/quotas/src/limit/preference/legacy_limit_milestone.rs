use super::*;

use acme_kit::text;

/// Where a legacy limit milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyLimitMilestoneStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LegacyLimitMilestoneStatus {
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

impl std::fmt::Display for LegacyLimitMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One legacy limit milestone, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyLimitMilestone {
    pub code: String,
    pub title: String,
    pub unit: String,
    pub sequence: i64,
    pub key_id: String,
    pub starts_at: i64,
    pub segment: String,
    pub archived: bool,
    pub tags: Vec<String>,
    pub duration_minutes: i64,
    pub status: LegacyLimitMilestoneStatus,
}

impl LegacyLimitMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyLimitMilestoneStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyLimitMilestoneStatus) -> bool {
        match self.status {
            LegacyLimitMilestoneStatus::Draft => {
                return next == LegacyLimitMilestoneStatus::Active;
            }
            LegacyLimitMilestoneStatus::Pending => {
                match next {
                    LegacyLimitMilestoneStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyLimitMilestoneStatus::Active => {
                return next == LegacyLimitMilestoneStatus::Suspended;
            }
            LegacyLimitMilestoneStatus::Suspended => {
                match next {
                    LegacyLimitMilestoneStatus::Closed => {
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

    pub fn move_to(&mut self, next: LegacyLimitMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy limit milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
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
        ((self.sequence as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyLimitMilestone {
        LegacyLimitMilestone {
            code: self.code.clone(),
            title: self.title.clone(),
            unit: self.unit.clone(),
            sequence: self.sequence,
            key_id: self.key_id.clone(),
            starts_at: self.starts_at,
            segment: self.segment.clone(),
            archived: self.archived,
            tags: self.tags.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a legacy limit milestone in status draft.
pub fn new_legacy_limit_milestone(code: &str, title: &str) -> LegacyLimitMilestone {
    LegacyLimitMilestone {
        code: code.to_string(),
        title: title.to_string(),
        unit: "".to_string(),
        sequence: 0,
        key_id: "".to_string(),
        starts_at: 0,
        segment: "".to_string(),
        archived: false,
        tags: Vec::new(),
        duration_minutes: 0,
        status: LegacyLimitMilestoneStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_legacy_limit_milestone("code-436", "title-763");
        assert_eq!(item.status, LegacyLimitMilestoneStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_legacy_limit_milestone("code-436", "title-763");
        if item.move_to(LegacyLimitMilestoneStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyLimitMilestoneStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_legacy_limit_milestone("code-436", "title-763");
        item.status = LegacyLimitMilestoneStatus::Closed;
        assert!(
            item.move_to(LegacyLimitMilestoneStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
