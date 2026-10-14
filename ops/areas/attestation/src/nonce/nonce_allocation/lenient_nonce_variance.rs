use super::*;

use acme_kit::text;

/// Where a lenient nonce variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientNonceVarianceStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LenientNonceVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LenientNonceVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient nonce variance. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientNonceVariance {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub audience: String,
    pub grace_days: i64,
    pub scope: String,
    pub source: String,
    pub cluster: String,
    pub status: LenientNonceVarianceStatus,
}

impl LenientNonceVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientNonceVarianceStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientNonceVarianceStatus) -> bool {
        match self.status {
            LenientNonceVarianceStatus::Proposed => {
                match next {
                    LenientNonceVarianceStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientNonceVarianceStatus::Approved => {
                return next == LenientNonceVarianceStatus::Rejected;
            }
            LenientNonceVarianceStatus::Rejected => {
                match next {
                    LenientNonceVarianceStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: LenientNonceVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient nonce variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> LenientNonceVariance {
        LenientNonceVariance {
            code: self.code.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            audience: self.audience.clone(),
            grace_days: self.grace_days,
            scope: self.scope.clone(),
            source: self.source.clone(),
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a lenient nonce variance in status proposed.
pub fn new_lenient_nonce_variance(code: &str, name: &str) -> LenientNonceVariance {
    LenientNonceVariance {
        code: code.to_string(),
        name: name.to_string(),
        owner_id: "".to_string(),
        audience: "".to_string(),
        grace_days: 0,
        scope: "".to_string(),
        source: "".to_string(),
        cluster: "".to_string(),
        status: LenientNonceVarianceStatus::Proposed,
    }
}
