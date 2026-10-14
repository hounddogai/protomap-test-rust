use super::*;

use acme_kit::text;

/// Where a segment draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentDraftStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl SegmentDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for SegmentDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A segment draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentDraft {
    pub code: String,
    pub name: String,
    pub ratio: f64,
    pub discount: f64,
    pub approved: bool,
    pub duration_minutes: i64,
    pub capacity: i64,
    pub weight: f64,
    pub status: SegmentDraftStatus,
}

impl SegmentDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentDraftStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentDraftStatus) -> bool {
        match self.status {
            SegmentDraftStatus::Created => {
                return next == SegmentDraftStatus::Published;
            }
            SegmentDraftStatus::Validated => {
                match next {
                    SegmentDraftStatus::Published | SegmentDraftStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentDraftStatus::Published => {
                match next {
                    SegmentDraftStatus::Retired => {
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

    pub fn move_to(&mut self, next: SegmentDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment draft {} cannot move from {} to {}",
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
        ((self.weight + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SegmentDraft {
        SegmentDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            ratio: self.ratio,
            discount: self.discount,
            approved: self.approved,
            duration_minutes: self.duration_minutes,
            capacity: self.capacity,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a segment draft in status created.
pub fn new_segment_draft(code: &str, name: &str) -> SegmentDraft {
    SegmentDraft {
        code: code.to_string(),
        name: name.to_string(),
        ratio: 0.0,
        discount: 0.0,
        approved: false,
        duration_minutes: 0,
        capacity: 0,
        weight: 0.0,
        status: SegmentDraftStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_segment_draft("code-611", "name-876");
        assert_eq!(item.status, SegmentDraftStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_segment_draft("code-611", "name-876");
        if item.move_to(SegmentDraftStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentDraftStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_segment_draft("code-611", "name-876");
        item.status = SegmentDraftStatus::Retired;
        assert!(
            item.move_to(SegmentDraftStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
