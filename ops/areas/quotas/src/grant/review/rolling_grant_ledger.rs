use super::*;

use acme_kit::text;

/// Where a rolling grant ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingGrantLedgerStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RollingGrantLedgerStatus {
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

impl std::fmt::Display for RollingGrantLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling grant ledger as the grant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingGrantLedger {
    pub code: String,
    pub label: String,
    pub principal: String,
    pub ceiling: f64,
    pub tags: Vec<String>,
    pub ttl_seconds: i64,
    pub quota_units: i64,
    pub locale: String,
    pub weight: f64,
    pub created_at: i64,
    pub timezone: String,
    pub status: RollingGrantLedgerStatus,
}

impl RollingGrantLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingGrantLedgerStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingGrantLedgerStatus) -> bool {
        match self.status {
            RollingGrantLedgerStatus::Planned => {
                return next == RollingGrantLedgerStatus::Completed;
            }
            RollingGrantLedgerStatus::Scheduled => {
                return next == RollingGrantLedgerStatus::Completed || next == RollingGrantLedgerStatus::Cancelled;
            }
            RollingGrantLedgerStatus::InProgress => {
                return next == RollingGrantLedgerStatus::Completed || next == RollingGrantLedgerStatus::Cancelled;
            }
            RollingGrantLedgerStatus::Completed => {
                match next {
                    RollingGrantLedgerStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RollingGrantLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling grant ledger {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.ceiling - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingGrantLedger {
        RollingGrantLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            principal: self.principal.clone(),
            ceiling: self.ceiling,
            tags: self.tags.clone(),
            ttl_seconds: self.ttl_seconds,
            quota_units: self.quota_units,
            locale: self.locale.clone(),
            weight: self.weight,
            created_at: self.created_at,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a rolling grant ledger in status planned.
pub fn new_rolling_grant_ledger(code: &str, label: &str) -> RollingGrantLedger {
    RollingGrantLedger {
        code: code.to_string(),
        label: label.to_string(),
        principal: "".to_string(),
        ceiling: 0.0,
        tags: Vec::new(),
        ttl_seconds: 0,
        quota_units: 0,
        locale: "".to_string(),
        weight: 0.0,
        created_at: 0,
        timezone: "".to_string(),
        status: RollingGrantLedgerStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_rolling_grant_ledger("code-882", "label-595");
        assert_eq!(item.status, RollingGrantLedgerStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_rolling_grant_ledger("code-882", "label-595");
        if item.move_to(RollingGrantLedgerStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingGrantLedgerStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_rolling_grant_ledger("code-882", "label-595");
        item.status = RollingGrantLedgerStatus::Cancelled;
        assert!(
            item.move_to(RollingGrantLedgerStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
