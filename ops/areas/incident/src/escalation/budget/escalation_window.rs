use super::*;

use acme_kit::text;

/// Where a escalation window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationWindowStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl EscalationWindowStatus {
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

impl std::fmt::Display for EscalationWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A escalation window as the escalation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationWindow {
    pub code: String,
    pub name: String,
    pub threshold: f64,
    pub pinned: bool,
    pub quota_units: i64,
    pub issuer: String,
    pub ends_at: i64,
    pub archived: bool,
    pub starts_at: i64,
    pub retries: i64,
    pub summary: String,
    pub status: EscalationWindowStatus,
}

impl EscalationWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationWindowStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationWindowStatus) -> bool {
        match self.status {
            EscalationWindowStatus::New => {
                return next == EscalationWindowStatus::Queued;
            }
            EscalationWindowStatus::Queued => {
                match next {
                    EscalationWindowStatus::Running | EscalationWindowStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationWindowStatus::Running => {
                match next {
                    EscalationWindowStatus::Done | EscalationWindowStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationWindowStatus::Done => {
                return next == EscalationWindowStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EscalationWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation window {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.ends_at as f64) * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationWindow {
        EscalationWindow {
            code: self.code.clone(),
            name: self.name.clone(),
            threshold: self.threshold,
            pinned: self.pinned,
            quota_units: self.quota_units,
            issuer: self.issuer.clone(),
            ends_at: self.ends_at,
            archived: self.archived,
            starts_at: self.starts_at,
            retries: self.retries,
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a escalation window in status new.
pub fn new_escalation_window(code: &str, name: &str) -> EscalationWindow {
    EscalationWindow {
        code: code.to_string(),
        name: name.to_string(),
        threshold: 0.0,
        pinned: false,
        quota_units: 0,
        issuer: "".to_string(),
        ends_at: 0,
        archived: false,
        starts_at: 0,
        retries: 0,
        summary: "".to_string(),
        status: EscalationWindowStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_escalation_window("code-42", "name-40");
        assert_eq!(item.status, EscalationWindowStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_escalation_window("code-42", "name-40");
        if item.move_to(EscalationWindowStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationWindowStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_escalation_window("code-42", "name-40");
        item.status = EscalationWindowStatus::Failed;
        assert!(
            item.move_to(EscalationWindowStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
