use super::*;

use acme_kit::text;

/// Where a lenient dns record quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientDnsRecordQuotaStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LenientDnsRecordQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LenientDnsRecordQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient dns record quota as the dns record team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientDnsRecordQuota {
    pub code: String,
    pub name: String,
    pub limit: i64,
    pub key_id: String,
    pub quantity: i64,
    pub status: LenientDnsRecordQuotaStatus,
}

impl LenientDnsRecordQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientDnsRecordQuotaStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientDnsRecordQuotaStatus) -> bool {
        match self.status {
            LenientDnsRecordQuotaStatus::Created => {
                match next {
                    LenientDnsRecordQuotaStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientDnsRecordQuotaStatus::Validated => {
                return next == LenientDnsRecordQuotaStatus::Retired;
            }
            LenientDnsRecordQuotaStatus::Published => {
                return next == LenientDnsRecordQuotaStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientDnsRecordQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient dns record quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "lenient dns record quota", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.limit as f64) * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientDnsRecordQuota {
        LenientDnsRecordQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            limit: self.limit,
            key_id: self.key_id.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a lenient dns record quota in status created.
pub fn new_lenient_dns_record_quota(code: &str, name: &str) -> LenientDnsRecordQuota {
    LenientDnsRecordQuota {
        code: code.to_string(),
        name: name.to_string(),
        limit: 0,
        key_id: "".to_string(),
        quantity: 0,
        status: LenientDnsRecordQuotaStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_lenient_dns_record_quota("code-966", "name-818");
        assert_eq!(item.status, LenientDnsRecordQuotaStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_lenient_dns_record_quota("code-966", "name-818");
        if item.move_to(LenientDnsRecordQuotaStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientDnsRecordQuotaStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_lenient_dns_record_quota("code-966", "name-818");
        item.status = LenientDnsRecordQuotaStatus::Retired;
        assert!(
            item.move_to(LenientDnsRecordQuotaStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
