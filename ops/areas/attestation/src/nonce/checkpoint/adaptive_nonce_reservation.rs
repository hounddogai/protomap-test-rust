use super::*;

use acme_kit::text;

/// Where a adaptive nonce reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveNonceReservationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl AdaptiveNonceReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for AdaptiveNonceReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive nonce reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveNonceReservation {
    pub code: String,
    pub name: String,
    pub tags: Vec<String>,
    pub namespace: String,
    pub tenant_id: String,
    pub scope: String,
    pub quantity: i64,
    pub status: AdaptiveNonceReservationStatus,
}

impl AdaptiveNonceReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveNonceReservationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveNonceReservationStatus) -> bool {
        match self.status {
            AdaptiveNonceReservationStatus::Requested => {
                return next == AdaptiveNonceReservationStatus::Granted
                    || next == AdaptiveNonceReservationStatus::Revoked;
            }
            AdaptiveNonceReservationStatus::Granted => {
                match next {
                    AdaptiveNonceReservationStatus::Revoked | AdaptiveNonceReservationStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveNonceReservationStatus::Revoked => {
                match next {
                    AdaptiveNonceReservationStatus::Expired => {
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

    pub fn move_to(&mut self, next: AdaptiveNonceReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive nonce reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> AdaptiveNonceReservation {
        AdaptiveNonceReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            tags: self.tags.clone(),
            namespace: self.namespace.clone(),
            tenant_id: self.tenant_id.clone(),
            scope: self.scope.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a adaptive nonce reservation in status requested.
pub fn new_adaptive_nonce_reservation(code: &str, name: &str) -> AdaptiveNonceReservation {
    AdaptiveNonceReservation {
        code: code.to_string(),
        name: name.to_string(),
        tags: Vec::new(),
        namespace: "".to_string(),
        tenant_id: "".to_string(),
        scope: "".to_string(),
        quantity: 0,
        status: AdaptiveNonceReservationStatus::Requested,
    }
}
