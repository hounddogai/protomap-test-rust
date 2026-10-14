use super::*;

use acme_kit::text;

/// Where a timeline preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelinePreferenceStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TimelinePreferenceStatus {
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

impl std::fmt::Display for TimelinePreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A timeline preference as the timeline team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelinePreference {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub tenant_id: String,
    pub fingerprint: String,
    pub active: bool,
    pub principal: String,
    pub quantity: i64,
    pub cluster: String,
    pub updated_at: i64,
    pub status: TimelinePreferenceStatus,
}

impl TimelinePreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelinePreferenceStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelinePreferenceStatus) -> bool {
        match self.status {
            TimelinePreferenceStatus::Open => {
                return next == TimelinePreferenceStatus::Resolved;
            }
            TimelinePreferenceStatus::InReview => {
                match next {
                    TimelinePreferenceStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelinePreferenceStatus::Resolved => {
                match next {
                    TimelinePreferenceStatus::Reopened | TimelinePreferenceStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelinePreferenceStatus::Reopened => {
                match next {
                    TimelinePreferenceStatus::Archived => {
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

    pub fn move_to(&mut self, next: TimelinePreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline preference {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.quantity as f64) * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelinePreference {
        TimelinePreference {
            code: self.code.clone(),
            title: self.title.clone(),
            audience: self.audience.clone(),
            tenant_id: self.tenant_id.clone(),
            fingerprint: self.fingerprint.clone(),
            active: self.active,
            principal: self.principal.clone(),
            quantity: self.quantity,
            cluster: self.cluster.clone(),
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a timeline preference in status open.
pub fn new_timeline_preference(code: &str, title: &str) -> TimelinePreference {
    TimelinePreference {
        code: code.to_string(),
        title: title.to_string(),
        audience: "".to_string(),
        tenant_id: "".to_string(),
        fingerprint: "".to_string(),
        active: false,
        principal: "".to_string(),
        quantity: 0,
        cluster: "".to_string(),
        updated_at: 0,
        status: TimelinePreferenceStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_timeline_preference("code-481", "title-786");
        assert_eq!(item.status, TimelinePreferenceStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_timeline_preference("code-481", "title-786");
        if item.move_to(TimelinePreferenceStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TimelinePreferenceStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_timeline_preference("code-481", "title-786");
        item.status = TimelinePreferenceStatus::Archived;
        assert!(
            item.move_to(TimelinePreferenceStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
