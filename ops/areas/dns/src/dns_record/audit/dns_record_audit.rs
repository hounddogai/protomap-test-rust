use super::*;

use acme_kit::text;

/// Where a dns record audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordAuditStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DnsRecordAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for DnsRecordAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a dns record audit and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordAudit {
    pub code: String,
    pub label: String,
    pub starts_at: i64,
    pub limit: i64,
    pub pinned: bool,
    pub status: DnsRecordAuditStatus,
}

impl DnsRecordAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordAuditStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordAuditStatus) -> bool {
        match self.status {
            DnsRecordAuditStatus::Draft => {
                return next == DnsRecordAuditStatus::Suspended;
            }
            DnsRecordAuditStatus::Pending => {
                match next {
                    DnsRecordAuditStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordAuditStatus::Active => {
                match next {
                    DnsRecordAuditStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordAuditStatus::Suspended => {
                return next == DnsRecordAuditStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DnsRecordAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.limit as f64 + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordAudit {
        DnsRecordAudit {
            code: self.code.clone(),
            label: self.label.clone(),
            starts_at: self.starts_at,
            limit: self.limit,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a dns record audit in status draft.
pub fn new_dns_record_audit(code: &str, label: &str) -> DnsRecordAudit {
    DnsRecordAudit {
        code: code.to_string(),
        label: label.to_string(),
        starts_at: 0,
        limit: 0,
        pinned: false,
        status: DnsRecordAuditStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_dns_record_audit("code-737", "label-363");
        assert_eq!(item.status, DnsRecordAuditStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_dns_record_audit("code-737", "label-363");
        if item.move_to(DnsRecordAuditStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordAuditStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_dns_record_audit("code-737", "label-363");
        item.status = DnsRecordAuditStatus::Closed;
        assert!(
            item.move_to(DnsRecordAuditStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
