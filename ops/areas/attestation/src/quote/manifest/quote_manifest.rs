use super::*;

use acme_kit::text;

/// Where a quote manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteManifestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl QuoteManifestStatus {
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

impl std::fmt::Display for QuoteManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a quote manifest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteManifest {
    pub code: String,
    pub title: String,
    pub budget: f64,
    pub rotation_days: i64,
    pub scope: String,
    pub name: String,
    pub status: QuoteManifestStatus,
}

impl QuoteManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteManifestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteManifestStatus) -> bool {
        match self.status {
            QuoteManifestStatus::New => {
                match next {
                    QuoteManifestStatus::Queued | QuoteManifestStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteManifestStatus::Queued => {
                return next == QuoteManifestStatus::Done || next == QuoteManifestStatus::Failed;
            }
            QuoteManifestStatus::Running => {
                match next {
                    QuoteManifestStatus::Done | QuoteManifestStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteManifestStatus::Done => {
                match next {
                    QuoteManifestStatus::Failed => {
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

    pub fn move_to(&mut self, next: QuoteManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote manifest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_value(&self) -> f64 {
        (self.budget * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuoteManifest {
        QuoteManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            budget: self.budget,
            rotation_days: self.rotation_days,
            scope: self.scope.clone(),
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a quote manifest in status new.
pub fn new_quote_manifest(code: &str, title: &str) -> QuoteManifest {
    QuoteManifest {
        code: code.to_string(),
        title: title.to_string(),
        budget: 0.0,
        rotation_days: 0,
        scope: "".to_string(),
        name: "".to_string(),
        status: QuoteManifestStatus::New,
    }
}
