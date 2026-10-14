use super::*;

use acme_kit::text;

/// Where a fallback trust domain notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackTrustDomainNoticeStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FallbackTrustDomainNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FallbackTrustDomainNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback trust domain notice, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTrustDomainNotice {
    pub code: String,
    pub label: String,
    pub expires_at: i64,
    pub score: f64,
    pub created_at: i64,
    pub revision: i64,
    pub category: String,
    pub maximum: f64,
    pub grace_days: i64,
    pub ratio: f64,
    pub status: FallbackTrustDomainNoticeStatus,
}

impl FallbackTrustDomainNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackTrustDomainNoticeStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackTrustDomainNoticeStatus) -> bool {
        match self.status {
            FallbackTrustDomainNoticeStatus::Created => {
                match next {
                    FallbackTrustDomainNoticeStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackTrustDomainNoticeStatus::Validated => {
                return next == FallbackTrustDomainNoticeStatus::Published
                    || next == FallbackTrustDomainNoticeStatus::Retired;
            }
            FallbackTrustDomainNoticeStatus::Published => {
                match next {
                    FallbackTrustDomainNoticeStatus::Retired => {
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

    pub fn move_to(&mut self, next: FallbackTrustDomainNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback trust domain notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.maximum - (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackTrustDomainNotice {
        FallbackTrustDomainNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            expires_at: self.expires_at,
            score: self.score,
            created_at: self.created_at,
            revision: self.revision,
            category: self.category.clone(),
            maximum: self.maximum,
            grace_days: self.grace_days,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a fallback trust domain notice in status created.
pub fn new_fallback_trust_domain_notice(code: &str, label: &str) -> FallbackTrustDomainNotice {
    FallbackTrustDomainNotice {
        code: code.to_string(),
        label: label.to_string(),
        expires_at: 0,
        score: 0.0,
        created_at: 0,
        revision: 0,
        category: "".to_string(),
        maximum: 0.0,
        grace_days: 0,
        ratio: 0.0,
        status: FallbackTrustDomainNoticeStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_fallback_trust_domain_notice("code-982", "label-262");
        assert_eq!(item.status, FallbackTrustDomainNoticeStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_fallback_trust_domain_notice("code-982", "label-262");
        if item.move_to(FallbackTrustDomainNoticeStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackTrustDomainNoticeStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_fallback_trust_domain_notice("code-982", "label-262");
        item.status = FallbackTrustDomainNoticeStatus::Retired;
        assert!(
            item.move_to(FallbackTrustDomainNoticeStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
