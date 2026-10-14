use super::*;

use acme_kit::text;

/// Where a adaptive probe summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveProbeSummaryStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl AdaptiveProbeSummaryStatus {
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

impl std::fmt::Display for AdaptiveProbeSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a adaptive probe summary and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveProbeSummary {
    pub code: String,
    pub title: String,
    pub pinned: bool,
    pub sequence: i64,
    pub quota_units: i64,
    pub reviewer: String,
    pub status: AdaptiveProbeSummaryStatus,
}

impl AdaptiveProbeSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveProbeSummaryStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveProbeSummaryStatus) -> bool {
        match self.status {
            AdaptiveProbeSummaryStatus::Idle => {
                match next {
                    AdaptiveProbeSummaryStatus::Ready | AdaptiveProbeSummaryStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveProbeSummaryStatus::Warming => {
                return next == AdaptiveProbeSummaryStatus::Ready || next == AdaptiveProbeSummaryStatus::Draining;
            }
            AdaptiveProbeSummaryStatus::Ready => {
                return next == AdaptiveProbeSummaryStatus::Stopped;
            }
            AdaptiveProbeSummaryStatus::Draining => {
                return next == AdaptiveProbeSummaryStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveProbeSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive probe summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.quota_units as f64 - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveProbeSummary {
        AdaptiveProbeSummary {
            code: self.code.clone(),
            title: self.title.clone(),
            pinned: self.pinned,
            sequence: self.sequence,
            quota_units: self.quota_units,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a adaptive probe summary in status idle.
pub fn new_adaptive_probe_summary(code: &str, title: &str) -> AdaptiveProbeSummary {
    AdaptiveProbeSummary {
        code: code.to_string(),
        title: title.to_string(),
        pinned: false,
        sequence: 0,
        quota_units: 0,
        reviewer: "".to_string(),
        status: AdaptiveProbeSummaryStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_adaptive_probe_summary("code-78", "title-26");
        assert_eq!(item.status, AdaptiveProbeSummaryStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_adaptive_probe_summary("code-78", "title-26");
        if item.move_to(AdaptiveProbeSummaryStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveProbeSummaryStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_adaptive_probe_summary("code-78", "title-26");
        item.status = AdaptiveProbeSummaryStatus::Stopped;
        assert!(
            item.move_to(AdaptiveProbeSummaryStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
