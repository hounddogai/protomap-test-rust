use super::*;

use acme_kit::text;

/// Where a tiered freeze group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredFreezeGroupStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TieredFreezeGroupStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TieredFreezeGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered freeze group and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredFreezeGroup {
    pub code: String,
    pub name: String,
    pub revision: i64,
    pub archived: bool,
    pub expires_at: i64,
    pub channel: String,
    pub status: TieredFreezeGroupStatus,
}

impl TieredFreezeGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredFreezeGroupStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredFreezeGroupStatus) -> bool {
        match self.status {
            TieredFreezeGroupStatus::Proposed => {
                return next == TieredFreezeGroupStatus::Approved || next == TieredFreezeGroupStatus::Withdrawn;
            }
            TieredFreezeGroupStatus::Approved => {
                match next {
                    TieredFreezeGroupStatus::Rejected | TieredFreezeGroupStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredFreezeGroupStatus::Rejected => {
                match next {
                    TieredFreezeGroupStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: TieredFreezeGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered freeze group {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.revision as f64 + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredFreezeGroup {
        TieredFreezeGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            revision: self.revision,
            archived: self.archived,
            expires_at: self.expires_at,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a tiered freeze group in status proposed.
pub fn new_tiered_freeze_group(code: &str, name: &str) -> TieredFreezeGroup {
    TieredFreezeGroup {
        code: code.to_string(),
        name: name.to_string(),
        revision: 0,
        archived: false,
        expires_at: 0,
        channel: "".to_string(),
        status: TieredFreezeGroupStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_tiered_freeze_group("code-772", "name-395");
        assert_eq!(item.status, TieredFreezeGroupStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_tiered_freeze_group("code-772", "name-395");
        if item.move_to(TieredFreezeGroupStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredFreezeGroupStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_tiered_freeze_group("code-772", "name-395");
        item.status = TieredFreezeGroupStatus::Withdrawn;
        assert!(
            item.move_to(TieredFreezeGroupStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
