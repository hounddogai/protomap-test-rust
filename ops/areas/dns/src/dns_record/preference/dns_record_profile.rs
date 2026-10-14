use super::*;

use acme_kit::text;

/// Where a dns record profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordProfileStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DnsRecordProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DnsRecordProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dns record profile. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordProfile {
    pub code: String,
    pub label: String,
    pub cipher: String,
    pub principal: String,
    pub region: String,
    pub unit: String,
    pub amount: f64,
    pub status: DnsRecordProfileStatus,
}

impl DnsRecordProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordProfileStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordProfileStatus) -> bool {
        match self.status {
            DnsRecordProfileStatus::Requested => {
                return next == DnsRecordProfileStatus::Revoked;
            }
            DnsRecordProfileStatus::Granted => {
                match next {
                    DnsRecordProfileStatus::Revoked | DnsRecordProfileStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordProfileStatus::Revoked => {
                return next == DnsRecordProfileStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DnsRecordProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "dns record profile", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> DnsRecordProfile {
        DnsRecordProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            cipher: self.cipher.clone(),
            principal: self.principal.clone(),
            region: self.region.clone(),
            unit: self.unit.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a dns record profile in status requested.
pub fn new_dns_record_profile(code: &str, label: &str) -> DnsRecordProfile {
    DnsRecordProfile {
        code: code.to_string(),
        label: label.to_string(),
        cipher: "".to_string(),
        principal: "".to_string(),
        region: "".to_string(),
        unit: "".to_string(),
        amount: 0.0,
        status: DnsRecordProfileStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_dns_record_profile("code-233", "label-761");
        assert_eq!(item.status, DnsRecordProfileStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_dns_record_profile("code-233", "label-761");
        if item.move_to(DnsRecordProfileStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordProfileStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_dns_record_profile("code-233", "label-761");
        item.status = DnsRecordProfileStatus::Expired;
        assert!(
            item.move_to(DnsRecordProfileStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
