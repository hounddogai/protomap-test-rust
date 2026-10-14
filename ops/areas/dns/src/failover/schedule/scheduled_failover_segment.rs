use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a scheduled failover segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledFailoverSegmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ScheduledFailoverSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ScheduledFailoverSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled failover segment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFailoverSegment {
    pub code: String,
    pub title: String,
    pub score: f64,
    pub window_days: i64,
    pub retries: i64,
    pub timezone: String,
    pub segment: String,
    pub attributes: BTreeMap<String, String>,
    pub phone: String,
    pub status: ScheduledFailoverSegmentStatus,
}

impl ScheduledFailoverSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledFailoverSegmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledFailoverSegmentStatus) -> bool {
        match self.status {
            ScheduledFailoverSegmentStatus::Created => {
                match next {
                    ScheduledFailoverSegmentStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledFailoverSegmentStatus::Validated => {
                return next == ScheduledFailoverSegmentStatus::Published
                    || next == ScheduledFailoverSegmentStatus::Retired;
            }
            ScheduledFailoverSegmentStatus::Published => {
                match next {
                    ScheduledFailoverSegmentStatus::Retired => {
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

    pub fn move_to(&mut self, next: ScheduledFailoverSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled failover segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.window_days as f64 - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledFailoverSegment {
        ScheduledFailoverSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            score: self.score,
            window_days: self.window_days,
            retries: self.retries,
            timezone: self.timezone.clone(),
            segment: self.segment.clone(),
            attributes: self.attributes.clone(),
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a scheduled failover segment in status created.
pub fn new_scheduled_failover_segment(code: &str, title: &str) -> ScheduledFailoverSegment {
    ScheduledFailoverSegment {
        code: code.to_string(),
        title: title.to_string(),
        score: 0.0,
        window_days: 0,
        retries: 0,
        timezone: "".to_string(),
        segment: "".to_string(),
        attributes: BTreeMap::new(),
        phone: "".to_string(),
        status: ScheduledFailoverSegmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_scheduled_failover_segment("code-414", "title-62");
        assert_eq!(item.status, ScheduledFailoverSegmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_scheduled_failover_segment("code-414", "title-62");
        if item.move_to(ScheduledFailoverSegmentStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledFailoverSegmentStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_scheduled_failover_segment("code-414", "title-62");
        item.status = ScheduledFailoverSegmentStatus::Retired;
        assert!(
            item.move_to(ScheduledFailoverSegmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
