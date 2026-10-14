use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a fallback segment draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackSegmentDraftStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FallbackSegmentDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FallbackSegmentDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback segment draft as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSegmentDraft {
    pub code: String,
    pub label: String,
    pub rate: f64,
    pub attributes: BTreeMap<String, String>,
    pub contact_email: String,
    pub unit: String,
    pub budget: f64,
    pub currency: String,
    pub notes: String,
    pub maximum: f64,
    pub status: FallbackSegmentDraftStatus,
}

impl FallbackSegmentDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackSegmentDraftStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackSegmentDraftStatus) -> bool {
        match self.status {
            FallbackSegmentDraftStatus::Created => {
                return next == FallbackSegmentDraftStatus::Validated || next == FallbackSegmentDraftStatus::Published;
            }
            FallbackSegmentDraftStatus::Validated => {
                match next {
                    FallbackSegmentDraftStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackSegmentDraftStatus::Published => {
                return next == FallbackSegmentDraftStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackSegmentDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback segment draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "fallback segment draft", self.code)
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

    pub fn weighted_value(&self) -> f64 {
        (self.budget * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackSegmentDraft {
        FallbackSegmentDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            rate: self.rate,
            attributes: self.attributes.clone(),
            contact_email: self.contact_email.clone(),
            unit: self.unit.clone(),
            budget: self.budget,
            currency: self.currency.clone(),
            notes: self.notes.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a fallback segment draft in status created.
pub fn new_fallback_segment_draft(code: &str, label: &str) -> FallbackSegmentDraft {
    FallbackSegmentDraft {
        code: code.to_string(),
        label: label.to_string(),
        rate: 0.0,
        attributes: BTreeMap::new(),
        contact_email: "".to_string(),
        unit: "".to_string(),
        budget: 0.0,
        currency: "".to_string(),
        notes: "".to_string(),
        maximum: 0.0,
        status: FallbackSegmentDraftStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_fallback_segment_draft("code-507", "label-122");
        assert_eq!(item.status, FallbackSegmentDraftStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_fallback_segment_draft("code-507", "label-122");
        if item.move_to(FallbackSegmentDraftStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackSegmentDraftStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_fallback_segment_draft("code-507", "label-122");
        item.status = FallbackSegmentDraftStatus::Retired;
        assert!(
            item.move_to(FallbackSegmentDraftStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
