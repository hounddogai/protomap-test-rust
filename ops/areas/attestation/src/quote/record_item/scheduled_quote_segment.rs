use super::*;

use acme_kit::text;

/// Where a scheduled quote segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledQuoteSegmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ScheduledQuoteSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for ScheduledQuoteSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled quote segment as the quote team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQuoteSegment {
    pub code: String,
    pub name: String,
    pub floor_value: f64,
    pub contact_email: String,
    pub window_days: i64,
    pub status: ScheduledQuoteSegmentStatus,
}

impl ScheduledQuoteSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledQuoteSegmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledQuoteSegmentStatus) -> bool {
        match self.status {
            ScheduledQuoteSegmentStatus::New => {
                return next == ScheduledQuoteSegmentStatus::Done;
            }
            ScheduledQuoteSegmentStatus::Queued => {
                return next == ScheduledQuoteSegmentStatus::Running;
            }
            ScheduledQuoteSegmentStatus::Running => {
                return next == ScheduledQuoteSegmentStatus::Done || next == ScheduledQuoteSegmentStatus::Failed;
            }
            ScheduledQuoteSegmentStatus::Done => {
                match next {
                    ScheduledQuoteSegmentStatus::Failed => {
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

    pub fn move_to(&mut self, next: ScheduledQuoteSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled quote segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        (self.floor_value * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledQuoteSegment {
        ScheduledQuoteSegment {
            code: self.code.clone(),
            name: self.name.clone(),
            floor_value: self.floor_value,
            contact_email: self.contact_email.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a scheduled quote segment in status new.
pub fn new_scheduled_quote_segment(code: &str, name: &str) -> ScheduledQuoteSegment {
    ScheduledQuoteSegment {
        code: code.to_string(),
        name: name.to_string(),
        floor_value: 0.0,
        contact_email: "".to_string(),
        window_days: 0,
        status: ScheduledQuoteSegmentStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_scheduled_quote_segment("code-381", "name-819");
        assert_eq!(item.status, ScheduledQuoteSegmentStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_scheduled_quote_segment("code-381", "name-819");
        if item.move_to(ScheduledQuoteSegmentStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledQuoteSegmentStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_scheduled_quote_segment("code-381", "name-819");
        item.status = ScheduledQuoteSegmentStatus::Failed;
        assert!(
            item.move_to(ScheduledQuoteSegmentStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
