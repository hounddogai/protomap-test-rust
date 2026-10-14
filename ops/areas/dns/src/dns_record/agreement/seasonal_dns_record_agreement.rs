use super::*;

use acme_kit::text;

/// Where a seasonal dns record agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalDnsRecordAgreementStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SeasonalDnsRecordAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SeasonalDnsRecordAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal dns record agreement, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalDnsRecordAgreement {
    pub code: String,
    pub title: String,
    pub cluster: String,
    pub minimum: f64,
    pub budget: f64,
    pub created_at: i64,
    pub maximum: f64,
    pub status: SeasonalDnsRecordAgreementStatus,
}

impl SeasonalDnsRecordAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalDnsRecordAgreementStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalDnsRecordAgreementStatus) -> bool {
        match self.status {
            SeasonalDnsRecordAgreementStatus::Requested => {
                return next == SeasonalDnsRecordAgreementStatus::Revoked
                    || next == SeasonalDnsRecordAgreementStatus::Expired;
            }
            SeasonalDnsRecordAgreementStatus::Granted => {
                match next {
                    SeasonalDnsRecordAgreementStatus::Revoked | SeasonalDnsRecordAgreementStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalDnsRecordAgreementStatus::Revoked => {
                match next {
                    SeasonalDnsRecordAgreementStatus::Expired => {
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

    pub fn move_to(&mut self, next: SeasonalDnsRecordAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal dns record agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.budget + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalDnsRecordAgreement {
        SeasonalDnsRecordAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            cluster: self.cluster.clone(),
            minimum: self.minimum,
            budget: self.budget,
            created_at: self.created_at,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a seasonal dns record agreement in status requested.
pub fn new_seasonal_dns_record_agreement(code: &str, title: &str) -> SeasonalDnsRecordAgreement {
    SeasonalDnsRecordAgreement {
        code: code.to_string(),
        title: title.to_string(),
        cluster: "".to_string(),
        minimum: 0.0,
        budget: 0.0,
        created_at: 0,
        maximum: 0.0,
        status: SeasonalDnsRecordAgreementStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_seasonal_dns_record_agreement("code-71", "title-373");
        assert_eq!(item.status, SeasonalDnsRecordAgreementStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_seasonal_dns_record_agreement("code-71", "title-373");
        if item.move_to(SeasonalDnsRecordAgreementStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalDnsRecordAgreementStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_seasonal_dns_record_agreement("code-71", "title-373");
        item.status = SeasonalDnsRecordAgreementStatus::Expired;
        assert!(
            item.move_to(SeasonalDnsRecordAgreementStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
