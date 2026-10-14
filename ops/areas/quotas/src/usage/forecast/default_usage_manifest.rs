use super::*;

use acme_kit::text;

/// Where a default usage manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultUsageManifestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DefaultUsageManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DefaultUsageManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default usage manifest as the usage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultUsageManifest {
    pub code: String,
    pub name: String,
    pub source: String,
    pub fingerprint: String,
    pub approved: bool,
    pub maximum: f64,
    pub score: f64,
    pub channel: String,
    pub status: DefaultUsageManifestStatus,
}

impl DefaultUsageManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultUsageManifestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultUsageManifestStatus) -> bool {
        match self.status {
            DefaultUsageManifestStatus::Proposed => {
                match next {
                    DefaultUsageManifestStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultUsageManifestStatus::Approved => {
                match next {
                    DefaultUsageManifestStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultUsageManifestStatus::Rejected => {
                match next {
                    DefaultUsageManifestStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: DefaultUsageManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default usage manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "default usage manifest", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.maximum + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultUsageManifest {
        DefaultUsageManifest {
            code: self.code.clone(),
            name: self.name.clone(),
            source: self.source.clone(),
            fingerprint: self.fingerprint.clone(),
            approved: self.approved,
            maximum: self.maximum,
            score: self.score,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a default usage manifest in status proposed.
pub fn new_default_usage_manifest(code: &str, name: &str) -> DefaultUsageManifest {
    DefaultUsageManifest {
        code: code.to_string(),
        name: name.to_string(),
        source: "".to_string(),
        fingerprint: "".to_string(),
        approved: false,
        maximum: 0.0,
        score: 0.0,
        channel: "".to_string(),
        status: DefaultUsageManifestStatus::Proposed,
    }
}
