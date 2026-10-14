use super::*;

use acme_kit::text;

/// Where a strict silence preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictSilencePreferenceStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl StrictSilencePreferenceStatus {
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

impl std::fmt::Display for StrictSilencePreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict silence preference and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictSilencePreference {
    pub code: String,
    pub name: String,
    pub version: i64,
    pub revision: i64,
    pub unit: String,
    pub status: StrictSilencePreferenceStatus,
}

impl StrictSilencePreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictSilencePreferenceStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictSilencePreferenceStatus) -> bool {
        match self.status {
            StrictSilencePreferenceStatus::Open => {
                match next {
                    StrictSilencePreferenceStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictSilencePreferenceStatus::InReview => {
                match next {
                    StrictSilencePreferenceStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictSilencePreferenceStatus::Resolved => {
                return next == StrictSilencePreferenceStatus::Archived;
            }
            StrictSilencePreferenceStatus::Reopened => {
                return next == StrictSilencePreferenceStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictSilencePreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict silence preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.version as f64) * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictSilencePreference {
        StrictSilencePreference {
            code: self.code.clone(),
            name: self.name.clone(),
            version: self.version,
            revision: self.revision,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a strict silence preference in status open.
pub fn new_strict_silence_preference(code: &str, name: &str) -> StrictSilencePreference {
    StrictSilencePreference {
        code: code.to_string(),
        name: name.to_string(),
        version: 0,
        revision: 0,
        unit: "".to_string(),
        status: StrictSilencePreferenceStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_strict_silence_preference("code-898", "name-363");
        assert_eq!(item.status, StrictSilencePreferenceStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_strict_silence_preference("code-898", "name-363");
        if item.move_to(StrictSilencePreferenceStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictSilencePreferenceStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_strict_silence_preference("code-898", "name-363");
        item.status = StrictSilencePreferenceStatus::Archived;
        assert!(
            item.move_to(StrictSilencePreferenceStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
