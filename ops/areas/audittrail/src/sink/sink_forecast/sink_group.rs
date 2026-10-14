use super::*;

use acme_kit::text;

/// Where a sink group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkGroupStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SinkGroupStatus {
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

impl std::fmt::Display for SinkGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A sink group as the sink team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkGroup {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub source: String,
    pub priority: i64,
    pub pinned: bool,
    pub status: SinkGroupStatus,
}

impl SinkGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkGroupStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkGroupStatus) -> bool {
        match self.status {
            SinkGroupStatus::New => {
                return next == SinkGroupStatus::Done;
            }
            SinkGroupStatus::Queued => {
                return next == SinkGroupStatus::Running || next == SinkGroupStatus::Failed;
            }
            SinkGroupStatus::Running => {
                match next {
                    SinkGroupStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SinkGroupStatus::Done => {
                return next == SinkGroupStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SinkGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.priority as f64
    }

    pub fn copy(&self) -> SinkGroup {
        SinkGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            tenant_id: self.tenant_id.clone(),
            source: self.source.clone(),
            priority: self.priority,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a sink group in status new.
pub fn new_sink_group(code: &str, name: &str) -> SinkGroup {
    SinkGroup {
        code: code.to_string(),
        name: name.to_string(),
        tenant_id: "".to_string(),
        source: "".to_string(),
        priority: 0,
        pinned: false,
        status: SinkGroupStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_sink_group("code-489", "name-873");
        assert_eq!(item.status, SinkGroupStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_sink_group("code-489", "name-873");
        if item.move_to(SinkGroupStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SinkGroupStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_sink_group("code-489", "name-873");
        item.status = SinkGroupStatus::Failed;
        assert!(item.move_to(SinkGroupStatus::New).is_err(), "expected move to fail");
    }
}
