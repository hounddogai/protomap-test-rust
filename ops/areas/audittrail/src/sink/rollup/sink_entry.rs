use super::*;

use acme_kit::text;

/// Where a sink entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkEntryStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SinkEntryStatus {
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

impl std::fmt::Display for SinkEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A sink entry as the sink team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkEntry {
    pub code: String,
    pub name: String,
    pub rate: f64,
    pub external_ref: String,
    pub fingerprint: String,
    pub expires_at: i64,
    pub archived: bool,
    pub audience: String,
    pub checksum: String,
    pub status: SinkEntryStatus,
}

impl SinkEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkEntryStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkEntryStatus) -> bool {
        match self.status {
            SinkEntryStatus::Draft => {
                return next == SinkEntryStatus::Closed;
            }
            SinkEntryStatus::Pending => {
                return next == SinkEntryStatus::Suspended;
            }
            SinkEntryStatus::Active => {
                return next == SinkEntryStatus::Closed;
            }
            SinkEntryStatus::Suspended => {
                match next {
                    SinkEntryStatus::Closed => {
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

    pub fn move_to(&mut self, next: SinkEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.rate + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SinkEntry {
        SinkEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            rate: self.rate,
            external_ref: self.external_ref.clone(),
            fingerprint: self.fingerprint.clone(),
            expires_at: self.expires_at,
            archived: self.archived,
            audience: self.audience.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a sink entry in status draft.
pub fn new_sink_entry(code: &str, name: &str) -> SinkEntry {
    SinkEntry {
        code: code.to_string(),
        name: name.to_string(),
        rate: 0.0,
        external_ref: "".to_string(),
        fingerprint: "".to_string(),
        expires_at: 0,
        archived: false,
        audience: "".to_string(),
        checksum: "".to_string(),
        status: SinkEntryStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_sink_entry("code-136", "name-747");
        assert_eq!(item.status, SinkEntryStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_sink_entry("code-136", "name-747");
        if item.move_to(SinkEntryStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SinkEntryStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_sink_entry("code-136", "name-747");
        item.status = SinkEntryStatus::Closed;
        assert!(item.move_to(SinkEntryStatus::Draft).is_err(), "expected move to fail");
    }
}
