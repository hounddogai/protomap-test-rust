use super::*;

use acme_kit::text;

/// Where a primary runbook entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryRunbookEntryStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PrimaryRunbookEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PrimaryRunbookEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary runbook entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRunbookEntry {
    pub code: String,
    pub title: String,
    pub scope: String,
    pub ends_at: i64,
    pub maximum: f64,
    pub limit: i64,
    pub channel: String,
    pub revision: i64,
    pub port: i64,
    pub issuer: String,
    pub status: PrimaryRunbookEntryStatus,
}

impl PrimaryRunbookEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryRunbookEntryStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryRunbookEntryStatus) -> bool {
        match self.status {
            PrimaryRunbookEntryStatus::Requested => {
                return next == PrimaryRunbookEntryStatus::Revoked || next == PrimaryRunbookEntryStatus::Expired;
            }
            PrimaryRunbookEntryStatus::Granted => {
                return next == PrimaryRunbookEntryStatus::Revoked || next == PrimaryRunbookEntryStatus::Expired;
            }
            PrimaryRunbookEntryStatus::Revoked => {
                match next {
                    PrimaryRunbookEntryStatus::Expired => {
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

    pub fn move_to(&mut self, next: PrimaryRunbookEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary runbook entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.ends_at as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryRunbookEntry {
        PrimaryRunbookEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            scope: self.scope.clone(),
            ends_at: self.ends_at,
            maximum: self.maximum,
            limit: self.limit,
            channel: self.channel.clone(),
            revision: self.revision,
            port: self.port,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a primary runbook entry in status requested.
pub fn new_primary_runbook_entry(code: &str, title: &str) -> PrimaryRunbookEntry {
    PrimaryRunbookEntry {
        code: code.to_string(),
        title: title.to_string(),
        scope: "".to_string(),
        ends_at: 0,
        maximum: 0.0,
        limit: 0,
        channel: "".to_string(),
        revision: 0,
        port: 0,
        issuer: "".to_string(),
        status: PrimaryRunbookEntryStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_primary_runbook_entry("code-168", "title-173");
        assert_eq!(item.status, PrimaryRunbookEntryStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_primary_runbook_entry("code-168", "title-173");
        if item.move_to(PrimaryRunbookEntryStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryRunbookEntryStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_primary_runbook_entry("code-168", "title-173");
        item.status = PrimaryRunbookEntryStatus::Expired;
        assert!(
            item.move_to(PrimaryRunbookEntryStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
