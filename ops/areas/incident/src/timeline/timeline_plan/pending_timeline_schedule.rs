use super::*;

use acme_kit::text;

/// Where a pending timeline schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingTimelineScheduleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PendingTimelineScheduleStatus {
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

impl std::fmt::Display for PendingTimelineScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending timeline schedule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingTimelineSchedule {
    pub code: String,
    pub name: String,
    pub weight: f64,
    pub maximum: f64,
    pub namespace: String,
    pub timezone: String,
    pub updated_at: i64,
    pub label: String,
    pub margin: f64,
    pub status: PendingTimelineScheduleStatus,
}

impl PendingTimelineSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingTimelineScheduleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingTimelineScheduleStatus) -> bool {
        match self.status {
            PendingTimelineScheduleStatus::Idle => {
                return next == PendingTimelineScheduleStatus::Warming
                    || next == PendingTimelineScheduleStatus::Draining;
            }
            PendingTimelineScheduleStatus::Warming => {
                return next == PendingTimelineScheduleStatus::Ready;
            }
            PendingTimelineScheduleStatus::Ready => {
                return next == PendingTimelineScheduleStatus::Draining;
            }
            PendingTimelineScheduleStatus::Draining => {
                return next == PendingTimelineScheduleStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingTimelineScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending timeline schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_load(&self) -> f64 {
        ((self.updated_at as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingTimelineSchedule {
        PendingTimelineSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            weight: self.weight,
            maximum: self.maximum,
            namespace: self.namespace.clone(),
            timezone: self.timezone.clone(),
            updated_at: self.updated_at,
            label: self.label.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a pending timeline schedule in status idle.
pub fn new_pending_timeline_schedule(code: &str, name: &str) -> PendingTimelineSchedule {
    PendingTimelineSchedule {
        code: code.to_string(),
        name: name.to_string(),
        weight: 0.0,
        maximum: 0.0,
        namespace: "".to_string(),
        timezone: "".to_string(),
        updated_at: 0,
        label: "".to_string(),
        margin: 0.0,
        status: PendingTimelineScheduleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_pending_timeline_schedule("code-351", "name-448");
        assert_eq!(item.status, PendingTimelineScheduleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_pending_timeline_schedule("code-351", "name-448");
        if item.move_to(PendingTimelineScheduleStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingTimelineScheduleStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_pending_timeline_schedule("code-351", "name-448");
        item.status = PendingTimelineScheduleStatus::Stopped;
        assert!(
            item.move_to(PendingTimelineScheduleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
