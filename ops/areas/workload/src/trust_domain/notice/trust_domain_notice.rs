use super::*;

use acme_kit::text;

/// Where a trust domain notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainNoticeStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TrustDomainNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TrustDomainNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One trust domain notice, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainNotice {
    pub code: String,
    pub title: String,
    pub ttl_seconds: i64,
    pub currency: String,
    pub cost: f64,
    pub checksum: String,
    pub status: TrustDomainNoticeStatus,
}

impl TrustDomainNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainNoticeStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainNoticeStatus) -> bool {
        match self.status {
            TrustDomainNoticeStatus::Created => {
                return next == TrustDomainNoticeStatus::Validated;
            }
            TrustDomainNoticeStatus::Validated => {
                return next == TrustDomainNoticeStatus::Published || next == TrustDomainNoticeStatus::Retired;
            }
            TrustDomainNoticeStatus::Published => {
                match next {
                    TrustDomainNoticeStatus::Retired => {
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

    pub fn move_to(&mut self, next: TrustDomainNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_value(&self) -> f64 {
        (self.cost * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TrustDomainNotice {
        TrustDomainNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            ttl_seconds: self.ttl_seconds,
            currency: self.currency.clone(),
            cost: self.cost,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a trust domain notice in status created.
pub fn new_trust_domain_notice(code: &str, title: &str) -> TrustDomainNotice {
    TrustDomainNotice {
        code: code.to_string(),
        title: title.to_string(),
        ttl_seconds: 0,
        currency: "".to_string(),
        cost: 0.0,
        checksum: "".to_string(),
        status: TrustDomainNoticeStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_trust_domain_notice("code-357", "title-27");
        assert_eq!(item.status, TrustDomainNoticeStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_trust_domain_notice("code-357", "title-27");
        if item.move_to(TrustDomainNoticeStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainNoticeStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_trust_domain_notice("code-357", "title-27");
        item.status = TrustDomainNoticeStatus::Retired;
        assert!(
            item.move_to(TrustDomainNoticeStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
