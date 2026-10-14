use super::*;

use acme_kit::text;

/// Where a weighted segment draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedSegmentDraftStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl WeightedSegmentDraftStatus {
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

impl std::fmt::Display for WeightedSegmentDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One weighted segment draft, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSegmentDraft {
    pub code: String,
    pub label: String,
    pub maximum: f64,
    pub weight: f64,
    pub retries: i64,
    pub tags: Vec<String>,
    pub currency: String,
    pub updated_at: i64,
    pub status: WeightedSegmentDraftStatus,
}

impl WeightedSegmentDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedSegmentDraftStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedSegmentDraftStatus) -> bool {
        match self.status {
            WeightedSegmentDraftStatus::Planned => {
                return next == WeightedSegmentDraftStatus::Completed || next == WeightedSegmentDraftStatus::Cancelled;
            }
            WeightedSegmentDraftStatus::Scheduled => {
                return next == WeightedSegmentDraftStatus::Completed || next == WeightedSegmentDraftStatus::Cancelled;
            }
            WeightedSegmentDraftStatus::InProgress => {
                return next == WeightedSegmentDraftStatus::Completed || next == WeightedSegmentDraftStatus::Cancelled;
            }
            WeightedSegmentDraftStatus::Completed => {
                match next {
                    WeightedSegmentDraftStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: WeightedSegmentDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted segment draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.weight + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedSegmentDraft {
        WeightedSegmentDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            maximum: self.maximum,
            weight: self.weight,
            retries: self.retries,
            tags: self.tags.clone(),
            currency: self.currency.clone(),
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a weighted segment draft in status planned.
pub fn new_weighted_segment_draft(code: &str, label: &str) -> WeightedSegmentDraft {
    WeightedSegmentDraft {
        code: code.to_string(),
        label: label.to_string(),
        maximum: 0.0,
        weight: 0.0,
        retries: 0,
        tags: Vec::new(),
        currency: "".to_string(),
        updated_at: 0,
        status: WeightedSegmentDraftStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_weighted_segment_draft("code-776", "label-816");
        assert_eq!(item.status, WeightedSegmentDraftStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_weighted_segment_draft("code-776", "label-816");
        if item.move_to(WeightedSegmentDraftStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedSegmentDraftStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_weighted_segment_draft("code-776", "label-816");
        item.status = WeightedSegmentDraftStatus::Cancelled;
        assert!(
            item.move_to(WeightedSegmentDraftStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
