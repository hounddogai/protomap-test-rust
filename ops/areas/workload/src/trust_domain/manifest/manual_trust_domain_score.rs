use super::*;

use acme_kit::text;

/// Where a manual trust domain score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualTrustDomainScoreStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ManualTrustDomainScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ManualTrustDomainScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual trust domain score and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualTrustDomainScore {
    pub code: String,
    pub title: String,
    pub cost: f64,
    pub unit: String,
    pub retries: i64,
    pub label: String,
    pub cluster: String,
    pub status: ManualTrustDomainScoreStatus,
}

impl ManualTrustDomainScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualTrustDomainScoreStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualTrustDomainScoreStatus) -> bool {
        match self.status {
            ManualTrustDomainScoreStatus::Requested => {
                return next == ManualTrustDomainScoreStatus::Revoked;
            }
            ManualTrustDomainScoreStatus::Granted => {
                match next {
                    ManualTrustDomainScoreStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualTrustDomainScoreStatus::Revoked => {
                match next {
                    ManualTrustDomainScoreStatus::Expired => {
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

    pub fn move_to(&mut self, next: ManualTrustDomainScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual trust domain score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "manual trust domain score", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        (self.cost * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualTrustDomainScore {
        ManualTrustDomainScore {
            code: self.code.clone(),
            title: self.title.clone(),
            cost: self.cost,
            unit: self.unit.clone(),
            retries: self.retries,
            label: self.label.clone(),
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a manual trust domain score in status requested.
pub fn new_manual_trust_domain_score(code: &str, title: &str) -> ManualTrustDomainScore {
    ManualTrustDomainScore {
        code: code.to_string(),
        title: title.to_string(),
        cost: 0.0,
        unit: "".to_string(),
        retries: 0,
        label: "".to_string(),
        cluster: "".to_string(),
        status: ManualTrustDomainScoreStatus::Requested,
    }
}
