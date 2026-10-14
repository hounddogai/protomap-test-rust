use super::*;

use acme_kit::text;

/// Where a canary variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryVarianceStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl CanaryVarianceStatus {
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

impl std::fmt::Display for CanaryVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a canary variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryVariance {
    pub code: String,
    pub name: String,
    pub rotation_days: i64,
    pub source: String,
    pub category: String,
    pub revision: i64,
    pub contact_email: String,
    pub version: i64,
    pub label: String,
    pub archived: bool,
    pub limit: i64,
    pub status: CanaryVarianceStatus,
}

impl CanaryVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryVarianceStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryVarianceStatus) -> bool {
        match self.status {
            CanaryVarianceStatus::New => {
                match next {
                    CanaryVarianceStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryVarianceStatus::Queued => {
                return next == CanaryVarianceStatus::Running;
            }
            CanaryVarianceStatus::Running => {
                match next {
                    CanaryVarianceStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryVarianceStatus::Done => {
                match next {
                    CanaryVarianceStatus::Failed => {
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

    pub fn move_to(&mut self, next: CanaryVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.rotation_days as f64 - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanaryVariance {
        CanaryVariance {
            code: self.code.clone(),
            name: self.name.clone(),
            rotation_days: self.rotation_days,
            source: self.source.clone(),
            category: self.category.clone(),
            revision: self.revision,
            contact_email: self.contact_email.clone(),
            version: self.version,
            label: self.label.clone(),
            archived: self.archived,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a canary variance in status new.
pub fn new_canary_variance(code: &str, name: &str) -> CanaryVariance {
    CanaryVariance {
        code: code.to_string(),
        name: name.to_string(),
        rotation_days: 0,
        source: "".to_string(),
        category: "".to_string(),
        revision: 0,
        contact_email: "".to_string(),
        version: 0,
        label: "".to_string(),
        archived: false,
        limit: 0,
        status: CanaryVarianceStatus::New,
    }
}
