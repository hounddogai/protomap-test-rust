use super::*;

use acme_kit::text;

/// Where a manual evidence draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualEvidenceDraftStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ManualEvidenceDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ManualEvidenceDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual evidence draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceDraft {
    pub code: String,
    pub title: String,
    pub capacity: i64,
    pub duration_minutes: i64,
    pub limit: i64,
    pub locale: String,
    pub expires_at: i64,
    pub visible: bool,
    pub status: ManualEvidenceDraftStatus,
}

impl ManualEvidenceDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualEvidenceDraftStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualEvidenceDraftStatus) -> bool {
        match self.status {
            ManualEvidenceDraftStatus::Idle => {
                match next {
                    ManualEvidenceDraftStatus::Ready | ManualEvidenceDraftStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualEvidenceDraftStatus::Warming => {
                return next == ManualEvidenceDraftStatus::Ready || next == ManualEvidenceDraftStatus::Draining;
            }
            ManualEvidenceDraftStatus::Ready => {
                match next {
                    ManualEvidenceDraftStatus::Draining | ManualEvidenceDraftStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualEvidenceDraftStatus::Draining => {
                return next == ManualEvidenceDraftStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualEvidenceDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual evidence draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.limit as f64 + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualEvidenceDraft {
        ManualEvidenceDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            capacity: self.capacity,
            duration_minutes: self.duration_minutes,
            limit: self.limit,
            locale: self.locale.clone(),
            expires_at: self.expires_at,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a manual evidence draft in status idle.
pub fn new_manual_evidence_draft(code: &str, title: &str) -> ManualEvidenceDraft {
    ManualEvidenceDraft {
        code: code.to_string(),
        title: title.to_string(),
        capacity: 0,
        duration_minutes: 0,
        limit: 0,
        locale: "".to_string(),
        expires_at: 0,
        visible: false,
        status: ManualEvidenceDraftStatus::Idle,
    }
}
