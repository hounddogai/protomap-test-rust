use super::*;

use acme_kit::text;

/// Where a isolation summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationSummaryStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl IsolationSummaryStatus {
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

impl std::fmt::Display for IsolationSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A isolation summary as the isolation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationSummary {
    pub code: String,
    pub label: String,
    pub cluster: String,
    pub expires_at: i64,
    pub pinned: bool,
    pub score: f64,
    pub status: IsolationSummaryStatus,
}

impl IsolationSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationSummaryStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationSummaryStatus) -> bool {
        match self.status {
            IsolationSummaryStatus::Idle => {
                return next == IsolationSummaryStatus::Stopped;
            }
            IsolationSummaryStatus::Warming => {
                match next {
                    IsolationSummaryStatus::Ready | IsolationSummaryStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationSummaryStatus::Ready => {
                return next == IsolationSummaryStatus::Draining;
            }
            IsolationSummaryStatus::Draining => {
                match next {
                    IsolationSummaryStatus::Stopped => {
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

    pub fn move_to(&mut self, next: IsolationSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation summary {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.expires_at as f64 + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IsolationSummary {
        IsolationSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            cluster: self.cluster.clone(),
            expires_at: self.expires_at,
            pinned: self.pinned,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a isolation summary in status idle.
pub fn new_isolation_summary(code: &str, label: &str) -> IsolationSummary {
    IsolationSummary {
        code: code.to_string(),
        label: label.to_string(),
        cluster: "".to_string(),
        expires_at: 0,
        pinned: false,
        score: 0.0,
        status: IsolationSummaryStatus::Idle,
    }
}
