use super::*;

use acme_kit::text;

/// Where a segment rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentRollupStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SegmentRollupStatus {
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

impl std::fmt::Display for SegmentRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A segment rollup. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentRollup {
    pub code: String,
    pub title: String,
    pub principal: String,
    pub owner_id: String,
    pub tags: Vec<String>,
    pub score: f64,
    pub status: SegmentRollupStatus,
}

impl SegmentRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentRollupStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentRollupStatus) -> bool {
        match self.status {
            SegmentRollupStatus::Planned => {
                return next == SegmentRollupStatus::InProgress;
            }
            SegmentRollupStatus::Scheduled => {
                match next {
                    SegmentRollupStatus::Completed | SegmentRollupStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentRollupStatus::InProgress => {
                match next {
                    SegmentRollupStatus::Completed | SegmentRollupStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentRollupStatus::Completed => {
                match next {
                    SegmentRollupStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: SegmentRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> SegmentRollup {
        SegmentRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            principal: self.principal.clone(),
            owner_id: self.owner_id.clone(),
            tags: self.tags.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a segment rollup in status planned.
pub fn new_segment_rollup(code: &str, title: &str) -> SegmentRollup {
    SegmentRollup {
        code: code.to_string(),
        title: title.to_string(),
        principal: "".to_string(),
        owner_id: "".to_string(),
        tags: Vec::new(),
        score: 0.0,
        status: SegmentRollupStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_segment_rollup("code-229", "title-993");
        assert_eq!(item.status, SegmentRollupStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_segment_rollup("code-229", "title-993");
        if item.move_to(SegmentRollupStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentRollupStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_segment_rollup("code-229", "title-993");
        item.status = SegmentRollupStatus::Cancelled;
        assert!(
            item.move_to(SegmentRollupStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
