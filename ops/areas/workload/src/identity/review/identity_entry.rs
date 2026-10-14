use super::*;

use acme_kit::text;

/// Where a identity entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityEntryStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl IdentityEntryStatus {
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

impl std::fmt::Display for IdentityEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One identity entry, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityEntry {
    pub code: String,
    pub title: String,
    pub external_ref: String,
    pub channel: String,
    pub locale: String,
    pub version: i64,
    pub status: IdentityEntryStatus,
}

impl IdentityEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityEntryStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityEntryStatus) -> bool {
        match self.status {
            IdentityEntryStatus::New => {
                match next {
                    IdentityEntryStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityEntryStatus::Queued => {
                match next {
                    IdentityEntryStatus::Running | IdentityEntryStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityEntryStatus::Running => {
                return next == IdentityEntryStatus::Failed;
            }
            IdentityEntryStatus::Done => {
                match next {
                    IdentityEntryStatus::Failed => {
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

    pub fn move_to(&mut self, next: IdentityEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "identity entry", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.version as f64
    }

    pub fn copy(&self) -> IdentityEntry {
        IdentityEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            external_ref: self.external_ref.clone(),
            channel: self.channel.clone(),
            locale: self.locale.clone(),
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a identity entry in status new.
pub fn new_identity_entry(code: &str, title: &str) -> IdentityEntry {
    IdentityEntry {
        code: code.to_string(),
        title: title.to_string(),
        external_ref: "".to_string(),
        channel: "".to_string(),
        locale: "".to_string(),
        version: 0,
        status: IdentityEntryStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_identity_entry("code-318", "title-962");
        assert_eq!(item.status, IdentityEntryStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_identity_entry("code-318", "title-962");
        if item.move_to(IdentityEntryStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentityEntryStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_identity_entry("code-318", "title-962");
        item.status = IdentityEntryStatus::Failed;
        assert!(item.move_to(IdentityEntryStatus::New).is_err(), "expected move to fail");
    }
}
