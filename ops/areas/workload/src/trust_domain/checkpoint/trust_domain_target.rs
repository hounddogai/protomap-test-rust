use super::*;

use acme_kit::text;

/// Where a trust domain target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainTargetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TrustDomainTargetStatus {
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

impl std::fmt::Display for TrustDomainTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A trust domain target as the trust domain team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainTarget {
    pub code: String,
    pub title: String,
    pub active: bool,
    pub sequence: i64,
    pub updated_at: i64,
    pub status: TrustDomainTargetStatus,
}

impl TrustDomainTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainTargetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainTargetStatus) -> bool {
        match self.status {
            TrustDomainTargetStatus::Draft => {
                return next == TrustDomainTargetStatus::Active;
            }
            TrustDomainTargetStatus::Pending => {
                match next {
                    TrustDomainTargetStatus::Active | TrustDomainTargetStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainTargetStatus::Active => {
                return next == TrustDomainTargetStatus::Suspended;
            }
            TrustDomainTargetStatus::Suspended => {
                return next == TrustDomainTargetStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TrustDomainTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.sequence as f64 - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TrustDomainTarget {
        TrustDomainTarget {
            code: self.code.clone(),
            title: self.title.clone(),
            active: self.active,
            sequence: self.sequence,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a trust domain target in status draft.
pub fn new_trust_domain_target(code: &str, title: &str) -> TrustDomainTarget {
    TrustDomainTarget {
        code: code.to_string(),
        title: title.to_string(),
        active: false,
        sequence: 0,
        updated_at: 0,
        status: TrustDomainTargetStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_trust_domain_target("code-136", "title-84");
        assert_eq!(item.status, TrustDomainTargetStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_trust_domain_target("code-136", "title-84");
        if item.move_to(TrustDomainTargetStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainTargetStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_trust_domain_target("code-136", "title-84");
        item.status = TrustDomainTargetStatus::Closed;
        assert!(
            item.move_to(TrustDomainTargetStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
