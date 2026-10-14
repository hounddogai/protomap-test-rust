use super::*;

use acme_kit::text;

/// Where a allowlist cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistCycleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AllowlistCycleStatus {
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

impl std::fmt::Display for AllowlistCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a allowlist cycle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistCycle {
    pub code: String,
    pub title: String,
    pub archived: bool,
    pub ceiling: f64,
    pub revision: i64,
    pub expires_at: i64,
    pub batch_size: i64,
    pub name: String,
    pub burst: i64,
    pub pinned: bool,
    pub status: AllowlistCycleStatus,
}

impl AllowlistCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistCycleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistCycleStatus) -> bool {
        match self.status {
            AllowlistCycleStatus::New => {
                match next {
                    AllowlistCycleStatus::Queued | AllowlistCycleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistCycleStatus::Queued => {
                return next == AllowlistCycleStatus::Failed;
            }
            AllowlistCycleStatus::Running => {
                match next {
                    AllowlistCycleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistCycleStatus::Done => {
                match next {
                    AllowlistCycleStatus::Failed => {
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

    pub fn move_to(&mut self, next: AllowlistCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.batch_size as f64 + (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistCycle {
        AllowlistCycle {
            code: self.code.clone(),
            title: self.title.clone(),
            archived: self.archived,
            ceiling: self.ceiling,
            revision: self.revision,
            expires_at: self.expires_at,
            batch_size: self.batch_size,
            name: self.name.clone(),
            burst: self.burst,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a allowlist cycle in status new.
pub fn new_allowlist_cycle(code: &str, title: &str) -> AllowlistCycle {
    AllowlistCycle {
        code: code.to_string(),
        title: title.to_string(),
        archived: false,
        ceiling: 0.0,
        revision: 0,
        expires_at: 0,
        batch_size: 0,
        name: "".to_string(),
        burst: 0,
        pinned: false,
        status: AllowlistCycleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_allowlist_cycle("code-864", "title-298");
        assert_eq!(item.status, AllowlistCycleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_allowlist_cycle("code-864", "title-298");
        if item.move_to(AllowlistCycleStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AllowlistCycleStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_allowlist_cycle("code-864", "title-298");
        item.status = AllowlistCycleStatus::Failed;
        assert!(
            item.move_to(AllowlistCycleStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
