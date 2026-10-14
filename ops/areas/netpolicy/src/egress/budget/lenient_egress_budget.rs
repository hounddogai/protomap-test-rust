use super::*;

use acme_kit::text;

/// Where a lenient egress budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientEgressBudgetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl LenientEgressBudgetStatus {
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

impl std::fmt::Display for LenientEgressBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient egress budget as the egress team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientEgressBudget {
    pub code: String,
    pub title: String,
    pub margin: f64,
    pub notes: String,
    pub fingerprint: String,
    pub checksum: String,
    pub pinned: bool,
    pub key_id: String,
    pub status: LenientEgressBudgetStatus,
}

impl LenientEgressBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientEgressBudgetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientEgressBudgetStatus) -> bool {
        match self.status {
            LenientEgressBudgetStatus::Draft => {
                return next == LenientEgressBudgetStatus::Active || next == LenientEgressBudgetStatus::Closed;
            }
            LenientEgressBudgetStatus::Pending => {
                match next {
                    LenientEgressBudgetStatus::Active | LenientEgressBudgetStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientEgressBudgetStatus::Active => {
                match next {
                    LenientEgressBudgetStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientEgressBudgetStatus::Suspended => {
                match next {
                    LenientEgressBudgetStatus::Closed => {
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

    pub fn move_to(&mut self, next: LenientEgressBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient egress budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.margin
    }

    pub fn copy(&self) -> LenientEgressBudget {
        LenientEgressBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            margin: self.margin,
            notes: self.notes.clone(),
            fingerprint: self.fingerprint.clone(),
            checksum: self.checksum.clone(),
            pinned: self.pinned,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a lenient egress budget in status draft.
pub fn new_lenient_egress_budget(code: &str, title: &str) -> LenientEgressBudget {
    LenientEgressBudget {
        code: code.to_string(),
        title: title.to_string(),
        margin: 0.0,
        notes: "".to_string(),
        fingerprint: "".to_string(),
        checksum: "".to_string(),
        pinned: false,
        key_id: "".to_string(),
        status: LenientEgressBudgetStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_lenient_egress_budget("code-951", "title-909");
        assert_eq!(item.status, LenientEgressBudgetStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_lenient_egress_budget("code-951", "title-909");
        if item.move_to(LenientEgressBudgetStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientEgressBudgetStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_lenient_egress_budget("code-951", "title-909");
        item.status = LenientEgressBudgetStatus::Closed;
        assert!(
            item.move_to(LenientEgressBudgetStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
