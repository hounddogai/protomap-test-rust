use super::*;

use acme_kit::text;

/// Where a adaptive silence budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveSilenceBudgetStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl AdaptiveSilenceBudgetStatus {
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

impl std::fmt::Display for AdaptiveSilenceBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive silence budget. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSilenceBudget {
    pub code: String,
    pub label: String,
    pub source: String,
    pub window_days: i64,
    pub host: String,
    pub cluster: String,
    pub version: i64,
    pub revision: i64,
    pub margin: f64,
    pub duration_minutes: i64,
    pub status: AdaptiveSilenceBudgetStatus,
}

impl AdaptiveSilenceBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveSilenceBudgetStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveSilenceBudgetStatus) -> bool {
        match self.status {
            AdaptiveSilenceBudgetStatus::Idle => {
                return next == AdaptiveSilenceBudgetStatus::Draining;
            }
            AdaptiveSilenceBudgetStatus::Warming => {
                return next == AdaptiveSilenceBudgetStatus::Stopped;
            }
            AdaptiveSilenceBudgetStatus::Ready => {
                return next == AdaptiveSilenceBudgetStatus::Draining || next == AdaptiveSilenceBudgetStatus::Stopped;
            }
            AdaptiveSilenceBudgetStatus::Draining => {
                return next == AdaptiveSilenceBudgetStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveSilenceBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive silence budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.version as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveSilenceBudget {
        AdaptiveSilenceBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            window_days: self.window_days,
            host: self.host.clone(),
            cluster: self.cluster.clone(),
            version: self.version,
            revision: self.revision,
            margin: self.margin,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a adaptive silence budget in status idle.
pub fn new_adaptive_silence_budget(code: &str, label: &str) -> AdaptiveSilenceBudget {
    AdaptiveSilenceBudget {
        code: code.to_string(),
        label: label.to_string(),
        source: "".to_string(),
        window_days: 0,
        host: "".to_string(),
        cluster: "".to_string(),
        version: 0,
        revision: 0,
        margin: 0.0,
        duration_minutes: 0,
        status: AdaptiveSilenceBudgetStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_adaptive_silence_budget("code-260", "label-873");
        assert_eq!(item.status, AdaptiveSilenceBudgetStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_adaptive_silence_budget("code-260", "label-873");
        if item.move_to(AdaptiveSilenceBudgetStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveSilenceBudgetStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_adaptive_silence_budget("code-260", "label-873");
        item.status = AdaptiveSilenceBudgetStatus::Stopped;
        assert!(
            item.move_to(AdaptiveSilenceBudgetStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
