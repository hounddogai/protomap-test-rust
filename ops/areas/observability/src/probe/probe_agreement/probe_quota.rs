use super::*;

use acme_kit::text;

/// Where a probe quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeQuotaStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ProbeQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ProbeQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A probe quota as the probe team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeQuota {
    pub code: String,
    pub name: String,
    pub grace_days: i64,
    pub ratio: f64,
    pub quantity: i64,
    pub minimum: f64,
    pub status: ProbeQuotaStatus,
}

impl ProbeQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeQuotaStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeQuotaStatus) -> bool {
        match self.status {
            ProbeQuotaStatus::Requested => {
                return next == ProbeQuotaStatus::Revoked;
            }
            ProbeQuotaStatus::Granted => {
                return next == ProbeQuotaStatus::Revoked;
            }
            ProbeQuotaStatus::Revoked => {
                match next {
                    ProbeQuotaStatus::Expired => {
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

    pub fn move_to(&mut self, next: ProbeQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "probe quota", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.grace_days as f64) * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeQuota {
        ProbeQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            grace_days: self.grace_days,
            ratio: self.ratio,
            quantity: self.quantity,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a probe quota in status requested.
pub fn new_probe_quota(code: &str, name: &str) -> ProbeQuota {
    ProbeQuota {
        code: code.to_string(),
        name: name.to_string(),
        grace_days: 0,
        ratio: 0.0,
        quantity: 0,
        minimum: 0.0,
        status: ProbeQuotaStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_probe_quota("code-463", "name-297");
        assert_eq!(item.status, ProbeQuotaStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_probe_quota("code-463", "name-297");
        if item.move_to(ProbeQuotaStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ProbeQuotaStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_probe_quota("code-463", "name-297");
        item.status = ProbeQuotaStatus::Expired;
        assert!(
            item.move_to(ProbeQuotaStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
