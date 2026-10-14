use super::*;

use acme_kit::text;

/// Where a manual silence score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualSilenceScoreStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ManualSilenceScoreStatus {
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

impl std::fmt::Display for ManualSilenceScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual silence score. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceScore {
    pub code: String,
    pub title: String,
    pub quota_units: i64,
    pub cipher: String,
    pub checksum: String,
    pub status: ManualSilenceScoreStatus,
}

impl ManualSilenceScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualSilenceScoreStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualSilenceScoreStatus) -> bool {
        match self.status {
            ManualSilenceScoreStatus::Open => {
                match next {
                    ManualSilenceScoreStatus::InReview | ManualSilenceScoreStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualSilenceScoreStatus::InReview => {
                return next == ManualSilenceScoreStatus::Archived;
            }
            ManualSilenceScoreStatus::Resolved => {
                return next == ManualSilenceScoreStatus::Reopened;
            }
            ManualSilenceScoreStatus::Reopened => {
                return next == ManualSilenceScoreStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualSilenceScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual silence score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.quota_units as f64
    }

    pub fn copy(&self) -> ManualSilenceScore {
        ManualSilenceScore {
            code: self.code.clone(),
            title: self.title.clone(),
            quota_units: self.quota_units,
            cipher: self.cipher.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a manual silence score in status open.
pub fn new_manual_silence_score(code: &str, title: &str) -> ManualSilenceScore {
    ManualSilenceScore {
        code: code.to_string(),
        title: title.to_string(),
        quota_units: 0,
        cipher: "".to_string(),
        checksum: "".to_string(),
        status: ManualSilenceScoreStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_manual_silence_score("code-762", "title-977");
        assert_eq!(item.status, ManualSilenceScoreStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_manual_silence_score("code-762", "title-977");
        if item.move_to(ManualSilenceScoreStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualSilenceScoreStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_manual_silence_score("code-762", "title-977");
        item.status = ManualSilenceScoreStatus::Archived;
        assert!(
            item.move_to(ManualSilenceScoreStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
