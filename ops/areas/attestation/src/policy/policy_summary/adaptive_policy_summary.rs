use super::*;

use acme_kit::text;

/// Where a adaptive policy summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptivePolicySummaryStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl AdaptivePolicySummaryStatus {
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

impl std::fmt::Display for AdaptivePolicySummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive policy summary, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePolicySummary {
    pub code: String,
    pub title: String,
    pub tenant_id: String,
    pub scope: String,
    pub pinned: bool,
    pub maximum: f64,
    pub batch_size: i64,
    pub rate: f64,
    pub retries: i64,
    pub status: AdaptivePolicySummaryStatus,
}

impl AdaptivePolicySummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptivePolicySummaryStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptivePolicySummaryStatus) -> bool {
        match self.status {
            AdaptivePolicySummaryStatus::Idle => {
                return next == AdaptivePolicySummaryStatus::Ready || next == AdaptivePolicySummaryStatus::Stopped;
            }
            AdaptivePolicySummaryStatus::Warming => {
                match next {
                    AdaptivePolicySummaryStatus::Ready | AdaptivePolicySummaryStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptivePolicySummaryStatus::Ready => {
                match next {
                    AdaptivePolicySummaryStatus::Draining | AdaptivePolicySummaryStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptivePolicySummaryStatus::Draining => {
                match next {
                    AdaptivePolicySummaryStatus::Stopped => {
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

    pub fn move_to(&mut self, next: AdaptivePolicySummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive policy summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_load(&self) -> f64 {
        (self.maximum * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptivePolicySummary {
        AdaptivePolicySummary {
            code: self.code.clone(),
            title: self.title.clone(),
            tenant_id: self.tenant_id.clone(),
            scope: self.scope.clone(),
            pinned: self.pinned,
            maximum: self.maximum,
            batch_size: self.batch_size,
            rate: self.rate,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a adaptive policy summary in status idle.
pub fn new_adaptive_policy_summary(code: &str, title: &str) -> AdaptivePolicySummary {
    AdaptivePolicySummary {
        code: code.to_string(),
        title: title.to_string(),
        tenant_id: "".to_string(),
        scope: "".to_string(),
        pinned: false,
        maximum: 0.0,
        batch_size: 0,
        rate: 0.0,
        retries: 0,
        status: AdaptivePolicySummaryStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_adaptive_policy_summary("code-764", "title-631");
        assert_eq!(item.status, AdaptivePolicySummaryStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_adaptive_policy_summary("code-764", "title-631");
        if item.move_to(AdaptivePolicySummaryStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptivePolicySummaryStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_adaptive_policy_summary("code-764", "title-631");
        item.status = AdaptivePolicySummaryStatus::Stopped;
        assert!(
            item.move_to(AdaptivePolicySummaryStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
