use super::*;

use acme_kit::text;

/// Where a weighted silence entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedSilenceEntryStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl WeightedSilenceEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for WeightedSilenceEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted silence entry as the silence team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedSilenceEntry {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub quantity: i64,
    pub tags: Vec<String>,
    pub title: String,
    pub threshold: f64,
    pub key_id: String,
    pub status: WeightedSilenceEntryStatus,
}

impl WeightedSilenceEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedSilenceEntryStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedSilenceEntryStatus) -> bool {
        match self.status {
            WeightedSilenceEntryStatus::Proposed => {
                match next {
                    WeightedSilenceEntryStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedSilenceEntryStatus::Approved => {
                return next == WeightedSilenceEntryStatus::Withdrawn;
            }
            WeightedSilenceEntryStatus::Rejected => {
                return next == WeightedSilenceEntryStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedSilenceEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted silence entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "weighted silence entry", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.quantity as f64 + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedSilenceEntry {
        WeightedSilenceEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            summary: self.summary.clone(),
            quantity: self.quantity,
            tags: self.tags.clone(),
            title: self.title.clone(),
            threshold: self.threshold,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a weighted silence entry in status proposed.
pub fn new_weighted_silence_entry(code: &str, label: &str) -> WeightedSilenceEntry {
    WeightedSilenceEntry {
        code: code.to_string(),
        label: label.to_string(),
        summary: "".to_string(),
        quantity: 0,
        tags: Vec::new(),
        title: "".to_string(),
        threshold: 0.0,
        key_id: "".to_string(),
        status: WeightedSilenceEntryStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_weighted_silence_entry("code-984", "label-539");
        assert_eq!(item.status, WeightedSilenceEntryStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_weighted_silence_entry("code-984", "label-539");
        if item.move_to(WeightedSilenceEntryStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedSilenceEntryStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_weighted_silence_entry("code-984", "label-539");
        item.status = WeightedSilenceEntryStatus::Withdrawn;
        assert!(
            item.move_to(WeightedSilenceEntryStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
