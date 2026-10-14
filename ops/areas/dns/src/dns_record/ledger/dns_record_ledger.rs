use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a dns record ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordLedgerStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DnsRecordLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DnsRecordLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One dns record ledger, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordLedger {
    pub code: String,
    pub label: String,
    pub title: String,
    pub ends_at: i64,
    pub attributes: BTreeMap<String, String>,
    pub expires_at: i64,
    pub timezone: String,
    pub amount: f64,
    pub rate: f64,
    pub port: i64,
    pub quantity: i64,
    pub status: DnsRecordLedgerStatus,
}

impl DnsRecordLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordLedgerStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordLedgerStatus) -> bool {
        match self.status {
            DnsRecordLedgerStatus::Requested => {
                return next == DnsRecordLedgerStatus::Expired;
            }
            DnsRecordLedgerStatus::Granted => {
                match next {
                    DnsRecordLedgerStatus::Revoked | DnsRecordLedgerStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordLedgerStatus::Revoked => {
                match next {
                    DnsRecordLedgerStatus::Expired => {
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

    pub fn move_to(&mut self, next: DnsRecordLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "dns record ledger", self.code)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn net_total(&self) -> f64 {
        ((self.expires_at as f64 - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordLedger {
        DnsRecordLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            title: self.title.clone(),
            ends_at: self.ends_at,
            attributes: self.attributes.clone(),
            expires_at: self.expires_at,
            timezone: self.timezone.clone(),
            amount: self.amount,
            rate: self.rate,
            port: self.port,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a dns record ledger in status requested.
pub fn new_dns_record_ledger(code: &str, label: &str) -> DnsRecordLedger {
    DnsRecordLedger {
        code: code.to_string(),
        label: label.to_string(),
        title: "".to_string(),
        ends_at: 0,
        attributes: BTreeMap::new(),
        expires_at: 0,
        timezone: "".to_string(),
        amount: 0.0,
        rate: 0.0,
        port: 0,
        quantity: 0,
        status: DnsRecordLedgerStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_dns_record_ledger("code-515", "label-68");
        assert_eq!(item.status, DnsRecordLedgerStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_dns_record_ledger("code-515", "label-68");
        if item.move_to(DnsRecordLedgerStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordLedgerStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_dns_record_ledger("code-515", "label-68");
        item.status = DnsRecordLedgerStatus::Expired;
        assert!(
            item.move_to(DnsRecordLedgerStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
