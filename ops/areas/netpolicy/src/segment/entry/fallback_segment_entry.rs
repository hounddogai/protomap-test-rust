use super::*;

use acme_kit::text;

/// Where a fallback segment entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackSegmentEntryStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FallbackSegmentEntryStatus {
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

impl std::fmt::Display for FallbackSegmentEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback segment entry, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSegmentEntry {
    pub code: String,
    pub title: String,
    pub version: i64,
    pub approved: bool,
    pub ends_at: i64,
    pub quota_units: i64,
    pub starts_at: i64,
    pub status: FallbackSegmentEntryStatus,
}

impl FallbackSegmentEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackSegmentEntryStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackSegmentEntryStatus) -> bool {
        match self.status {
            FallbackSegmentEntryStatus::New => {
                return next == FallbackSegmentEntryStatus::Running;
            }
            FallbackSegmentEntryStatus::Queued => {
                match next {
                    FallbackSegmentEntryStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackSegmentEntryStatus::Running => {
                return next == FallbackSegmentEntryStatus::Failed;
            }
            FallbackSegmentEntryStatus::Done => {
                return next == FallbackSegmentEntryStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackSegmentEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback segment entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "fallback segment entry", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.version as f64) * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackSegmentEntry {
        FallbackSegmentEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            version: self.version,
            approved: self.approved,
            ends_at: self.ends_at,
            quota_units: self.quota_units,
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a fallback segment entry in status new.
pub fn new_fallback_segment_entry(code: &str, title: &str) -> FallbackSegmentEntry {
    FallbackSegmentEntry {
        code: code.to_string(),
        title: title.to_string(),
        version: 0,
        approved: false,
        ends_at: 0,
        quota_units: 0,
        starts_at: 0,
        status: FallbackSegmentEntryStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_fallback_segment_entry("code-677", "title-413");
        assert_eq!(item.status, FallbackSegmentEntryStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_fallback_segment_entry("code-677", "title-413");
        if item.move_to(FallbackSegmentEntryStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackSegmentEntryStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_fallback_segment_entry("code-677", "title-413");
        item.status = FallbackSegmentEntryStatus::Failed;
        assert!(
            item.move_to(FallbackSegmentEntryStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
