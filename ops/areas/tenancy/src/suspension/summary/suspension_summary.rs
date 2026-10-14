use super::*;

use acme_kit::text;

/// Where a suspension summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionSummaryStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SuspensionSummaryStatus {
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

impl std::fmt::Display for SuspensionSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A suspension summary. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionSummary {
    pub code: String,
    pub title: String,
    pub expires_at: i64,
    pub rotation_days: i64,
    pub grace_days: i64,
    pub window_days: i64,
    pub capacity: i64,
    pub status: SuspensionSummaryStatus,
}

impl SuspensionSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionSummaryStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionSummaryStatus) -> bool {
        match self.status {
            SuspensionSummaryStatus::Idle => {
                match next {
                    SuspensionSummaryStatus::Draining | SuspensionSummaryStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionSummaryStatus::Warming => {
                match next {
                    SuspensionSummaryStatus::Ready | SuspensionSummaryStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionSummaryStatus::Ready => {
                match next {
                    SuspensionSummaryStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionSummaryStatus::Draining => {
                return next == SuspensionSummaryStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SuspensionSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension summary {} cannot move from {} to {}",
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
        format!("{} {}", "suspension summary", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.window_days as f64) * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionSummary {
        SuspensionSummary {
            code: self.code.clone(),
            title: self.title.clone(),
            expires_at: self.expires_at,
            rotation_days: self.rotation_days,
            grace_days: self.grace_days,
            window_days: self.window_days,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a suspension summary in status idle.
pub fn new_suspension_summary(code: &str, title: &str) -> SuspensionSummary {
    SuspensionSummary {
        code: code.to_string(),
        title: title.to_string(),
        expires_at: 0,
        rotation_days: 0,
        grace_days: 0,
        window_days: 0,
        capacity: 0,
        status: SuspensionSummaryStatus::Idle,
    }
}
