use super::*;

use acme_kit::text;

/// Where a default silence preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultSilencePreferenceStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DefaultSilencePreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for DefaultSilencePreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default silence preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultSilencePreference {
    pub code: String,
    pub label: String,
    pub revision: i64,
    pub rotation_days: i64,
    pub ratio: f64,
    pub sequence: i64,
    pub ttl_seconds: i64,
    pub description: String,
    pub approved: bool,
    pub budget: f64,
    pub status: DefaultSilencePreferenceStatus,
}

impl DefaultSilencePreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultSilencePreferenceStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultSilencePreferenceStatus) -> bool {
        match self.status {
            DefaultSilencePreferenceStatus::Open => {
                match next {
                    DefaultSilencePreferenceStatus::Resolved | DefaultSilencePreferenceStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultSilencePreferenceStatus::InReview => {
                return next == DefaultSilencePreferenceStatus::Reopened
                    || next == DefaultSilencePreferenceStatus::Archived;
            }
            DefaultSilencePreferenceStatus::Resolved => {
                return next == DefaultSilencePreferenceStatus::Reopened;
            }
            DefaultSilencePreferenceStatus::Reopened => {
                return next == DefaultSilencePreferenceStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultSilencePreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default silence preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.ratio - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultSilencePreference {
        DefaultSilencePreference {
            code: self.code.clone(),
            label: self.label.clone(),
            revision: self.revision,
            rotation_days: self.rotation_days,
            ratio: self.ratio,
            sequence: self.sequence,
            ttl_seconds: self.ttl_seconds,
            description: self.description.clone(),
            approved: self.approved,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a default silence preference in status open.
pub fn new_default_silence_preference(code: &str, label: &str) -> DefaultSilencePreference {
    DefaultSilencePreference {
        code: code.to_string(),
        label: label.to_string(),
        revision: 0,
        rotation_days: 0,
        ratio: 0.0,
        sequence: 0,
        ttl_seconds: 0,
        description: "".to_string(),
        approved: false,
        budget: 0.0,
        status: DefaultSilencePreferenceStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_default_silence_preference("code-495", "label-969");
        assert_eq!(item.status, DefaultSilencePreferenceStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_default_silence_preference("code-495", "label-969");
        if item.move_to(DefaultSilencePreferenceStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultSilencePreferenceStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_default_silence_preference("code-495", "label-969");
        item.status = DefaultSilencePreferenceStatus::Archived;
        assert!(
            item.move_to(DefaultSilencePreferenceStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
