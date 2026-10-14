use super::*;

use acme_kit::text;

/// Where a redaction entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionEntryStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RedactionEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for RedactionEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One redaction entry, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionEntry {
    pub code: String,
    pub title: String,
    pub ttl_seconds: i64,
    pub principal: String,
    pub tenant_id: String,
    pub issuer: String,
    pub revision: i64,
    pub burst: i64,
    pub status: RedactionEntryStatus,
}

impl RedactionEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionEntryStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionEntryStatus) -> bool {
        match self.status {
            RedactionEntryStatus::Planned => {
                return next == RedactionEntryStatus::Scheduled || next == RedactionEntryStatus::Cancelled;
            }
            RedactionEntryStatus::Scheduled => {
                match next {
                    RedactionEntryStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionEntryStatus::InProgress => {
                match next {
                    RedactionEntryStatus::Completed | RedactionEntryStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionEntryStatus::Completed => {
                return next == RedactionEntryStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RedactionEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.ttl_seconds as f64 - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionEntry {
        RedactionEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            ttl_seconds: self.ttl_seconds,
            principal: self.principal.clone(),
            tenant_id: self.tenant_id.clone(),
            issuer: self.issuer.clone(),
            revision: self.revision,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a redaction entry in status planned.
pub fn new_redaction_entry(code: &str, title: &str) -> RedactionEntry {
    RedactionEntry {
        code: code.to_string(),
        title: title.to_string(),
        ttl_seconds: 0,
        principal: "".to_string(),
        tenant_id: "".to_string(),
        issuer: "".to_string(),
        revision: 0,
        burst: 0,
        status: RedactionEntryStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_redaction_entry("code-793", "title-356");
        assert_eq!(item.status, RedactionEntryStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_redaction_entry("code-793", "title-356");
        if item.move_to(RedactionEntryStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionEntryStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_redaction_entry("code-793", "title-356");
        item.status = RedactionEntryStatus::Cancelled;
        assert!(
            item.move_to(RedactionEntryStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
