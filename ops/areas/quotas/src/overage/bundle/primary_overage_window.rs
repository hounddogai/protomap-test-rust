use super::*;

use acme_kit::text;

/// Where a primary overage window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryOverageWindowStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PrimaryOverageWindowStatus {
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

impl std::fmt::Display for PrimaryOverageWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary overage window, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryOverageWindow {
    pub code: String,
    pub title: String,
    pub minimum: f64,
    pub description: String,
    pub cipher: String,
    pub fingerprint: String,
    pub rotation_days: i64,
    pub external_ref: String,
    pub cost: f64,
    pub source: String,
    pub status: PrimaryOverageWindowStatus,
}

impl PrimaryOverageWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryOverageWindowStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryOverageWindowStatus) -> bool {
        match self.status {
            PrimaryOverageWindowStatus::Planned => {
                return next == PrimaryOverageWindowStatus::Scheduled || next == PrimaryOverageWindowStatus::InProgress;
            }
            PrimaryOverageWindowStatus::Scheduled => {
                match next {
                    PrimaryOverageWindowStatus::InProgress | PrimaryOverageWindowStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryOverageWindowStatus::InProgress => {
                return next == PrimaryOverageWindowStatus::Cancelled;
            }
            PrimaryOverageWindowStatus::Completed => {
                return next == PrimaryOverageWindowStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryOverageWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary overage window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_volume(&self) -> f64 {
        (self.cost * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryOverageWindow {
        PrimaryOverageWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            minimum: self.minimum,
            description: self.description.clone(),
            cipher: self.cipher.clone(),
            fingerprint: self.fingerprint.clone(),
            rotation_days: self.rotation_days,
            external_ref: self.external_ref.clone(),
            cost: self.cost,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a primary overage window in status planned.
pub fn new_primary_overage_window(code: &str, title: &str) -> PrimaryOverageWindow {
    PrimaryOverageWindow {
        code: code.to_string(),
        title: title.to_string(),
        minimum: 0.0,
        description: "".to_string(),
        cipher: "".to_string(),
        fingerprint: "".to_string(),
        rotation_days: 0,
        external_ref: "".to_string(),
        cost: 0.0,
        source: "".to_string(),
        status: PrimaryOverageWindowStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_primary_overage_window("code-72", "title-655");
        assert_eq!(item.status, PrimaryOverageWindowStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_primary_overage_window("code-72", "title-655");
        if item.move_to(PrimaryOverageWindowStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryOverageWindowStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_primary_overage_window("code-72", "title-655");
        item.status = PrimaryOverageWindowStatus::Cancelled;
        assert!(
            item.move_to(PrimaryOverageWindowStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
