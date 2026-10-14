use super::*;

use acme_kit::text;

/// Where a binding ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingLedgerStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl BindingLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for BindingLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a binding ledger and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingLedger {
    pub code: String,
    pub label: String,
    pub minimum: f64,
    pub title: String,
    pub ttl_seconds: i64,
    pub currency: String,
    pub pinned: bool,
    pub timezone: String,
    pub phone: String,
    pub duration_minutes: i64,
    pub region: String,
    pub status: BindingLedgerStatus,
}

impl BindingLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingLedgerStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingLedgerStatus) -> bool {
        match self.status {
            BindingLedgerStatus::Created => {
                match next {
                    BindingLedgerStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingLedgerStatus::Validated => {
                return next == BindingLedgerStatus::Published || next == BindingLedgerStatus::Retired;
            }
            BindingLedgerStatus::Published => {
                return next == BindingLedgerStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BindingLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "binding ledger", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.minimum + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingLedger {
        BindingLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            minimum: self.minimum,
            title: self.title.clone(),
            ttl_seconds: self.ttl_seconds,
            currency: self.currency.clone(),
            pinned: self.pinned,
            timezone: self.timezone.clone(),
            phone: self.phone.clone(),
            duration_minutes: self.duration_minutes,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a binding ledger in status created.
pub fn new_binding_ledger(code: &str, label: &str) -> BindingLedger {
    BindingLedger {
        code: code.to_string(),
        label: label.to_string(),
        minimum: 0.0,
        title: "".to_string(),
        ttl_seconds: 0,
        currency: "".to_string(),
        pinned: false,
        timezone: "".to_string(),
        phone: "".to_string(),
        duration_minutes: 0,
        region: "".to_string(),
        status: BindingLedgerStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_binding_ledger("code-526", "label-670");
        assert_eq!(item.status, BindingLedgerStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_binding_ledger("code-526", "label-670");
        if item.move_to(BindingLedgerStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BindingLedgerStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_binding_ledger("code-526", "label-670");
        item.status = BindingLedgerStatus::Retired;
        assert!(
            item.move_to(BindingLedgerStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
