use super::*;

use acme_kit::text;

/// Where a verifier manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierManifestStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl VerifierManifestStatus {
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

impl std::fmt::Display for VerifierManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A verifier manifest as the verifier team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierManifest {
    pub code: String,
    pub label: String,
    pub batch_size: i64,
    pub tags: Vec<String>,
    pub cost: f64,
    pub contact_email: String,
    pub status: VerifierManifestStatus,
}

impl VerifierManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierManifestStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierManifestStatus) -> bool {
        match self.status {
            VerifierManifestStatus::Planned => {
                match next {
                    VerifierManifestStatus::Scheduled | VerifierManifestStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierManifestStatus::Scheduled => {
                match next {
                    VerifierManifestStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierManifestStatus::InProgress => {
                return next == VerifierManifestStatus::Completed;
            }
            VerifierManifestStatus::Completed => {
                return next == VerifierManifestStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: VerifierManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.batch_size as f64 + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierManifest {
        VerifierManifest {
            code: self.code.clone(),
            label: self.label.clone(),
            batch_size: self.batch_size,
            tags: self.tags.clone(),
            cost: self.cost,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a verifier manifest in status planned.
pub fn new_verifier_manifest(code: &str, label: &str) -> VerifierManifest {
    VerifierManifest {
        code: code.to_string(),
        label: label.to_string(),
        batch_size: 0,
        tags: Vec::new(),
        cost: 0.0,
        contact_email: "".to_string(),
        status: VerifierManifestStatus::Planned,
    }
}
