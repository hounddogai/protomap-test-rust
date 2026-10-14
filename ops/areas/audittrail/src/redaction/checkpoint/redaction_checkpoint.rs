use super::*;

use acme_kit::text;

/// Where a redaction checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionCheckpointStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RedactionCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for RedactionCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A redaction checkpoint as the redaction team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionCheckpoint {
    pub code: String,
    pub label: String,
    pub burst: i64,
    pub visible: bool,
    pub host: String,
    pub port: i64,
    pub maximum: f64,
    pub source: String,
    pub status: RedactionCheckpointStatus,
}

impl RedactionCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionCheckpointStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionCheckpointStatus) -> bool {
        match self.status {
            RedactionCheckpointStatus::Draft => {
                return next == RedactionCheckpointStatus::Suspended || next == RedactionCheckpointStatus::Closed;
            }
            RedactionCheckpointStatus::Pending => {
                match next {
                    RedactionCheckpointStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionCheckpointStatus::Active => {
                match next {
                    RedactionCheckpointStatus::Suspended | RedactionCheckpointStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionCheckpointStatus::Suspended => {
                match next {
                    RedactionCheckpointStatus::Closed => {
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

    pub fn move_to(&mut self, next: RedactionCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.port as f64 - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionCheckpoint {
        RedactionCheckpoint {
            code: self.code.clone(),
            label: self.label.clone(),
            burst: self.burst,
            visible: self.visible,
            host: self.host.clone(),
            port: self.port,
            maximum: self.maximum,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a redaction checkpoint in status draft.
pub fn new_redaction_checkpoint(code: &str, label: &str) -> RedactionCheckpoint {
    RedactionCheckpoint {
        code: code.to_string(),
        label: label.to_string(),
        burst: 0,
        visible: false,
        host: "".to_string(),
        port: 0,
        maximum: 0.0,
        source: "".to_string(),
        status: RedactionCheckpointStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_redaction_checkpoint("code-205", "label-371");
        assert_eq!(item.status, RedactionCheckpointStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_redaction_checkpoint("code-205", "label-371");
        if item.move_to(RedactionCheckpointStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionCheckpointStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_redaction_checkpoint("code-205", "label-371");
        item.status = RedactionCheckpointStatus::Closed;
        assert!(
            item.move_to(RedactionCheckpointStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
