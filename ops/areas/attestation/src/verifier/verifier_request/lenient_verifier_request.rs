use super::*;

use acme_kit::text;

/// Where a lenient verifier request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientVerifierRequestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LenientVerifierRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LenientVerifierRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient verifier request. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientVerifierRequest {
    pub code: String,
    pub label: String,
    pub discount: f64,
    pub threshold: f64,
    pub external_ref: String,
    pub rate: f64,
    pub cipher: String,
    pub status: LenientVerifierRequestStatus,
}

impl LenientVerifierRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientVerifierRequestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientVerifierRequestStatus) -> bool {
        match self.status {
            LenientVerifierRequestStatus::Requested => {
                match next {
                    LenientVerifierRequestStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientVerifierRequestStatus::Granted => {
                return next == LenientVerifierRequestStatus::Revoked || next == LenientVerifierRequestStatus::Expired;
            }
            LenientVerifierRequestStatus::Revoked => {
                match next {
                    LenientVerifierRequestStatus::Expired => {
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

    pub fn move_to(&mut self, next: LenientVerifierRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient verifier request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.discount + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientVerifierRequest {
        LenientVerifierRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            discount: self.discount,
            threshold: self.threshold,
            external_ref: self.external_ref.clone(),
            rate: self.rate,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a lenient verifier request in status requested.
pub fn new_lenient_verifier_request(code: &str, label: &str) -> LenientVerifierRequest {
    LenientVerifierRequest {
        code: code.to_string(),
        label: label.to_string(),
        discount: 0.0,
        threshold: 0.0,
        external_ref: "".to_string(),
        rate: 0.0,
        cipher: "".to_string(),
        status: LenientVerifierRequestStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_lenient_verifier_request("code-358", "label-77");
        assert_eq!(item.status, LenientVerifierRequestStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_lenient_verifier_request("code-358", "label-77");
        if item.move_to(LenientVerifierRequestStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientVerifierRequestStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_lenient_verifier_request("code-358", "label-77");
        item.status = LenientVerifierRequestStatus::Expired;
        assert!(
            item.move_to(LenientVerifierRequestStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
