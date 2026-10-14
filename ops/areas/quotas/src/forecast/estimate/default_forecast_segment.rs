use super::*;

use acme_kit::text;

/// Where a default forecast segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultForecastSegmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DefaultForecastSegmentStatus {
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

impl std::fmt::Display for DefaultForecastSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default forecast segment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultForecastSegment {
    pub code: String,
    pub title: String,
    pub checksum: String,
    pub created_at: i64,
    pub namespace: String,
    pub updated_at: i64,
    pub status: DefaultForecastSegmentStatus,
}

impl DefaultForecastSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultForecastSegmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultForecastSegmentStatus) -> bool {
        match self.status {
            DefaultForecastSegmentStatus::New => {
                match next {
                    DefaultForecastSegmentStatus::Queued | DefaultForecastSegmentStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultForecastSegmentStatus::Queued => {
                match next {
                    DefaultForecastSegmentStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultForecastSegmentStatus::Running => {
                match next {
                    DefaultForecastSegmentStatus::Done | DefaultForecastSegmentStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultForecastSegmentStatus::Done => {
                return next == DefaultForecastSegmentStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultForecastSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default forecast segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.created_at as f64 - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultForecastSegment {
        DefaultForecastSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            checksum: self.checksum.clone(),
            created_at: self.created_at,
            namespace: self.namespace.clone(),
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a default forecast segment in status new.
pub fn new_default_forecast_segment(code: &str, title: &str) -> DefaultForecastSegment {
    DefaultForecastSegment {
        code: code.to_string(),
        title: title.to_string(),
        checksum: "".to_string(),
        created_at: 0,
        namespace: "".to_string(),
        updated_at: 0,
        status: DefaultForecastSegmentStatus::New,
    }
}
