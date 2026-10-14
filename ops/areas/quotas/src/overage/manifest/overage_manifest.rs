use super::*;

use acme_kit::text;

/// Where a overage manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageManifestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl OverageManifestStatus {
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

impl std::fmt::Display for OverageManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One overage manifest, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageManifest {
    pub code: String,
    pub label: String,
    pub revision: i64,
    pub threshold: f64,
    pub checksum: String,
    pub status: OverageManifestStatus,
}

impl OverageManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageManifestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageManifestStatus) -> bool {
        match self.status {
            OverageManifestStatus::Open => {
                return next == OverageManifestStatus::Resolved || next == OverageManifestStatus::Reopened;
            }
            OverageManifestStatus::InReview => {
                return next == OverageManifestStatus::Reopened;
            }
            OverageManifestStatus::Resolved => {
                return next == OverageManifestStatus::Reopened || next == OverageManifestStatus::Archived;
            }
            OverageManifestStatus::Reopened => {
                match next {
                    OverageManifestStatus::Archived => {
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

    pub fn move_to(&mut self, next: OverageManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage manifest {} cannot move from {} to {}",
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

    pub fn effective_exposure(&self) -> f64 {
        ((self.revision as f64 - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> OverageManifest {
        OverageManifest {
            code: self.code.clone(),
            label: self.label.clone(),
            revision: self.revision,
            threshold: self.threshold,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a overage manifest in status open.
pub fn new_overage_manifest(code: &str, label: &str) -> OverageManifest {
    OverageManifest {
        code: code.to_string(),
        label: label.to_string(),
        revision: 0,
        threshold: 0.0,
        checksum: "".to_string(),
        status: OverageManifestStatus::Open,
    }
}
