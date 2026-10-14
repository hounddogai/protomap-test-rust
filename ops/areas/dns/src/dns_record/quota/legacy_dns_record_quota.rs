use super::*;

use acme_kit::text;

/// Where a legacy dns record quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyDnsRecordQuotaStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LegacyDnsRecordQuotaStatus {
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

impl std::fmt::Display for LegacyDnsRecordQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy dns record quota. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDnsRecordQuota {
    pub code: String,
    pub name: String,
    pub label: String,
    pub cluster: String,
    pub batch_size: i64,
    pub status: LegacyDnsRecordQuotaStatus,
}

impl LegacyDnsRecordQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyDnsRecordQuotaStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyDnsRecordQuotaStatus) -> bool {
        match self.status {
            LegacyDnsRecordQuotaStatus::Idle => {
                match next {
                    LegacyDnsRecordQuotaStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyDnsRecordQuotaStatus::Warming => {
                return next == LegacyDnsRecordQuotaStatus::Ready || next == LegacyDnsRecordQuotaStatus::Draining;
            }
            LegacyDnsRecordQuotaStatus::Ready => {
                match next {
                    LegacyDnsRecordQuotaStatus::Draining | LegacyDnsRecordQuotaStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyDnsRecordQuotaStatus::Draining => {
                return next == LegacyDnsRecordQuotaStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyDnsRecordQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy dns record quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "legacy dns record quota", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.batch_size as f64
    }

    pub fn copy(&self) -> LegacyDnsRecordQuota {
        LegacyDnsRecordQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            label: self.label.clone(),
            cluster: self.cluster.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a legacy dns record quota in status idle.
pub fn new_legacy_dns_record_quota(code: &str, name: &str) -> LegacyDnsRecordQuota {
    LegacyDnsRecordQuota {
        code: code.to_string(),
        name: name.to_string(),
        label: "".to_string(),
        cluster: "".to_string(),
        batch_size: 0,
        status: LegacyDnsRecordQuotaStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_legacy_dns_record_quota("code-846", "name-812");
        assert_eq!(item.status, LegacyDnsRecordQuotaStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_legacy_dns_record_quota("code-846", "name-812");
        if item.move_to(LegacyDnsRecordQuotaStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyDnsRecordQuotaStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_legacy_dns_record_quota("code-846", "name-812");
        item.status = LegacyDnsRecordQuotaStatus::Stopped;
        assert!(
            item.move_to(LegacyDnsRecordQuotaStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
