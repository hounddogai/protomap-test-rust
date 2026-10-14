use super::*;

use acme_kit::text;

/// Where a tiered usage batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredUsageBatchStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TieredUsageBatchStatus {
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

impl std::fmt::Display for TieredUsageBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tiered usage batch, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredUsageBatch {
    pub code: String,
    pub name: String,
    pub starts_at: i64,
    pub currency: String,
    pub pinned: bool,
    pub tenant_id: String,
    pub status: TieredUsageBatchStatus,
}

impl TieredUsageBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredUsageBatchStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredUsageBatchStatus) -> bool {
        match self.status {
            TieredUsageBatchStatus::Draft => {
                return next == TieredUsageBatchStatus::Closed;
            }
            TieredUsageBatchStatus::Pending => {
                return next == TieredUsageBatchStatus::Suspended;
            }
            TieredUsageBatchStatus::Active => {
                match next {
                    TieredUsageBatchStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredUsageBatchStatus::Suspended => {
                match next {
                    TieredUsageBatchStatus::Closed => {
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

    pub fn move_to(&mut self, next: TieredUsageBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered usage batch {} cannot move from {} to {}",
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
        format!("{} {}", "tiered usage batch", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.starts_at as f64
    }

    pub fn copy(&self) -> TieredUsageBatch {
        TieredUsageBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            starts_at: self.starts_at,
            currency: self.currency.clone(),
            pinned: self.pinned,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a tiered usage batch in status draft.
pub fn new_tiered_usage_batch(code: &str, name: &str) -> TieredUsageBatch {
    TieredUsageBatch {
        code: code.to_string(),
        name: name.to_string(),
        starts_at: 0,
        currency: "".to_string(),
        pinned: false,
        tenant_id: "".to_string(),
        status: TieredUsageBatchStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_tiered_usage_batch("code-273", "name-727");
        assert_eq!(item.status, TieredUsageBatchStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_tiered_usage_batch("code-273", "name-727");
        if item.move_to(TieredUsageBatchStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredUsageBatchStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_tiered_usage_batch("code-273", "name-727");
        item.status = TieredUsageBatchStatus::Closed;
        assert!(
            item.move_to(TieredUsageBatchStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
