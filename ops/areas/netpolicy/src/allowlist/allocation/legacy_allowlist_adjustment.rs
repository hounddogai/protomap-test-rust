use super::*;

use acme_kit::text;

/// Where a legacy allowlist adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyAllowlistAdjustmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LegacyAllowlistAdjustmentStatus {
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

impl std::fmt::Display for LegacyAllowlistAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy allowlist adjustment as the allowlist team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyAllowlistAdjustment {
    pub code: String,
    pub name: String,
    pub scope: String,
    pub retries: i64,
    pub ends_at: i64,
    pub status: LegacyAllowlistAdjustmentStatus,
}

impl LegacyAllowlistAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyAllowlistAdjustmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyAllowlistAdjustmentStatus) -> bool {
        match self.status {
            LegacyAllowlistAdjustmentStatus::Draft => {
                match next {
                    LegacyAllowlistAdjustmentStatus::Pending | LegacyAllowlistAdjustmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyAllowlistAdjustmentStatus::Pending => {
                return next == LegacyAllowlistAdjustmentStatus::Active;
            }
            LegacyAllowlistAdjustmentStatus::Active => {
                match next {
                    LegacyAllowlistAdjustmentStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyAllowlistAdjustmentStatus::Suspended => {
                return next == LegacyAllowlistAdjustmentStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyAllowlistAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy allowlist adjustment {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.ends_at as f64 - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyAllowlistAdjustment {
        LegacyAllowlistAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            scope: self.scope.clone(),
            retries: self.retries,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a legacy allowlist adjustment in status draft.
pub fn new_legacy_allowlist_adjustment(code: &str, name: &str) -> LegacyAllowlistAdjustment {
    LegacyAllowlistAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        scope: "".to_string(),
        retries: 0,
        ends_at: 0,
        status: LegacyAllowlistAdjustmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_legacy_allowlist_adjustment("code-353", "name-732");
        assert_eq!(item.status, LegacyAllowlistAdjustmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_legacy_allowlist_adjustment("code-353", "name-732");
        if item.move_to(LegacyAllowlistAdjustmentStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyAllowlistAdjustmentStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_legacy_allowlist_adjustment("code-353", "name-732");
        item.status = LegacyAllowlistAdjustmentStatus::Closed;
        assert!(
            item.move_to(LegacyAllowlistAdjustmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
