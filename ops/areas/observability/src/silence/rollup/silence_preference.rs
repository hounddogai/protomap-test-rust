use super::*;

use acme_kit::text;

/// Where a silence preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilencePreferenceStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SilencePreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SilencePreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One silence preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SilencePreference {
    pub code: String,
    pub label: String,
    pub burst: i64,
    pub budget: f64,
    pub duration_minutes: i64,
    pub name: String,
    pub status: SilencePreferenceStatus,
}

impl SilencePreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilencePreferenceStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilencePreferenceStatus) -> bool {
        match self.status {
            SilencePreferenceStatus::Proposed => {
                return next == SilencePreferenceStatus::Withdrawn;
            }
            SilencePreferenceStatus::Approved => {
                return next == SilencePreferenceStatus::Rejected;
            }
            SilencePreferenceStatus::Rejected => {
                return next == SilencePreferenceStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SilencePreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "silence preference", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.duration_minutes as f64 + (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SilencePreference {
        SilencePreference {
            code: self.code.clone(),
            label: self.label.clone(),
            burst: self.burst,
            budget: self.budget,
            duration_minutes: self.duration_minutes,
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a silence preference in status proposed.
pub fn new_silence_preference(code: &str, label: &str) -> SilencePreference {
    SilencePreference {
        code: code.to_string(),
        label: label.to_string(),
        burst: 0,
        budget: 0.0,
        duration_minutes: 0,
        name: "".to_string(),
        status: SilencePreferenceStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_silence_preference("code-765", "label-179");
        assert_eq!(item.status, SilencePreferenceStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_silence_preference("code-765", "label-179");
        if item.move_to(SilencePreferenceStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SilencePreferenceStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_silence_preference("code-765", "label-179");
        item.status = SilencePreferenceStatus::Withdrawn;
        assert!(
            item.move_to(SilencePreferenceStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
