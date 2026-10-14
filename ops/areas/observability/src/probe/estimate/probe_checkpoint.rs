use super::*;

use acme_kit::text;

/// Where a probe checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeCheckpointStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ProbeCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for ProbeCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One probe checkpoint, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeCheckpoint {
    pub code: String,
    pub label: String,
    pub expires_at: i64,
    pub threshold: f64,
    pub visible: bool,
    pub locale: String,
    pub currency: String,
    pub ceiling: f64,
    pub priority: i64,
    pub capacity: i64,
    pub status: ProbeCheckpointStatus,
}

impl ProbeCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeCheckpointStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeCheckpointStatus) -> bool {
        match self.status {
            ProbeCheckpointStatus::Open => {
                return next == ProbeCheckpointStatus::Resolved || next == ProbeCheckpointStatus::Archived;
            }
            ProbeCheckpointStatus::InReview => {
                return next == ProbeCheckpointStatus::Resolved || next == ProbeCheckpointStatus::Reopened;
            }
            ProbeCheckpointStatus::Resolved => {
                match next {
                    ProbeCheckpointStatus::Reopened | ProbeCheckpointStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeCheckpointStatus::Reopened => {
                return next == ProbeCheckpointStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ProbeCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe checkpoint {} cannot move from {} to {}",
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
        format!("{} {}", "probe checkpoint", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        (self.threshold * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeCheckpoint {
        ProbeCheckpoint {
            code: self.code.clone(),
            label: self.label.clone(),
            expires_at: self.expires_at,
            threshold: self.threshold,
            visible: self.visible,
            locale: self.locale.clone(),
            currency: self.currency.clone(),
            ceiling: self.ceiling,
            priority: self.priority,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a probe checkpoint in status open.
pub fn new_probe_checkpoint(code: &str, label: &str) -> ProbeCheckpoint {
    ProbeCheckpoint {
        code: code.to_string(),
        label: label.to_string(),
        expires_at: 0,
        threshold: 0.0,
        visible: false,
        locale: "".to_string(),
        currency: "".to_string(),
        ceiling: 0.0,
        priority: 0,
        capacity: 0,
        status: ProbeCheckpointStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_probe_checkpoint("code-967", "label-64");
        assert_eq!(item.status, ProbeCheckpointStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_probe_checkpoint("code-967", "label-64");
        if item.move_to(ProbeCheckpointStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ProbeCheckpointStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_probe_checkpoint("code-967", "label-64");
        item.status = ProbeCheckpointStatus::Archived;
        assert!(
            item.move_to(ProbeCheckpointStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
