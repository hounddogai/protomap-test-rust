use super::*;

use acme_kit::text;

/// Where a pending nonce claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingNonceClaimStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PendingNonceClaimStatus {
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

impl std::fmt::Display for PendingNonceClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending nonce claim and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceClaim {
    pub code: String,
    pub name: String,
    pub cipher: String,
    pub tenant_id: String,
    pub source: String,
    pub issuer: String,
    pub quantity: i64,
    pub status: PendingNonceClaimStatus,
}

impl PendingNonceClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingNonceClaimStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingNonceClaimStatus) -> bool {
        match self.status {
            PendingNonceClaimStatus::Planned => {
                return next == PendingNonceClaimStatus::Scheduled || next == PendingNonceClaimStatus::Cancelled;
            }
            PendingNonceClaimStatus::Scheduled => {
                return next == PendingNonceClaimStatus::Cancelled;
            }
            PendingNonceClaimStatus::InProgress => {
                match next {
                    PendingNonceClaimStatus::Completed | PendingNonceClaimStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingNonceClaimStatus::Completed => {
                return next == PendingNonceClaimStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingNonceClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending nonce claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> PendingNonceClaim {
        PendingNonceClaim {
            code: self.code.clone(),
            name: self.name.clone(),
            cipher: self.cipher.clone(),
            tenant_id: self.tenant_id.clone(),
            source: self.source.clone(),
            issuer: self.issuer.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a pending nonce claim in status planned.
pub fn new_pending_nonce_claim(code: &str, name: &str) -> PendingNonceClaim {
    PendingNonceClaim {
        code: code.to_string(),
        name: name.to_string(),
        cipher: "".to_string(),
        tenant_id: "".to_string(),
        source: "".to_string(),
        issuer: "".to_string(),
        quantity: 0,
        status: PendingNonceClaimStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_pending_nonce_claim("code-930", "name-938");
        assert_eq!(item.status, PendingNonceClaimStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_pending_nonce_claim("code-930", "name-938");
        if item.move_to(PendingNonceClaimStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingNonceClaimStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_pending_nonce_claim("code-930", "name-938");
        item.status = PendingNonceClaimStatus::Cancelled;
        assert!(
            item.move_to(PendingNonceClaimStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
