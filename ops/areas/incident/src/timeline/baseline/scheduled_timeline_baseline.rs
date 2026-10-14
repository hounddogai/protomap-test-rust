use super::*;

use acme_kit::text;

/// Where a scheduled timeline baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledTimelineBaselineStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ScheduledTimelineBaselineStatus {
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

impl std::fmt::Display for ScheduledTimelineBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled timeline baseline and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTimelineBaseline {
    pub code: String,
    pub label: String,
    pub channel: String,
    pub archived: bool,
    pub source: String,
    pub country: String,
    pub namespace: String,
    pub rotation_days: i64,
    pub status: ScheduledTimelineBaselineStatus,
}

impl ScheduledTimelineBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledTimelineBaselineStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledTimelineBaselineStatus) -> bool {
        match self.status {
            ScheduledTimelineBaselineStatus::Draft => {
                return next == ScheduledTimelineBaselineStatus::Suspended;
            }
            ScheduledTimelineBaselineStatus::Pending => {
                return next == ScheduledTimelineBaselineStatus::Suspended;
            }
            ScheduledTimelineBaselineStatus::Active => {
                return next == ScheduledTimelineBaselineStatus::Suspended
                    || next == ScheduledTimelineBaselineStatus::Closed;
            }
            ScheduledTimelineBaselineStatus::Suspended => {
                return next == ScheduledTimelineBaselineStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledTimelineBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled timeline baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.rotation_days as f64
    }

    pub fn copy(&self) -> ScheduledTimelineBaseline {
        ScheduledTimelineBaseline {
            code: self.code.clone(),
            label: self.label.clone(),
            channel: self.channel.clone(),
            archived: self.archived,
            source: self.source.clone(),
            country: self.country.clone(),
            namespace: self.namespace.clone(),
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a scheduled timeline baseline in status draft.
pub fn new_scheduled_timeline_baseline(code: &str, label: &str) -> ScheduledTimelineBaseline {
    ScheduledTimelineBaseline {
        code: code.to_string(),
        label: label.to_string(),
        channel: "".to_string(),
        archived: false,
        source: "".to_string(),
        country: "".to_string(),
        namespace: "".to_string(),
        rotation_days: 0,
        status: ScheduledTimelineBaselineStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_scheduled_timeline_baseline("code-407", "label-358");
        assert_eq!(item.status, ScheduledTimelineBaselineStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_scheduled_timeline_baseline("code-407", "label-358");
        if item.move_to(ScheduledTimelineBaselineStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledTimelineBaselineStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_scheduled_timeline_baseline("code-407", "label-358");
        item.status = ScheduledTimelineBaselineStatus::Closed;
        assert!(
            item.move_to(ScheduledTimelineBaselineStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
