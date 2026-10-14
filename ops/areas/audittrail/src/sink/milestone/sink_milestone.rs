use super::*;

use acme_kit::text;

/// Where a sink milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkMilestoneStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SinkMilestoneStatus {
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

impl std::fmt::Display for SinkMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A sink milestone as the sink team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkMilestone {
    pub code: String,
    pub label: String,
    pub timezone: String,
    pub checksum: String,
    pub phone: String,
    pub cipher: String,
    pub created_at: i64,
    pub rate: f64,
    pub status: SinkMilestoneStatus,
}

impl SinkMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkMilestoneStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkMilestoneStatus) -> bool {
        match self.status {
            SinkMilestoneStatus::New => {
                match next {
                    SinkMilestoneStatus::Running | SinkMilestoneStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkMilestoneStatus::Queued => {
                match next {
                    SinkMilestoneStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkMilestoneStatus::Running => {
                match next {
                    SinkMilestoneStatus::Done | SinkMilestoneStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkMilestoneStatus::Done => {
                match next {
                    SinkMilestoneStatus::Failed => {
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

    pub fn move_to(&mut self, next: SinkMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink milestone {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.created_at as f64) * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SinkMilestone {
        SinkMilestone {
            code: self.code.clone(),
            label: self.label.clone(),
            timezone: self.timezone.clone(),
            checksum: self.checksum.clone(),
            phone: self.phone.clone(),
            cipher: self.cipher.clone(),
            created_at: self.created_at,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a sink milestone in status new.
pub fn new_sink_milestone(code: &str, label: &str) -> SinkMilestone {
    SinkMilestone {
        code: code.to_string(),
        label: label.to_string(),
        timezone: "".to_string(),
        checksum: "".to_string(),
        phone: "".to_string(),
        cipher: "".to_string(),
        created_at: 0,
        rate: 0.0,
        status: SinkMilestoneStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_sink_milestone("code-947", "label-654");
        assert_eq!(item.status, SinkMilestoneStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_sink_milestone("code-947", "label-654");
        if item.move_to(SinkMilestoneStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SinkMilestoneStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_sink_milestone("code-947", "label-654");
        item.status = SinkMilestoneStatus::Failed;
        assert!(item.move_to(SinkMilestoneStatus::New).is_err(), "expected move to fail");
    }
}
