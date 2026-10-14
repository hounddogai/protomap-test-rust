use super::*;

use acme_kit::text;

/// Where a dns record draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordDraftStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DnsRecordDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DnsRecordDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dns record draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordDraft {
    pub code: String,
    pub label: String,
    pub maximum: f64,
    pub approved: bool,
    pub margin: f64,
    pub minimum: f64,
    pub status: DnsRecordDraftStatus,
}

impl DnsRecordDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordDraftStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordDraftStatus) -> bool {
        match self.status {
            DnsRecordDraftStatus::Requested => {
                return next == DnsRecordDraftStatus::Revoked || next == DnsRecordDraftStatus::Expired;
            }
            DnsRecordDraftStatus::Granted => {
                match next {
                    DnsRecordDraftStatus::Revoked | DnsRecordDraftStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordDraftStatus::Revoked => {
                return next == DnsRecordDraftStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DnsRecordDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.margin - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordDraft {
        DnsRecordDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            maximum: self.maximum,
            approved: self.approved,
            margin: self.margin,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a dns record draft in status requested.
pub fn new_dns_record_draft(code: &str, label: &str) -> DnsRecordDraft {
    DnsRecordDraft {
        code: code.to_string(),
        label: label.to_string(),
        maximum: 0.0,
        approved: false,
        margin: 0.0,
        minimum: 0.0,
        status: DnsRecordDraftStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_dns_record_draft("code-42", "label-811");
        assert_eq!(item.status, DnsRecordDraftStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_dns_record_draft("code-42", "label-811");
        if item.move_to(DnsRecordDraftStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordDraftStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_dns_record_draft("code-42", "label-811");
        item.status = DnsRecordDraftStatus::Expired;
        assert!(
            item.move_to(DnsRecordDraftStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
