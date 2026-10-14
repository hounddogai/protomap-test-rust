use super::*;

use acme_kit::text;

/// Where a manual retention variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualRetentionVarianceStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ManualRetentionVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ManualRetentionVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual retention variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRetentionVariance {
    pub code: String,
    pub title: String,
    pub timezone: String,
    pub notes: String,
    pub pinned: bool,
    pub ratio: f64,
    pub cipher: String,
    pub batch_size: i64,
    pub host: String,
    pub priority: i64,
    pub tags: Vec<String>,
    pub status: ManualRetentionVarianceStatus,
}

impl ManualRetentionVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualRetentionVarianceStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualRetentionVarianceStatus) -> bool {
        match self.status {
            ManualRetentionVarianceStatus::Requested => {
                match next {
                    ManualRetentionVarianceStatus::Granted | ManualRetentionVarianceStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualRetentionVarianceStatus::Granted => {
                match next {
                    ManualRetentionVarianceStatus::Revoked | ManualRetentionVarianceStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualRetentionVarianceStatus::Revoked => {
                match next {
                    ManualRetentionVarianceStatus::Expired => {
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

    pub fn move_to(&mut self, next: ManualRetentionVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual retention variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual retention variance", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.ratio - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualRetentionVariance {
        ManualRetentionVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            timezone: self.timezone.clone(),
            notes: self.notes.clone(),
            pinned: self.pinned,
            ratio: self.ratio,
            cipher: self.cipher.clone(),
            batch_size: self.batch_size,
            host: self.host.clone(),
            priority: self.priority,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a manual retention variance in status requested.
pub fn new_manual_retention_variance(code: &str, title: &str) -> ManualRetentionVariance {
    ManualRetentionVariance {
        code: code.to_string(),
        title: title.to_string(),
        timezone: "".to_string(),
        notes: "".to_string(),
        pinned: false,
        ratio: 0.0,
        cipher: "".to_string(),
        batch_size: 0,
        host: "".to_string(),
        priority: 0,
        tags: Vec::new(),
        status: ManualRetentionVarianceStatus::Requested,
    }
}
