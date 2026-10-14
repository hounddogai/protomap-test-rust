use super::*;

use acme_kit::text;

/// Where a default flow log tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultFlowLogTierStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl DefaultFlowLogTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for DefaultFlowLogTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default flow log tier, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFlowLogTier {
    pub code: String,
    pub title: String,
    pub channel: String,
    pub scope: String,
    pub threshold: f64,
    pub segment: String,
    pub status: DefaultFlowLogTierStatus,
}

impl DefaultFlowLogTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultFlowLogTierStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultFlowLogTierStatus) -> bool {
        match self.status {
            DefaultFlowLogTierStatus::Planned => {
                match next {
                    DefaultFlowLogTierStatus::Scheduled | DefaultFlowLogTierStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultFlowLogTierStatus::Scheduled => {
                match next {
                    DefaultFlowLogTierStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultFlowLogTierStatus::InProgress => {
                return next == DefaultFlowLogTierStatus::Completed || next == DefaultFlowLogTierStatus::Cancelled;
            }
            DefaultFlowLogTierStatus::Completed => {
                return next == DefaultFlowLogTierStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultFlowLogTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default flow log tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.threshold
    }

    pub fn copy(&self) -> DefaultFlowLogTier {
        DefaultFlowLogTier {
            code: self.code.clone(),
            title: self.title.clone(),
            channel: self.channel.clone(),
            scope: self.scope.clone(),
            threshold: self.threshold,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a default flow log tier in status planned.
pub fn new_default_flow_log_tier(code: &str, title: &str) -> DefaultFlowLogTier {
    DefaultFlowLogTier {
        code: code.to_string(),
        title: title.to_string(),
        channel: "".to_string(),
        scope: "".to_string(),
        threshold: 0.0,
        segment: "".to_string(),
        status: DefaultFlowLogTierStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_default_flow_log_tier("code-448", "title-193");
        assert_eq!(item.status, DefaultFlowLogTierStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_default_flow_log_tier("code-448", "title-193");
        if item.move_to(DefaultFlowLogTierStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultFlowLogTierStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_default_flow_log_tier("code-448", "title-193");
        item.status = DefaultFlowLogTierStatus::Cancelled;
        assert!(
            item.move_to(DefaultFlowLogTierStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
