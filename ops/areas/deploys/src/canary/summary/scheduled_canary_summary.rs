use super::*;

use acme_kit::text;

/// Where a scheduled canary summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledCanarySummaryStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ScheduledCanarySummaryStatus {
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

impl std::fmt::Display for ScheduledCanarySummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled canary summary. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCanarySummary {
    pub code: String,
    pub name: String,
    pub notes: String,
    pub ttl_seconds: i64,
    pub principal: String,
    pub external_ref: String,
    pub status: ScheduledCanarySummaryStatus,
}

impl ScheduledCanarySummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledCanarySummaryStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledCanarySummaryStatus) -> bool {
        match self.status {
            ScheduledCanarySummaryStatus::Idle => {
                return next == ScheduledCanarySummaryStatus::Draining;
            }
            ScheduledCanarySummaryStatus::Warming => {
                match next {
                    ScheduledCanarySummaryStatus::Ready | ScheduledCanarySummaryStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledCanarySummaryStatus::Ready => {
                match next {
                    ScheduledCanarySummaryStatus::Draining | ScheduledCanarySummaryStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledCanarySummaryStatus::Draining => {
                match next {
                    ScheduledCanarySummaryStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ScheduledCanarySummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled canary summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.ttl_seconds as f64
    }

    pub fn copy(&self) -> ScheduledCanarySummary {
        ScheduledCanarySummary {
            code: self.code.clone(),
            name: self.name.clone(),
            notes: self.notes.clone(),
            ttl_seconds: self.ttl_seconds,
            principal: self.principal.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a scheduled canary summary in status idle.
pub fn new_scheduled_canary_summary(code: &str, name: &str) -> ScheduledCanarySummary {
    ScheduledCanarySummary {
        code: code.to_string(),
        name: name.to_string(),
        notes: "".to_string(),
        ttl_seconds: 0,
        principal: "".to_string(),
        external_ref: "".to_string(),
        status: ScheduledCanarySummaryStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_scheduled_canary_summary("code-556", "name-67");
        assert_eq!(item.status, ScheduledCanarySummaryStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_scheduled_canary_summary("code-556", "name-67");
        if item.move_to(ScheduledCanarySummaryStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledCanarySummaryStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_scheduled_canary_summary("code-556", "name-67");
        item.status = ScheduledCanarySummaryStatus::Stopped;
        assert!(
            item.move_to(ScheduledCanarySummaryStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
