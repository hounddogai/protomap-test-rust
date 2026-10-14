use super::*;

use acme_kit::text;

/// Where a global page variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalPageVarianceStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl GlobalPageVarianceStatus {
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

impl std::fmt::Display for GlobalPageVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global page variance and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalPageVariance {
    pub code: String,
    pub name: String,
    pub burst: i64,
    pub sequence: i64,
    pub cost: f64,
    pub expires_at: i64,
    pub retries: i64,
    pub label: String,
    pub status: GlobalPageVarianceStatus,
}

impl GlobalPageVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalPageVarianceStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalPageVarianceStatus) -> bool {
        match self.status {
            GlobalPageVarianceStatus::New => {
                return next == GlobalPageVarianceStatus::Queued;
            }
            GlobalPageVarianceStatus::Queued => {
                match next {
                    GlobalPageVarianceStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalPageVarianceStatus::Running => {
                return next == GlobalPageVarianceStatus::Done || next == GlobalPageVarianceStatus::Failed;
            }
            GlobalPageVarianceStatus::Done => {
                match next {
                    GlobalPageVarianceStatus::Failed => {
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

    pub fn move_to(&mut self, next: GlobalPageVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global page variance {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.expires_at as f64 + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalPageVariance {
        GlobalPageVariance {
            code: self.code.clone(),
            name: self.name.clone(),
            burst: self.burst,
            sequence: self.sequence,
            cost: self.cost,
            expires_at: self.expires_at,
            retries: self.retries,
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a global page variance in status new.
pub fn new_global_page_variance(code: &str, name: &str) -> GlobalPageVariance {
    GlobalPageVariance {
        code: code.to_string(),
        name: name.to_string(),
        burst: 0,
        sequence: 0,
        cost: 0.0,
        expires_at: 0,
        retries: 0,
        label: "".to_string(),
        status: GlobalPageVarianceStatus::New,
    }
}
