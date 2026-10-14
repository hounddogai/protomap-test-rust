use super::*;

use acme_kit::text;

/// Where a scheduled segment item is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledSegmentItemStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ScheduledSegmentItemStatus {
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

impl std::fmt::Display for ScheduledSegmentItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled segment item, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSegmentItem {
    pub code: String,
    pub name: String,
    pub issuer: String,
    pub created_at: i64,
    pub principal: String,
    pub host: String,
    pub fingerprint: String,
    pub archived: bool,
    pub margin: f64,
    pub summary: String,
    pub revision: i64,
    pub status: ScheduledSegmentItemStatus,
}

impl ScheduledSegmentItem {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledSegmentItemStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledSegmentItemStatus) -> bool {
        match self.status {
            ScheduledSegmentItemStatus::Planned => {
                match next {
                    ScheduledSegmentItemStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledSegmentItemStatus::Scheduled => {
                return next == ScheduledSegmentItemStatus::Cancelled;
            }
            ScheduledSegmentItemStatus::InProgress => {
                match next {
                    ScheduledSegmentItemStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledSegmentItemStatus::Completed => {
                match next {
                    ScheduledSegmentItemStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: ScheduledSegmentItemStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled segment item {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled segment item", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.revision as f64) * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledSegmentItem {
        ScheduledSegmentItem {
            code: self.code.clone(),
            name: self.name.clone(),
            issuer: self.issuer.clone(),
            created_at: self.created_at,
            principal: self.principal.clone(),
            host: self.host.clone(),
            fingerprint: self.fingerprint.clone(),
            archived: self.archived,
            margin: self.margin,
            summary: self.summary.clone(),
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a scheduled segment item in status planned.
pub fn new_scheduled_segment_item(code: &str, name: &str) -> ScheduledSegmentItem {
    ScheduledSegmentItem {
        code: code.to_string(),
        name: name.to_string(),
        issuer: "".to_string(),
        created_at: 0,
        principal: "".to_string(),
        host: "".to_string(),
        fingerprint: "".to_string(),
        archived: false,
        margin: 0.0,
        summary: "".to_string(),
        revision: 0,
        status: ScheduledSegmentItemStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_scheduled_segment_item("code-277", "name-520");
        assert_eq!(item.status, ScheduledSegmentItemStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_scheduled_segment_item("code-277", "name-520");
        if item.move_to(ScheduledSegmentItemStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledSegmentItemStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_scheduled_segment_item("code-277", "name-520");
        item.status = ScheduledSegmentItemStatus::Cancelled;
        assert!(
            item.move_to(ScheduledSegmentItemStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
