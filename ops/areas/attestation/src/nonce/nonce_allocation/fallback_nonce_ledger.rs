use super::*;

use acme_kit::text;

/// Where a fallback nonce ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackNonceLedgerStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FallbackNonceLedgerStatus {
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

impl std::fmt::Display for FallbackNonceLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback nonce ledger. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackNonceLedger {
    pub code: String,
    pub name: String,
    pub description: String,
    pub priority: i64,
    pub region: String,
    pub owner_id: String,
    pub country: String,
    pub grace_days: i64,
    pub label: String,
    pub audience: String,
    pub created_at: i64,
    pub status: FallbackNonceLedgerStatus,
}

impl FallbackNonceLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackNonceLedgerStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackNonceLedgerStatus) -> bool {
        match self.status {
            FallbackNonceLedgerStatus::New => {
                match next {
                    FallbackNonceLedgerStatus::Queued | FallbackNonceLedgerStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackNonceLedgerStatus::Queued => {
                match next {
                    FallbackNonceLedgerStatus::Done | FallbackNonceLedgerStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackNonceLedgerStatus::Running => {
                match next {
                    FallbackNonceLedgerStatus::Done | FallbackNonceLedgerStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackNonceLedgerStatus::Done => {
                match next {
                    FallbackNonceLedgerStatus::Failed => {
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

    pub fn move_to(&mut self, next: FallbackNonceLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback nonce ledger {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.grace_days as f64 + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackNonceLedger {
        FallbackNonceLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            priority: self.priority,
            region: self.region.clone(),
            owner_id: self.owner_id.clone(),
            country: self.country.clone(),
            grace_days: self.grace_days,
            label: self.label.clone(),
            audience: self.audience.clone(),
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a fallback nonce ledger in status new.
pub fn new_fallback_nonce_ledger(code: &str, name: &str) -> FallbackNonceLedger {
    FallbackNonceLedger {
        code: code.to_string(),
        name: name.to_string(),
        description: "".to_string(),
        priority: 0,
        region: "".to_string(),
        owner_id: "".to_string(),
        country: "".to_string(),
        grace_days: 0,
        label: "".to_string(),
        audience: "".to_string(),
        created_at: 0,
        status: FallbackNonceLedgerStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_fallback_nonce_ledger("code-471", "name-438");
        assert_eq!(item.status, FallbackNonceLedgerStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_fallback_nonce_ledger("code-471", "name-438");
        if item.move_to(FallbackNonceLedgerStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackNonceLedgerStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_fallback_nonce_ledger("code-471", "name-438");
        item.status = FallbackNonceLedgerStatus::Failed;
        assert!(
            item.move_to(FallbackNonceLedgerStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
