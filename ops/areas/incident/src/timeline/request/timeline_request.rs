use super::*;

use acme_kit::text;

/// Where a timeline request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineRequestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl TimelineRequestStatus {
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

impl std::fmt::Display for TimelineRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A timeline request. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineRequest {
    pub code: String,
    pub title: String,
    pub ends_at: i64,
    pub source: String,
    pub principal: String,
    pub namespace: String,
    pub created_at: i64,
    pub name: String,
    pub pinned: bool,
    pub issuer: String,
    pub tags: Vec<String>,
    pub status: TimelineRequestStatus,
}

impl TimelineRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineRequestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineRequestStatus) -> bool {
        match self.status {
            TimelineRequestStatus::Planned => {
                return next == TimelineRequestStatus::Cancelled;
            }
            TimelineRequestStatus::Scheduled => {
                return next == TimelineRequestStatus::Completed;
            }
            TimelineRequestStatus::InProgress => {
                return next == TimelineRequestStatus::Completed;
            }
            TimelineRequestStatus::Completed => {
                match next {
                    TimelineRequestStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: TimelineRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline request {} cannot move from {} to {}",
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
        format!("{} {}", "timeline request", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_value(&self) -> f64 {
        ((self.ends_at as f64) * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelineRequest {
        TimelineRequest {
            code: self.code.clone(),
            title: self.title.clone(),
            ends_at: self.ends_at,
            source: self.source.clone(),
            principal: self.principal.clone(),
            namespace: self.namespace.clone(),
            created_at: self.created_at,
            name: self.name.clone(),
            pinned: self.pinned,
            issuer: self.issuer.clone(),
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a timeline request in status planned.
pub fn new_timeline_request(code: &str, title: &str) -> TimelineRequest {
    TimelineRequest {
        code: code.to_string(),
        title: title.to_string(),
        ends_at: 0,
        source: "".to_string(),
        principal: "".to_string(),
        namespace: "".to_string(),
        created_at: 0,
        name: "".to_string(),
        pinned: false,
        issuer: "".to_string(),
        tags: Vec::new(),
        status: TimelineRequestStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_timeline_request("code-211", "title-527");
        assert_eq!(item.status, TimelineRequestStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_timeline_request("code-211", "title-527");
        if item.move_to(TimelineRequestStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TimelineRequestStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_timeline_request("code-211", "title-527");
        item.status = TimelineRequestStatus::Cancelled;
        assert!(
            item.move_to(TimelineRequestStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
