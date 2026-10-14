use super::*;

use acme_kit::text;

/// Where a pending trust domain exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingTrustDomainExceptionStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PendingTrustDomainExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PendingTrustDomainExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending trust domain exception, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingTrustDomainException {
    pub code: String,
    pub name: String,
    pub cost: f64,
    pub floor_value: f64,
    pub burst: i64,
    pub weight: f64,
    pub status: PendingTrustDomainExceptionStatus,
}

impl PendingTrustDomainException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingTrustDomainExceptionStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingTrustDomainExceptionStatus) -> bool {
        match self.status {
            PendingTrustDomainExceptionStatus::Created => {
                match next {
                    PendingTrustDomainExceptionStatus::Validated | PendingTrustDomainExceptionStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingTrustDomainExceptionStatus::Validated => {
                match next {
                    PendingTrustDomainExceptionStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingTrustDomainExceptionStatus::Published => {
                match next {
                    PendingTrustDomainExceptionStatus::Retired => {
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

    pub fn move_to(&mut self, next: PendingTrustDomainExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending trust domain exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.weight - self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingTrustDomainException {
        PendingTrustDomainException {
            code: self.code.clone(),
            name: self.name.clone(),
            cost: self.cost,
            floor_value: self.floor_value,
            burst: self.burst,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a pending trust domain exception in status created.
pub fn new_pending_trust_domain_exception(code: &str, name: &str) -> PendingTrustDomainException {
    PendingTrustDomainException {
        code: code.to_string(),
        name: name.to_string(),
        cost: 0.0,
        floor_value: 0.0,
        burst: 0,
        weight: 0.0,
        status: PendingTrustDomainExceptionStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_pending_trust_domain_exception("code-141", "name-917");
        assert_eq!(item.status, PendingTrustDomainExceptionStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_pending_trust_domain_exception("code-141", "name-917");
        if item.move_to(PendingTrustDomainExceptionStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingTrustDomainExceptionStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_pending_trust_domain_exception("code-141", "name-917");
        item.status = PendingTrustDomainExceptionStatus::Retired;
        assert!(
            item.move_to(PendingTrustDomainExceptionStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
