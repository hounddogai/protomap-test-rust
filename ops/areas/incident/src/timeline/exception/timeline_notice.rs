use super::*;

use acme_kit::text;

/// Where a timeline notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineNoticeStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TimelineNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TimelineNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a timeline notice and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineNotice {
    pub code: String,
    pub label: String,
    pub ttl_seconds: i64,
    pub name: String,
    pub margin: f64,
    pub active: bool,
    pub status: TimelineNoticeStatus,
}

impl TimelineNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineNoticeStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineNoticeStatus) -> bool {
        match self.status {
            TimelineNoticeStatus::Created => {
                match next {
                    TimelineNoticeStatus::Validated | TimelineNoticeStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineNoticeStatus::Validated => {
                return next == TimelineNoticeStatus::Published || next == TimelineNoticeStatus::Retired;
            }
            TimelineNoticeStatus::Published => {
                match next {
                    TimelineNoticeStatus::Retired => {
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

    pub fn move_to(&mut self, next: TimelineNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.ttl_seconds as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelineNotice {
        TimelineNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            ttl_seconds: self.ttl_seconds,
            name: self.name.clone(),
            margin: self.margin,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a timeline notice in status created.
pub fn new_timeline_notice(code: &str, label: &str) -> TimelineNotice {
    TimelineNotice {
        code: code.to_string(),
        label: label.to_string(),
        ttl_seconds: 0,
        name: "".to_string(),
        margin: 0.0,
        active: false,
        status: TimelineNoticeStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_timeline_notice("code-349", "label-172");
        assert_eq!(item.status, TimelineNoticeStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_timeline_notice("code-349", "label-172");
        if item.move_to(TimelineNoticeStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TimelineNoticeStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_timeline_notice("code-349", "label-172");
        item.status = TimelineNoticeStatus::Retired;
        assert!(
            item.move_to(TimelineNoticeStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
