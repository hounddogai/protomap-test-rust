use super::*;

use acme_kit::text;

/// Where a trust domain rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainRollupStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TrustDomainRollupStatus {
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

impl std::fmt::Display for TrustDomainRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A trust domain rollup as the trust domain team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainRollup {
    pub code: String,
    pub label: String,
    pub batch_size: i64,
    pub retries: i64,
    pub margin: f64,
    pub discount: f64,
    pub limit: i64,
    pub namespace: String,
    pub burst: i64,
    pub ttl_seconds: i64,
    pub status: TrustDomainRollupStatus,
}

impl TrustDomainRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainRollupStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainRollupStatus) -> bool {
        match self.status {
            TrustDomainRollupStatus::Draft => {
                return next == TrustDomainRollupStatus::Suspended;
            }
            TrustDomainRollupStatus::Pending => {
                match next {
                    TrustDomainRollupStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainRollupStatus::Active => {
                return next == TrustDomainRollupStatus::Suspended || next == TrustDomainRollupStatus::Closed;
            }
            TrustDomainRollupStatus::Suspended => {
                return next == TrustDomainRollupStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TrustDomainRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.batch_size as f64 - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TrustDomainRollup {
        TrustDomainRollup {
            code: self.code.clone(),
            label: self.label.clone(),
            batch_size: self.batch_size,
            retries: self.retries,
            margin: self.margin,
            discount: self.discount,
            limit: self.limit,
            namespace: self.namespace.clone(),
            burst: self.burst,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a trust domain rollup in status draft.
pub fn new_trust_domain_rollup(code: &str, label: &str) -> TrustDomainRollup {
    TrustDomainRollup {
        code: code.to_string(),
        label: label.to_string(),
        batch_size: 0,
        retries: 0,
        margin: 0.0,
        discount: 0.0,
        limit: 0,
        namespace: "".to_string(),
        burst: 0,
        ttl_seconds: 0,
        status: TrustDomainRollupStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_trust_domain_rollup("code-83", "label-191");
        assert_eq!(item.status, TrustDomainRollupStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_trust_domain_rollup("code-83", "label-191");
        if item.move_to(TrustDomainRollupStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainRollupStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_trust_domain_rollup("code-83", "label-191");
        item.status = TrustDomainRollupStatus::Closed;
        assert!(
            item.move_to(TrustDomainRollupStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
