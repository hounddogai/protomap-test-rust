use super::*;

use acme_kit::text;

/// Where a dns record notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordNoticeStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DnsRecordNoticeStatus {
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

impl std::fmt::Display for DnsRecordNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One dns record notice, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordNotice {
    pub code: String,
    pub name: String,
    pub grace_days: i64,
    pub ttl_seconds: i64,
    pub fingerprint: String,
    pub revision: i64,
    pub expires_at: i64,
    pub margin: f64,
    pub ceiling: f64,
    pub score: f64,
    pub weight: f64,
    pub status: DnsRecordNoticeStatus,
}

impl DnsRecordNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordNoticeStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordNoticeStatus) -> bool {
        match self.status {
            DnsRecordNoticeStatus::Idle => {
                match next {
                    DnsRecordNoticeStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordNoticeStatus::Warming => {
                match next {
                    DnsRecordNoticeStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordNoticeStatus::Ready => {
                match next {
                    DnsRecordNoticeStatus::Draining | DnsRecordNoticeStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordNoticeStatus::Draining => {
                return next == DnsRecordNoticeStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DnsRecordNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record notice {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.grace_days as f64) * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordNotice {
        DnsRecordNotice {
            code: self.code.clone(),
            name: self.name.clone(),
            grace_days: self.grace_days,
            ttl_seconds: self.ttl_seconds,
            fingerprint: self.fingerprint.clone(),
            revision: self.revision,
            expires_at: self.expires_at,
            margin: self.margin,
            ceiling: self.ceiling,
            score: self.score,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a dns record notice in status idle.
pub fn new_dns_record_notice(code: &str, name: &str) -> DnsRecordNotice {
    DnsRecordNotice {
        code: code.to_string(),
        name: name.to_string(),
        grace_days: 0,
        ttl_seconds: 0,
        fingerprint: "".to_string(),
        revision: 0,
        expires_at: 0,
        margin: 0.0,
        ceiling: 0.0,
        score: 0.0,
        weight: 0.0,
        status: DnsRecordNoticeStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_dns_record_notice("code-880", "name-100");
        assert_eq!(item.status, DnsRecordNoticeStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_dns_record_notice("code-880", "name-100");
        if item.move_to(DnsRecordNoticeStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordNoticeStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_dns_record_notice("code-880", "name-100");
        item.status = DnsRecordNoticeStatus::Stopped;
        assert!(
            item.move_to(DnsRecordNoticeStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
