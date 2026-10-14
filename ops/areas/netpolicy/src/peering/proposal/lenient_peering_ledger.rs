use super::*;

use acme_kit::text;

/// Where a lenient peering ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientPeeringLedgerStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LenientPeeringLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for LenientPeeringLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient peering ledger, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientPeeringLedger {
    pub code: String,
    pub title: String,
    pub port: i64,
    pub principal: String,
    pub audience: String,
    pub status: LenientPeeringLedgerStatus,
}

impl LenientPeeringLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientPeeringLedgerStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientPeeringLedgerStatus) -> bool {
        match self.status {
            LenientPeeringLedgerStatus::Open => {
                match next {
                    LenientPeeringLedgerStatus::InReview | LenientPeeringLedgerStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientPeeringLedgerStatus::InReview => {
                match next {
                    LenientPeeringLedgerStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientPeeringLedgerStatus::Resolved => {
                return next == LenientPeeringLedgerStatus::Reopened;
            }
            LenientPeeringLedgerStatus::Reopened => {
                return next == LenientPeeringLedgerStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientPeeringLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient peering ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.port as f64
    }

    pub fn copy(&self) -> LenientPeeringLedger {
        LenientPeeringLedger {
            code: self.code.clone(),
            title: self.title.clone(),
            port: self.port,
            principal: self.principal.clone(),
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a lenient peering ledger in status open.
pub fn new_lenient_peering_ledger(code: &str, title: &str) -> LenientPeeringLedger {
    LenientPeeringLedger {
        code: code.to_string(),
        title: title.to_string(),
        port: 0,
        principal: "".to_string(),
        audience: "".to_string(),
        status: LenientPeeringLedgerStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_lenient_peering_ledger("code-312", "title-776");
        assert_eq!(item.status, LenientPeeringLedgerStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_lenient_peering_ledger("code-312", "title-776");
        if item.move_to(LenientPeeringLedgerStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientPeeringLedgerStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_lenient_peering_ledger("code-312", "title-776");
        item.status = LenientPeeringLedgerStatus::Archived;
        assert!(
            item.move_to(LenientPeeringLedgerStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
