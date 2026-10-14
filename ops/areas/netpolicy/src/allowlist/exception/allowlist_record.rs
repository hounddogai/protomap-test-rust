use super::*;

use acme_kit::text;

/// Where a allowlist record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistRecordStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AllowlistRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AllowlistRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A allowlist record as the allowlist team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistRecord {
    pub code: String,
    pub name: String,
    pub priority: i64,
    pub maximum: f64,
    pub cipher: String,
    pub archived: bool,
    pub channel: String,
    pub starts_at: i64,
    pub locale: String,
    pub status: AllowlistRecordStatus,
}

impl AllowlistRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistRecordStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistRecordStatus) -> bool {
        match self.status {
            AllowlistRecordStatus::Proposed => {
                return next == AllowlistRecordStatus::Rejected;
            }
            AllowlistRecordStatus::Approved => {
                return next == AllowlistRecordStatus::Rejected || next == AllowlistRecordStatus::Withdrawn;
            }
            AllowlistRecordStatus::Rejected => {
                return next == AllowlistRecordStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AllowlistRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_load(&self) -> f64 {
        (self.maximum * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistRecord {
        AllowlistRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            priority: self.priority,
            maximum: self.maximum,
            cipher: self.cipher.clone(),
            archived: self.archived,
            channel: self.channel.clone(),
            starts_at: self.starts_at,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a allowlist record in status proposed.
pub fn new_allowlist_record(code: &str, name: &str) -> AllowlistRecord {
    AllowlistRecord {
        code: code.to_string(),
        name: name.to_string(),
        priority: 0,
        maximum: 0.0,
        cipher: "".to_string(),
        archived: false,
        channel: "".to_string(),
        starts_at: 0,
        locale: "".to_string(),
        status: AllowlistRecordStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_allowlist_record("code-295", "name-487");
        assert_eq!(item.status, AllowlistRecordStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_allowlist_record("code-295", "name-487");
        if item.move_to(AllowlistRecordStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AllowlistRecordStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_allowlist_record("code-295", "name-487");
        item.status = AllowlistRecordStatus::Withdrawn;
        assert!(
            item.move_to(AllowlistRecordStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
