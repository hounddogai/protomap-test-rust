use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a seasonal probe draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalProbeDraftStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SeasonalProbeDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for SeasonalProbeDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal probe draft, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalProbeDraft {
    pub code: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub rate: f64,
    pub rotation_days: i64,
    pub duration_minutes: i64,
    pub status: SeasonalProbeDraftStatus,
}

impl SeasonalProbeDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalProbeDraftStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalProbeDraftStatus) -> bool {
        match self.status {
            SeasonalProbeDraftStatus::Draft => {
                match next {
                    SeasonalProbeDraftStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalProbeDraftStatus::Pending => {
                return next == SeasonalProbeDraftStatus::Active;
            }
            SeasonalProbeDraftStatus::Active => {
                return next == SeasonalProbeDraftStatus::Suspended || next == SeasonalProbeDraftStatus::Closed;
            }
            SeasonalProbeDraftStatus::Suspended => {
                match next {
                    SeasonalProbeDraftStatus::Closed => {
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

    pub fn move_to(&mut self, next: SeasonalProbeDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal probe draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
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

    pub fn adjusted_exposure(&self) -> f64 {
        (self.rate * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalProbeDraft {
        SeasonalProbeDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            rate: self.rate,
            rotation_days: self.rotation_days,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a seasonal probe draft in status draft.
pub fn new_seasonal_probe_draft(code: &str, label: &str) -> SeasonalProbeDraft {
    SeasonalProbeDraft {
        code: code.to_string(),
        label: label.to_string(),
        attributes: BTreeMap::new(),
        rate: 0.0,
        rotation_days: 0,
        duration_minutes: 0,
        status: SeasonalProbeDraftStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_seasonal_probe_draft("code-549", "label-772");
        assert_eq!(item.status, SeasonalProbeDraftStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_seasonal_probe_draft("code-549", "label-772");
        if item.move_to(SeasonalProbeDraftStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalProbeDraftStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_seasonal_probe_draft("code-549", "label-772");
        item.status = SeasonalProbeDraftStatus::Closed;
        assert!(
            item.move_to(SeasonalProbeDraftStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
