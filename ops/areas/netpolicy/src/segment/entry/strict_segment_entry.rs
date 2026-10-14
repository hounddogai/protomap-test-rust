use super::*;

use acme_kit::text;

/// Where a strict segment entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictSegmentEntryStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl StrictSegmentEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for StrictSegmentEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict segment entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictSegmentEntry {
    pub code: String,
    pub title: String,
    pub approved: bool,
    pub phone: String,
    pub source: String,
    pub archived: bool,
    pub rotation_days: i64,
    pub status: StrictSegmentEntryStatus,
}

impl StrictSegmentEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictSegmentEntryStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictSegmentEntryStatus) -> bool {
        match self.status {
            StrictSegmentEntryStatus::Created => {
                return next == StrictSegmentEntryStatus::Published;
            }
            StrictSegmentEntryStatus::Validated => {
                match next {
                    StrictSegmentEntryStatus::Published | StrictSegmentEntryStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictSegmentEntryStatus::Published => {
                return next == StrictSegmentEntryStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictSegmentEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict segment entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.rotation_days as f64
    }

    pub fn copy(&self) -> StrictSegmentEntry {
        StrictSegmentEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            approved: self.approved,
            phone: self.phone.clone(),
            source: self.source.clone(),
            archived: self.archived,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a strict segment entry in status created.
pub fn new_strict_segment_entry(code: &str, title: &str) -> StrictSegmentEntry {
    StrictSegmentEntry {
        code: code.to_string(),
        title: title.to_string(),
        approved: false,
        phone: "".to_string(),
        source: "".to_string(),
        archived: false,
        rotation_days: 0,
        status: StrictSegmentEntryStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_strict_segment_entry("code-745", "title-896");
        assert_eq!(item.status, StrictSegmentEntryStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_strict_segment_entry("code-745", "title-896");
        if item.move_to(StrictSegmentEntryStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictSegmentEntryStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_strict_segment_entry("code-745", "title-896");
        item.status = StrictSegmentEntryStatus::Retired;
        assert!(
            item.move_to(StrictSegmentEntryStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
