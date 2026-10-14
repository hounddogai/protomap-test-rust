use super::*;

use acme_kit::text;

/// Where a lenient nonce tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientNonceTierStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LenientNonceTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LenientNonceTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient nonce tier and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientNonceTier {
    pub code: String,
    pub label: String,
    pub grace_days: i64,
    pub expires_at: i64,
    pub issuer: String,
    pub retries: i64,
    pub starts_at: i64,
    pub host: String,
    pub contact_email: String,
    pub score: f64,
    pub limit: i64,
    pub status: LenientNonceTierStatus,
}

impl LenientNonceTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientNonceTierStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientNonceTierStatus) -> bool {
        match self.status {
            LenientNonceTierStatus::Requested => {
                return next == LenientNonceTierStatus::Revoked || next == LenientNonceTierStatus::Expired;
            }
            LenientNonceTierStatus::Granted => {
                match next {
                    LenientNonceTierStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientNonceTierStatus::Revoked => {
                return next == LenientNonceTierStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientNonceTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient nonce tier {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.expires_at as f64 + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientNonceTier {
        LenientNonceTier {
            code: self.code.clone(),
            label: self.label.clone(),
            grace_days: self.grace_days,
            expires_at: self.expires_at,
            issuer: self.issuer.clone(),
            retries: self.retries,
            starts_at: self.starts_at,
            host: self.host.clone(),
            contact_email: self.contact_email.clone(),
            score: self.score,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient nonce tier in status requested.
pub fn new_lenient_nonce_tier(code: &str, label: &str) -> LenientNonceTier {
    LenientNonceTier {
        code: code.to_string(),
        label: label.to_string(),
        grace_days: 0,
        expires_at: 0,
        issuer: "".to_string(),
        retries: 0,
        starts_at: 0,
        host: "".to_string(),
        contact_email: "".to_string(),
        score: 0.0,
        limit: 0,
        status: LenientNonceTierStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_lenient_nonce_tier("code-500", "label-704");
        assert_eq!(item.status, LenientNonceTierStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_lenient_nonce_tier("code-500", "label-704");
        if item.move_to(LenientNonceTierStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientNonceTierStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_lenient_nonce_tier("code-500", "label-704");
        item.status = LenientNonceTierStatus::Expired;
        assert!(
            item.move_to(LenientNonceTierStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
