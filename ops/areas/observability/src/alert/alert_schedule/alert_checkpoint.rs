use super::*;

use acme_kit::text;

/// Where a alert checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertCheckpointStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AlertCheckpointStatus {
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

impl std::fmt::Display for AlertCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One alert checkpoint, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertCheckpoint {
    pub code: String,
    pub label: String,
    pub namespace: String,
    pub discount: f64,
    pub archived: bool,
    pub capacity: i64,
    pub minimum: f64,
    pub cipher: String,
    pub quota_units: i64,
    pub status: AlertCheckpointStatus,
}

impl AlertCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertCheckpointStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertCheckpointStatus) -> bool {
        match self.status {
            AlertCheckpointStatus::Open => {
                match next {
                    AlertCheckpointStatus::Resolved | AlertCheckpointStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertCheckpointStatus::InReview => {
                match next {
                    AlertCheckpointStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertCheckpointStatus::Resolved => {
                match next {
                    AlertCheckpointStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertCheckpointStatus::Reopened => {
                match next {
                    AlertCheckpointStatus::Archived => {
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

    pub fn move_to(&mut self, next: AlertCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.minimum + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertCheckpoint {
        AlertCheckpoint {
            code: self.code.clone(),
            label: self.label.clone(),
            namespace: self.namespace.clone(),
            discount: self.discount,
            archived: self.archived,
            capacity: self.capacity,
            minimum: self.minimum,
            cipher: self.cipher.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a alert checkpoint in status open.
pub fn new_alert_checkpoint(code: &str, label: &str) -> AlertCheckpoint {
    AlertCheckpoint {
        code: code.to_string(),
        label: label.to_string(),
        namespace: "".to_string(),
        discount: 0.0,
        archived: false,
        capacity: 0,
        minimum: 0.0,
        cipher: "".to_string(),
        quota_units: 0,
        status: AlertCheckpointStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_alert_checkpoint("code-694", "label-708");
        assert_eq!(item.status, AlertCheckpointStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_alert_checkpoint("code-694", "label-708");
        if item.move_to(AlertCheckpointStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AlertCheckpointStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_alert_checkpoint("code-694", "label-708");
        item.status = AlertCheckpointStatus::Archived;
        assert!(
            item.move_to(AlertCheckpointStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
