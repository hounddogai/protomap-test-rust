use super::*;

use acme_kit::text;

/// Where a probe schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeScheduleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ProbeScheduleStatus {
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

impl std::fmt::Display for ProbeScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a probe schedule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeSchedule {
    pub code: String,
    pub label: String,
    pub grace_days: i64,
    pub tags: Vec<String>,
    pub attempts: i64,
    pub title: String,
    pub limit: i64,
    pub ends_at: i64,
    pub status: ProbeScheduleStatus,
}

impl ProbeSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeScheduleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeScheduleStatus) -> bool {
        match self.status {
            ProbeScheduleStatus::New => {
                return next == ProbeScheduleStatus::Failed;
            }
            ProbeScheduleStatus::Queued => {
                match next {
                    ProbeScheduleStatus::Running | ProbeScheduleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeScheduleStatus::Running => {
                return next == ProbeScheduleStatus::Done || next == ProbeScheduleStatus::Failed;
            }
            ProbeScheduleStatus::Done => {
                match next {
                    ProbeScheduleStatus::Failed => {
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

    pub fn move_to(&mut self, next: ProbeScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_value(&self) -> f64 {
        ((self.limit as f64 + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeSchedule {
        ProbeSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            grace_days: self.grace_days,
            tags: self.tags.clone(),
            attempts: self.attempts,
            title: self.title.clone(),
            limit: self.limit,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a probe schedule in status new.
pub fn new_probe_schedule(code: &str, label: &str) -> ProbeSchedule {
    ProbeSchedule {
        code: code.to_string(),
        label: label.to_string(),
        grace_days: 0,
        tags: Vec::new(),
        attempts: 0,
        title: "".to_string(),
        limit: 0,
        ends_at: 0,
        status: ProbeScheduleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_probe_schedule("code-872", "label-40");
        assert_eq!(item.status, ProbeScheduleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_probe_schedule("code-872", "label-40");
        if item.move_to(ProbeScheduleStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ProbeScheduleStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_probe_schedule("code-872", "label-40");
        item.status = ProbeScheduleStatus::Failed;
        assert!(item.move_to(ProbeScheduleStatus::New).is_err(), "expected move to fail");
    }
}
