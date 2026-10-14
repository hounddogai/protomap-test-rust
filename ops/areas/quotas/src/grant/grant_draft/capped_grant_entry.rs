use super::*;

use acme_kit::text;

/// Where a capped grant entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedGrantEntryStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl CappedGrantEntryStatus {
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

impl std::fmt::Display for CappedGrantEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a capped grant entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedGrantEntry {
    pub code: String,
    pub name: String,
    pub updated_at: i64,
    pub host: String,
    pub cipher: String,
    pub issuer: String,
    pub reviewer: String,
    pub status: CappedGrantEntryStatus,
}

impl CappedGrantEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedGrantEntryStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedGrantEntryStatus) -> bool {
        match self.status {
            CappedGrantEntryStatus::New => {
                match next {
                    CappedGrantEntryStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedGrantEntryStatus::Queued => {
                return next == CappedGrantEntryStatus::Done || next == CappedGrantEntryStatus::Failed;
            }
            CappedGrantEntryStatus::Running => {
                return next == CappedGrantEntryStatus::Done || next == CappedGrantEntryStatus::Failed;
            }
            CappedGrantEntryStatus::Done => {
                return next == CappedGrantEntryStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedGrantEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped grant entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.updated_at as f64
    }

    pub fn copy(&self) -> CappedGrantEntry {
        CappedGrantEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            updated_at: self.updated_at,
            host: self.host.clone(),
            cipher: self.cipher.clone(),
            issuer: self.issuer.clone(),
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a capped grant entry in status new.
pub fn new_capped_grant_entry(code: &str, name: &str) -> CappedGrantEntry {
    CappedGrantEntry {
        code: code.to_string(),
        name: name.to_string(),
        updated_at: 0,
        host: "".to_string(),
        cipher: "".to_string(),
        issuer: "".to_string(),
        reviewer: "".to_string(),
        status: CappedGrantEntryStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_capped_grant_entry("code-930", "name-687");
        assert_eq!(item.status, CappedGrantEntryStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_capped_grant_entry("code-930", "name-687");
        if item.move_to(CappedGrantEntryStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedGrantEntryStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_capped_grant_entry("code-930", "name-687");
        item.status = CappedGrantEntryStatus::Failed;
        assert!(
            item.move_to(CappedGrantEntryStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
