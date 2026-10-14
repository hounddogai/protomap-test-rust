use super::*;

use acme_kit::text;

/// Where a rollback preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackPreferenceStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RollbackPreferenceStatus {
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

impl std::fmt::Display for RollbackPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollback preference. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackPreference {
    pub code: String,
    pub label: String,
    pub source: String,
    pub expires_at: i64,
    pub tags: Vec<String>,
    pub notes: String,
    pub visible: bool,
    pub ratio: f64,
    pub pinned: bool,
    pub status: RollbackPreferenceStatus,
}

impl RollbackPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackPreferenceStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackPreferenceStatus) -> bool {
        match self.status {
            RollbackPreferenceStatus::Planned => {
                match next {
                    RollbackPreferenceStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackPreferenceStatus::Scheduled => {
                return next == RollbackPreferenceStatus::InProgress || next == RollbackPreferenceStatus::Completed;
            }
            RollbackPreferenceStatus::InProgress => {
                return next == RollbackPreferenceStatus::Completed || next == RollbackPreferenceStatus::Cancelled;
            }
            RollbackPreferenceStatus::Completed => {
                match next {
                    RollbackPreferenceStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RollbackPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback preference {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "rollback preference", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_exposure(&self) -> f64 {
        (self.ratio * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollbackPreference {
        RollbackPreference {
            code: self.code.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            expires_at: self.expires_at,
            tags: self.tags.clone(),
            notes: self.notes.clone(),
            visible: self.visible,
            ratio: self.ratio,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a rollback preference in status planned.
pub fn new_rollback_preference(code: &str, label: &str) -> RollbackPreference {
    RollbackPreference {
        code: code.to_string(),
        label: label.to_string(),
        source: "".to_string(),
        expires_at: 0,
        tags: Vec::new(),
        notes: "".to_string(),
        visible: false,
        ratio: 0.0,
        pinned: false,
        status: RollbackPreferenceStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_rollback_preference("code-938", "label-949");
        assert_eq!(item.status, RollbackPreferenceStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_rollback_preference("code-938", "label-949");
        if item.move_to(RollbackPreferenceStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackPreferenceStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_rollback_preference("code-938", "label-949");
        item.status = RollbackPreferenceStatus::Cancelled;
        assert!(
            item.move_to(RollbackPreferenceStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
