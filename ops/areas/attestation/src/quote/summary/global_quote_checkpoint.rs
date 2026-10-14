use super::*;

use acme_kit::text;

/// Where a global quote checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalQuoteCheckpointStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl GlobalQuoteCheckpointStatus {
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

impl std::fmt::Display for GlobalQuoteCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global quote checkpoint, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalQuoteCheckpoint {
    pub code: String,
    pub name: String,
    pub label: String,
    pub locale: String,
    pub margin: f64,
    pub active: bool,
    pub version: i64,
    pub tenant_id: String,
    pub status: GlobalQuoteCheckpointStatus,
}

impl GlobalQuoteCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalQuoteCheckpointStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalQuoteCheckpointStatus) -> bool {
        match self.status {
            GlobalQuoteCheckpointStatus::New => {
                match next {
                    GlobalQuoteCheckpointStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalQuoteCheckpointStatus::Queued => {
                return next == GlobalQuoteCheckpointStatus::Done || next == GlobalQuoteCheckpointStatus::Failed;
            }
            GlobalQuoteCheckpointStatus::Running => {
                match next {
                    GlobalQuoteCheckpointStatus::Done | GlobalQuoteCheckpointStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalQuoteCheckpointStatus::Done => {
                match next {
                    GlobalQuoteCheckpointStatus::Failed => {
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

    pub fn move_to(&mut self, next: GlobalQuoteCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global quote checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.margin - (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalQuoteCheckpoint {
        GlobalQuoteCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            label: self.label.clone(),
            locale: self.locale.clone(),
            margin: self.margin,
            active: self.active,
            version: self.version,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a global quote checkpoint in status new.
pub fn new_global_quote_checkpoint(code: &str, name: &str) -> GlobalQuoteCheckpoint {
    GlobalQuoteCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        label: "".to_string(),
        locale: "".to_string(),
        margin: 0.0,
        active: false,
        version: 0,
        tenant_id: "".to_string(),
        status: GlobalQuoteCheckpointStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_global_quote_checkpoint("code-955", "name-208");
        assert_eq!(item.status, GlobalQuoteCheckpointStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_global_quote_checkpoint("code-955", "name-208");
        if item.move_to(GlobalQuoteCheckpointStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalQuoteCheckpointStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_global_quote_checkpoint("code-955", "name-208");
        item.status = GlobalQuoteCheckpointStatus::Failed;
        assert!(
            item.move_to(GlobalQuoteCheckpointStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
