use super::*;

use acme_kit::text;

/// Where a suspension entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionEntryStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SuspensionEntryStatus {
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

impl std::fmt::Display for SuspensionEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A suspension entry as the suspension team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionEntry {
    pub code: String,
    pub title: String,
    pub category: String,
    pub port: i64,
    pub duration_minutes: i64,
    pub revision: i64,
    pub cipher: String,
    pub status: SuspensionEntryStatus,
}

impl SuspensionEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionEntryStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionEntryStatus) -> bool {
        match self.status {
            SuspensionEntryStatus::New => {
                return next == SuspensionEntryStatus::Running || next == SuspensionEntryStatus::Done;
            }
            SuspensionEntryStatus::Queued => {
                match next {
                    SuspensionEntryStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionEntryStatus::Running => {
                match next {
                    SuspensionEntryStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionEntryStatus::Done => {
                match next {
                    SuspensionEntryStatus::Failed => {
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

    pub fn move_to(&mut self, next: SuspensionEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.duration_minutes as f64 - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionEntry {
        SuspensionEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            category: self.category.clone(),
            port: self.port,
            duration_minutes: self.duration_minutes,
            revision: self.revision,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a suspension entry in status new.
pub fn new_suspension_entry(code: &str, title: &str) -> SuspensionEntry {
    SuspensionEntry {
        code: code.to_string(),
        title: title.to_string(),
        category: "".to_string(),
        port: 0,
        duration_minutes: 0,
        revision: 0,
        cipher: "".to_string(),
        status: SuspensionEntryStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_suspension_entry("code-800", "title-513");
        assert_eq!(item.status, SuspensionEntryStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_suspension_entry("code-800", "title-513");
        if item.move_to(SuspensionEntryStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SuspensionEntryStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_suspension_entry("code-800", "title-513");
        item.status = SuspensionEntryStatus::Failed;
        assert!(
            item.move_to(SuspensionEntryStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
