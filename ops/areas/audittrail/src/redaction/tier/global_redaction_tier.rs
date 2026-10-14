use super::*;

use acme_kit::text;

/// Where a global redaction tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalRedactionTierStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl GlobalRedactionTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for GlobalRedactionTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global redaction tier. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRedactionTier {
    pub code: String,
    pub title: String,
    pub retries: i64,
    pub ceiling: f64,
    pub ends_at: i64,
    pub floor_value: f64,
    pub checksum: String,
    pub segment: String,
    pub batch_size: i64,
    pub tags: Vec<String>,
    pub updated_at: i64,
    pub status: GlobalRedactionTierStatus,
}

impl GlobalRedactionTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalRedactionTierStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalRedactionTierStatus) -> bool {
        match self.status {
            GlobalRedactionTierStatus::Requested => {
                return next == GlobalRedactionTierStatus::Expired;
            }
            GlobalRedactionTierStatus::Granted => {
                match next {
                    GlobalRedactionTierStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalRedactionTierStatus::Revoked => {
                match next {
                    GlobalRedactionTierStatus::Expired => {
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

    pub fn move_to(&mut self, next: GlobalRedactionTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global redaction tier {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_value(&self) -> f64 {
        ((self.floor_value - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalRedactionTier {
        GlobalRedactionTier {
            code: self.code.clone(),
            title: self.title.clone(),
            retries: self.retries,
            ceiling: self.ceiling,
            ends_at: self.ends_at,
            floor_value: self.floor_value,
            checksum: self.checksum.clone(),
            segment: self.segment.clone(),
            batch_size: self.batch_size,
            tags: self.tags.clone(),
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a global redaction tier in status requested.
pub fn new_global_redaction_tier(code: &str, title: &str) -> GlobalRedactionTier {
    GlobalRedactionTier {
        code: code.to_string(),
        title: title.to_string(),
        retries: 0,
        ceiling: 0.0,
        ends_at: 0,
        floor_value: 0.0,
        checksum: "".to_string(),
        segment: "".to_string(),
        batch_size: 0,
        tags: Vec::new(),
        updated_at: 0,
        status: GlobalRedactionTierStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_global_redaction_tier("code-978", "title-14");
        assert_eq!(item.status, GlobalRedactionTierStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_global_redaction_tier("code-978", "title-14");
        if item.move_to(GlobalRedactionTierStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalRedactionTierStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_global_redaction_tier("code-978", "title-14");
        item.status = GlobalRedactionTierStatus::Expired;
        assert!(
            item.move_to(GlobalRedactionTierStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
