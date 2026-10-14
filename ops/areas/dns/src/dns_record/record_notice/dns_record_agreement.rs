use super::*;

use acme_kit::text;

/// Where a dns record agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordAgreementStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DnsRecordAgreementStatus {
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

impl std::fmt::Display for DnsRecordAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dns record agreement. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordAgreement {
    pub code: String,
    pub label: String,
    pub visible: bool,
    pub description: String,
    pub active: bool,
    pub revision: i64,
    pub rate: f64,
    pub sequence: i64,
    pub batch_size: i64,
    pub port: i64,
    pub grace_days: i64,
    pub status: DnsRecordAgreementStatus,
}

impl DnsRecordAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordAgreementStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordAgreementStatus) -> bool {
        match self.status {
            DnsRecordAgreementStatus::Draft => {
                match next {
                    DnsRecordAgreementStatus::Suspended | DnsRecordAgreementStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordAgreementStatus::Pending => {
                match next {
                    DnsRecordAgreementStatus::Active | DnsRecordAgreementStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordAgreementStatus::Active => {
                return next == DnsRecordAgreementStatus::Suspended || next == DnsRecordAgreementStatus::Closed;
            }
            DnsRecordAgreementStatus::Suspended => {
                match next {
                    DnsRecordAgreementStatus::Closed => {
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

    pub fn move_to(&mut self, next: DnsRecordAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.revision as f64) * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordAgreement {
        DnsRecordAgreement {
            code: self.code.clone(),
            label: self.label.clone(),
            visible: self.visible,
            description: self.description.clone(),
            active: self.active,
            revision: self.revision,
            rate: self.rate,
            sequence: self.sequence,
            batch_size: self.batch_size,
            port: self.port,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a dns record agreement in status draft.
pub fn new_dns_record_agreement(code: &str, label: &str) -> DnsRecordAgreement {
    DnsRecordAgreement {
        code: code.to_string(),
        label: label.to_string(),
        visible: false,
        description: "".to_string(),
        active: false,
        revision: 0,
        rate: 0.0,
        sequence: 0,
        batch_size: 0,
        port: 0,
        grace_days: 0,
        status: DnsRecordAgreementStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_dns_record_agreement("code-882", "label-674");
        assert_eq!(item.status, DnsRecordAgreementStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_dns_record_agreement("code-882", "label-674");
        if item.move_to(DnsRecordAgreementStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordAgreementStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_dns_record_agreement("code-882", "label-674");
        item.status = DnsRecordAgreementStatus::Closed;
        assert!(
            item.move_to(DnsRecordAgreementStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
