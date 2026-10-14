use super::*;

use acme_kit::text;

/// Where a dns record target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordTargetStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DnsRecordTargetStatus {
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

impl std::fmt::Display for DnsRecordTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a dns record target and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordTarget {
    pub code: String,
    pub name: String,
    pub updated_at: i64,
    pub grace_days: i64,
    pub quota_units: i64,
    pub sequence: i64,
    pub created_at: i64,
    pub margin: f64,
    pub reviewer: String,
    pub status: DnsRecordTargetStatus,
}

impl DnsRecordTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordTargetStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordTargetStatus) -> bool {
        match self.status {
            DnsRecordTargetStatus::Idle => {
                return next == DnsRecordTargetStatus::Warming || next == DnsRecordTargetStatus::Draining;
            }
            DnsRecordTargetStatus::Warming => {
                match next {
                    DnsRecordTargetStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordTargetStatus::Ready => {
                return next == DnsRecordTargetStatus::Draining || next == DnsRecordTargetStatus::Stopped;
            }
            DnsRecordTargetStatus::Draining => {
                match next {
                    DnsRecordTargetStatus::Stopped => {
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

    pub fn move_to(&mut self, next: DnsRecordTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record target {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn adjusted_total(&self) -> f64 {
        (self.margin * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordTarget {
        DnsRecordTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            updated_at: self.updated_at,
            grace_days: self.grace_days,
            quota_units: self.quota_units,
            sequence: self.sequence,
            created_at: self.created_at,
            margin: self.margin,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a dns record target in status idle.
pub fn new_dns_record_target(code: &str, name: &str) -> DnsRecordTarget {
    DnsRecordTarget {
        code: code.to_string(),
        name: name.to_string(),
        updated_at: 0,
        grace_days: 0,
        quota_units: 0,
        sequence: 0,
        created_at: 0,
        margin: 0.0,
        reviewer: "".to_string(),
        status: DnsRecordTargetStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_dns_record_target("code-640", "name-812");
        assert_eq!(item.status, DnsRecordTargetStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_dns_record_target("code-640", "name-812");
        if item.move_to(DnsRecordTargetStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordTargetStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_dns_record_target("code-640", "name-812");
        item.status = DnsRecordTargetStatus::Stopped;
        assert!(
            item.move_to(DnsRecordTargetStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
