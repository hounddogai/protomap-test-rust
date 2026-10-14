use super::*;

use acme_kit::text;

/// Where a allowlist tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistTierStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AllowlistTierStatus {
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

impl std::fmt::Display for AllowlistTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One allowlist tier, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistTier {
    pub code: String,
    pub label: String,
    pub score: f64,
    pub created_at: i64,
    pub source: String,
    pub maximum: f64,
    pub channel: String,
    pub cipher: String,
    pub status: AllowlistTierStatus,
}

impl AllowlistTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistTierStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistTierStatus) -> bool {
        match self.status {
            AllowlistTierStatus::Draft => {
                return next == AllowlistTierStatus::Pending || next == AllowlistTierStatus::Suspended;
            }
            AllowlistTierStatus::Pending => {
                return next == AllowlistTierStatus::Closed;
            }
            AllowlistTierStatus::Active => {
                return next == AllowlistTierStatus::Suspended || next == AllowlistTierStatus::Closed;
            }
            AllowlistTierStatus::Suspended => {
                return next == AllowlistTierStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AllowlistTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.maximum + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistTier {
        AllowlistTier {
            code: self.code.clone(),
            label: self.label.clone(),
            score: self.score,
            created_at: self.created_at,
            source: self.source.clone(),
            maximum: self.maximum,
            channel: self.channel.clone(),
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a allowlist tier in status draft.
pub fn new_allowlist_tier(code: &str, label: &str) -> AllowlistTier {
    AllowlistTier {
        code: code.to_string(),
        label: label.to_string(),
        score: 0.0,
        created_at: 0,
        source: "".to_string(),
        maximum: 0.0,
        channel: "".to_string(),
        cipher: "".to_string(),
        status: AllowlistTierStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_allowlist_tier("code-753", "label-316");
        assert_eq!(item.status, AllowlistTierStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_allowlist_tier("code-753", "label-316");
        if item.move_to(AllowlistTierStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AllowlistTierStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_allowlist_tier("code-753", "label-316");
        item.status = AllowlistTierStatus::Closed;
        assert!(
            item.move_to(AllowlistTierStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
