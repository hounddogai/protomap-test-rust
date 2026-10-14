use super::*;

use acme_kit::text;

/// Where a regional trust domain notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalTrustDomainNoticeStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionalTrustDomainNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for RegionalTrustDomainNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional trust domain notice. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalTrustDomainNotice {
    pub code: String,
    pub name: String,
    pub label: String,
    pub tenant_id: String,
    pub score: f64,
    pub region: String,
    pub sequence: i64,
    pub minimum: f64,
    pub ttl_seconds: i64,
    pub attempts: i64,
    pub status: RegionalTrustDomainNoticeStatus,
}

impl RegionalTrustDomainNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalTrustDomainNoticeStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalTrustDomainNoticeStatus) -> bool {
        match self.status {
            RegionalTrustDomainNoticeStatus::Idle => {
                return next == RegionalTrustDomainNoticeStatus::Draining;
            }
            RegionalTrustDomainNoticeStatus::Warming => {
                match next {
                    RegionalTrustDomainNoticeStatus::Ready | RegionalTrustDomainNoticeStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalTrustDomainNoticeStatus::Ready => {
                return next == RegionalTrustDomainNoticeStatus::Draining
                    || next == RegionalTrustDomainNoticeStatus::Stopped;
            }
            RegionalTrustDomainNoticeStatus::Draining => {
                match next {
                    RegionalTrustDomainNoticeStatus::Stopped => {
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

    pub fn move_to(&mut self, next: RegionalTrustDomainNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional trust domain notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.score + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalTrustDomainNotice {
        RegionalTrustDomainNotice {
            code: self.code.clone(),
            name: self.name.clone(),
            label: self.label.clone(),
            tenant_id: self.tenant_id.clone(),
            score: self.score,
            region: self.region.clone(),
            sequence: self.sequence,
            minimum: self.minimum,
            ttl_seconds: self.ttl_seconds,
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a regional trust domain notice in status idle.
pub fn new_regional_trust_domain_notice(code: &str, name: &str) -> RegionalTrustDomainNotice {
    RegionalTrustDomainNotice {
        code: code.to_string(),
        name: name.to_string(),
        label: "".to_string(),
        tenant_id: "".to_string(),
        score: 0.0,
        region: "".to_string(),
        sequence: 0,
        minimum: 0.0,
        ttl_seconds: 0,
        attempts: 0,
        status: RegionalTrustDomainNoticeStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_regional_trust_domain_notice("code-931", "name-657");
        assert_eq!(item.status, RegionalTrustDomainNoticeStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_regional_trust_domain_notice("code-931", "name-657");
        if item.move_to(RegionalTrustDomainNoticeStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalTrustDomainNoticeStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_regional_trust_domain_notice("code-931", "name-657");
        item.status = RegionalTrustDomainNoticeStatus::Stopped;
        assert!(
            item.move_to(RegionalTrustDomainNoticeStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
