use super::*;

use acme_kit::text;

/// Where a retention segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RetentionSegmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RetentionSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RetentionSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One retention segment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RetentionSegment {
    pub code: String,
    pub label: String,
    pub minimum: f64,
    pub key_id: String,
    pub margin: f64,
    pub status: RetentionSegmentStatus,
}

impl RetentionSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RetentionSegmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RetentionSegmentStatus) -> bool {
        match self.status {
            RetentionSegmentStatus::Proposed => {
                return next == RetentionSegmentStatus::Approved;
            }
            RetentionSegmentStatus::Approved => {
                match next {
                    RetentionSegmentStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RetentionSegmentStatus::Rejected => {
                match next {
                    RetentionSegmentStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RetentionSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "retention segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "retention segment", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        (self.minimum * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RetentionSegment {
        RetentionSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            minimum: self.minimum,
            key_id: self.key_id.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a retention segment in status proposed.
pub fn new_retention_segment(code: &str, label: &str) -> RetentionSegment {
    RetentionSegment {
        code: code.to_string(),
        label: label.to_string(),
        minimum: 0.0,
        key_id: "".to_string(),
        margin: 0.0,
        status: RetentionSegmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_retention_segment("code-271", "label-461");
        assert_eq!(item.status, RetentionSegmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_retention_segment("code-271", "label-461");
        if item.move_to(RetentionSegmentStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RetentionSegmentStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_retention_segment("code-271", "label-461");
        item.status = RetentionSegmentStatus::Withdrawn;
        assert!(
            item.move_to(RetentionSegmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
