use super::*;

use acme_kit::text;

/// Where a dns record quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordQuotaStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DnsRecordQuotaStatus {
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

impl std::fmt::Display for DnsRecordQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One dns record quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordQuota {
    pub code: String,
    pub title: String,
    pub pinned: bool,
    pub capacity: i64,
    pub active: bool,
    pub threshold: f64,
    pub audience: String,
    pub quantity: i64,
    pub channel: String,
    pub status: DnsRecordQuotaStatus,
}

impl DnsRecordQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordQuotaStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordQuotaStatus) -> bool {
        match self.status {
            DnsRecordQuotaStatus::Idle => {
                match next {
                    DnsRecordQuotaStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordQuotaStatus::Warming => {
                return next == DnsRecordQuotaStatus::Ready;
            }
            DnsRecordQuotaStatus::Ready => {
                return next == DnsRecordQuotaStatus::Draining || next == DnsRecordQuotaStatus::Stopped;
            }
            DnsRecordQuotaStatus::Draining => {
                return next == DnsRecordQuotaStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DnsRecordQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.capacity as f64 + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordQuota {
        DnsRecordQuota {
            code: self.code.clone(),
            title: self.title.clone(),
            pinned: self.pinned,
            capacity: self.capacity,
            active: self.active,
            threshold: self.threshold,
            audience: self.audience.clone(),
            quantity: self.quantity,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a dns record quota in status idle.
pub fn new_dns_record_quota(code: &str, title: &str) -> DnsRecordQuota {
    DnsRecordQuota {
        code: code.to_string(),
        title: title.to_string(),
        pinned: false,
        capacity: 0,
        active: false,
        threshold: 0.0,
        audience: "".to_string(),
        quantity: 0,
        channel: "".to_string(),
        status: DnsRecordQuotaStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_dns_record_quota("code-933", "title-573");
        assert_eq!(item.status, DnsRecordQuotaStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_dns_record_quota("code-933", "title-573");
        if item.move_to(DnsRecordQuotaStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordQuotaStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_dns_record_quota("code-933", "title-573");
        item.status = DnsRecordQuotaStatus::Stopped;
        assert!(
            item.move_to(DnsRecordQuotaStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
