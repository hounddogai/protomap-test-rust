use super::*;

use acme_kit::text;

/// Where a manual dns record agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualDnsRecordAgreementStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ManualDnsRecordAgreementStatus {
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

impl std::fmt::Display for ManualDnsRecordAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual dns record agreement as the dns record team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualDnsRecordAgreement {
    pub code: String,
    pub title: String,
    pub grace_days: i64,
    pub maximum: f64,
    pub rate: f64,
    pub minimum: f64,
    pub status: ManualDnsRecordAgreementStatus,
}

impl ManualDnsRecordAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualDnsRecordAgreementStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualDnsRecordAgreementStatus) -> bool {
        match self.status {
            ManualDnsRecordAgreementStatus::Draft => {
                return next == ManualDnsRecordAgreementStatus::Active;
            }
            ManualDnsRecordAgreementStatus::Pending => {
                return next == ManualDnsRecordAgreementStatus::Active;
            }
            ManualDnsRecordAgreementStatus::Active => {
                return next == ManualDnsRecordAgreementStatus::Closed;
            }
            ManualDnsRecordAgreementStatus::Suspended => {
                match next {
                    ManualDnsRecordAgreementStatus::Closed => {
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

    pub fn move_to(&mut self, next: ManualDnsRecordAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual dns record agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual dns record agreement", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.rate - (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualDnsRecordAgreement {
        ManualDnsRecordAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            grace_days: self.grace_days,
            maximum: self.maximum,
            rate: self.rate,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a manual dns record agreement in status draft.
pub fn new_manual_dns_record_agreement(code: &str, title: &str) -> ManualDnsRecordAgreement {
    ManualDnsRecordAgreement {
        code: code.to_string(),
        title: title.to_string(),
        grace_days: 0,
        maximum: 0.0,
        rate: 0.0,
        minimum: 0.0,
        status: ManualDnsRecordAgreementStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_manual_dns_record_agreement("code-572", "title-764");
        assert_eq!(item.status, ManualDnsRecordAgreementStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_manual_dns_record_agreement("code-572", "title-764");
        if item.move_to(ManualDnsRecordAgreementStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualDnsRecordAgreementStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_manual_dns_record_agreement("code-572", "title-764");
        item.status = ManualDnsRecordAgreementStatus::Closed;
        assert!(
            item.move_to(ManualDnsRecordAgreementStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
