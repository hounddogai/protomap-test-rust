use super::*;

use acme_kit::text;

/// Where a allowlist agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistAgreementStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AllowlistAgreementStatus {
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

impl std::fmt::Display for AllowlistAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One allowlist agreement, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistAgreement {
    pub code: String,
    pub name: String,
    pub sequence: i64,
    pub expires_at: i64,
    pub capacity: i64,
    pub attempts: i64,
    pub score: f64,
    pub margin: f64,
    pub status: AllowlistAgreementStatus,
}

impl AllowlistAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistAgreementStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistAgreementStatus) -> bool {
        match self.status {
            AllowlistAgreementStatus::Draft => {
                return next == AllowlistAgreementStatus::Pending;
            }
            AllowlistAgreementStatus::Pending => {
                match next {
                    AllowlistAgreementStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistAgreementStatus::Active => {
                match next {
                    AllowlistAgreementStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistAgreementStatus::Suspended => {
                match next {
                    AllowlistAgreementStatus::Closed => {
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

    pub fn move_to(&mut self, next: AllowlistAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.sequence as f64 + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistAgreement {
        AllowlistAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            sequence: self.sequence,
            expires_at: self.expires_at,
            capacity: self.capacity,
            attempts: self.attempts,
            score: self.score,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a allowlist agreement in status draft.
pub fn new_allowlist_agreement(code: &str, name: &str) -> AllowlistAgreement {
    AllowlistAgreement {
        code: code.to_string(),
        name: name.to_string(),
        sequence: 0,
        expires_at: 0,
        capacity: 0,
        attempts: 0,
        score: 0.0,
        margin: 0.0,
        status: AllowlistAgreementStatus::Draft,
    }
}
