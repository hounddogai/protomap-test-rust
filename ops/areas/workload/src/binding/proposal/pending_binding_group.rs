use super::*;

use acme_kit::text;

/// Where a pending binding group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingBindingGroupStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PendingBindingGroupStatus {
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

impl std::fmt::Display for PendingBindingGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending binding group as the binding team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingBindingGroup {
    pub code: String,
    pub name: String,
    pub notes: String,
    pub ceiling: f64,
    pub tags: Vec<String>,
    pub channel: String,
    pub ends_at: i64,
    pub status: PendingBindingGroupStatus,
}

impl PendingBindingGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingBindingGroupStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingBindingGroupStatus) -> bool {
        match self.status {
            PendingBindingGroupStatus::New => {
                return next == PendingBindingGroupStatus::Done;
            }
            PendingBindingGroupStatus::Queued => {
                return next == PendingBindingGroupStatus::Failed;
            }
            PendingBindingGroupStatus::Running => {
                return next == PendingBindingGroupStatus::Failed;
            }
            PendingBindingGroupStatus::Done => {
                match next {
                    PendingBindingGroupStatus::Failed => {
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

    pub fn move_to(&mut self, next: PendingBindingGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending binding group {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.ends_at as f64) * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingBindingGroup {
        PendingBindingGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            notes: self.notes.clone(),
            ceiling: self.ceiling,
            tags: self.tags.clone(),
            channel: self.channel.clone(),
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a pending binding group in status new.
pub fn new_pending_binding_group(code: &str, name: &str) -> PendingBindingGroup {
    PendingBindingGroup {
        code: code.to_string(),
        name: name.to_string(),
        notes: "".to_string(),
        ceiling: 0.0,
        tags: Vec::new(),
        channel: "".to_string(),
        ends_at: 0,
        status: PendingBindingGroupStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_pending_binding_group("code-420", "name-315");
        assert_eq!(item.status, PendingBindingGroupStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_pending_binding_group("code-420", "name-315");
        if item.move_to(PendingBindingGroupStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingBindingGroupStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_pending_binding_group("code-420", "name-315");
        item.status = PendingBindingGroupStatus::Failed;
        assert!(
            item.move_to(PendingBindingGroupStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
