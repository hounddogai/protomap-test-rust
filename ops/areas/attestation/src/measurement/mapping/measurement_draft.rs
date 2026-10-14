use super::*;

use acme_kit::text;

/// Where a measurement draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementDraftStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl MeasurementDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for MeasurementDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A measurement draft as the measurement team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementDraft {
    pub code: String,
    pub title: String,
    pub tags: Vec<String>,
    pub reviewer: String,
    pub key_id: String,
    pub quantity: i64,
    pub segment: String,
    pub pinned: bool,
    pub retries: i64,
    pub batch_size: i64,
    pub host: String,
    pub status: MeasurementDraftStatus,
}

impl MeasurementDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementDraftStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementDraftStatus) -> bool {
        match self.status {
            MeasurementDraftStatus::Proposed => {
                match next {
                    MeasurementDraftStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementDraftStatus::Approved => {
                return next == MeasurementDraftStatus::Rejected;
            }
            MeasurementDraftStatus::Rejected => {
                match next {
                    MeasurementDraftStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: MeasurementDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "measurement draft", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.retries as f64) * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MeasurementDraft {
        MeasurementDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            tags: self.tags.clone(),
            reviewer: self.reviewer.clone(),
            key_id: self.key_id.clone(),
            quantity: self.quantity,
            segment: self.segment.clone(),
            pinned: self.pinned,
            retries: self.retries,
            batch_size: self.batch_size,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a measurement draft in status proposed.
pub fn new_measurement_draft(code: &str, title: &str) -> MeasurementDraft {
    MeasurementDraft {
        code: code.to_string(),
        title: title.to_string(),
        tags: Vec::new(),
        reviewer: "".to_string(),
        key_id: "".to_string(),
        quantity: 0,
        segment: "".to_string(),
        pinned: false,
        retries: 0,
        batch_size: 0,
        host: "".to_string(),
        status: MeasurementDraftStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_measurement_draft("code-438", "title-54");
        assert_eq!(item.status, MeasurementDraftStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_measurement_draft("code-438", "title-54");
        if item.move_to(MeasurementDraftStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MeasurementDraftStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_measurement_draft("code-438", "title-54");
        item.status = MeasurementDraftStatus::Withdrawn;
        assert!(
            item.move_to(MeasurementDraftStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
