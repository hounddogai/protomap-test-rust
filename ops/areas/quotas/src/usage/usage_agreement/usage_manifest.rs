use super::*;

use acme_kit::text;

/// Where a usage manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageManifestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl UsageManifestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for UsageManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a usage manifest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageManifest {
    pub code: String,
    pub title: String,
    pub country: String,
    pub revision: i64,
    pub source: String,
    pub rate: f64,
    pub sequence: i64,
    pub label: String,
    pub cluster: String,
    pub updated_at: i64,
    pub version: i64,
    pub status: UsageManifestStatus,
}

impl UsageManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageManifestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageManifestStatus) -> bool {
        match self.status {
            UsageManifestStatus::New => {
                match next {
                    UsageManifestStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageManifestStatus::Queued => {
                return next == UsageManifestStatus::Done || next == UsageManifestStatus::Failed;
            }
            UsageManifestStatus::Running => {
                return next == UsageManifestStatus::Done;
            }
            UsageManifestStatus::Done => {
                match next {
                    UsageManifestStatus::Failed => {
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

    pub fn move_to(&mut self, next: UsageManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.rate - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsageManifest {
        UsageManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            revision: self.revision,
            source: self.source.clone(),
            rate: self.rate,
            sequence: self.sequence,
            label: self.label.clone(),
            cluster: self.cluster.clone(),
            updated_at: self.updated_at,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a usage manifest in status new.
pub fn new_usage_manifest(code: &str, title: &str) -> UsageManifest {
    UsageManifest {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        revision: 0,
        source: "".to_string(),
        rate: 0.0,
        sequence: 0,
        label: "".to_string(),
        cluster: "".to_string(),
        updated_at: 0,
        version: 0,
        status: UsageManifestStatus::New,
    }
}
