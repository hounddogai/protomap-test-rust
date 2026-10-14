use super::*;

use acme_kit::text;

/// Where a canary ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryLedgerStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl CanaryLedgerStatus {
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

impl std::fmt::Display for CanaryLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One canary ledger, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryLedger {
    pub code: String,
    pub title: String,
    pub amount: f64,
    pub cluster: String,
    pub region: String,
    pub timezone: String,
    pub quota_units: i64,
    pub tags: Vec<String>,
    pub phone: String,
    pub status: CanaryLedgerStatus,
}

impl CanaryLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryLedgerStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryLedgerStatus) -> bool {
        match self.status {
            CanaryLedgerStatus::Draft => {
                match next {
                    CanaryLedgerStatus::Active | CanaryLedgerStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryLedgerStatus::Pending => {
                return next == CanaryLedgerStatus::Active || next == CanaryLedgerStatus::Closed;
            }
            CanaryLedgerStatus::Active => {
                match next {
                    CanaryLedgerStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryLedgerStatus::Suspended => {
                return next == CanaryLedgerStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CanaryLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
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

    pub fn adjusted_load(&self) -> f64 {
        (self.amount * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanaryLedger {
        CanaryLedger {
            code: self.code.clone(),
            title: self.title.clone(),
            amount: self.amount,
            cluster: self.cluster.clone(),
            region: self.region.clone(),
            timezone: self.timezone.clone(),
            quota_units: self.quota_units,
            tags: self.tags.clone(),
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a canary ledger in status draft.
pub fn new_canary_ledger(code: &str, title: &str) -> CanaryLedger {
    CanaryLedger {
        code: code.to_string(),
        title: title.to_string(),
        amount: 0.0,
        cluster: "".to_string(),
        region: "".to_string(),
        timezone: "".to_string(),
        quota_units: 0,
        tags: Vec::new(),
        phone: "".to_string(),
        status: CanaryLedgerStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_canary_ledger("code-654", "title-914");
        assert_eq!(item.status, CanaryLedgerStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_canary_ledger("code-654", "title-914");
        if item.move_to(CanaryLedgerStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryLedgerStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_canary_ledger("code-654", "title-914");
        item.status = CanaryLedgerStatus::Closed;
        assert!(
            item.move_to(CanaryLedgerStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
