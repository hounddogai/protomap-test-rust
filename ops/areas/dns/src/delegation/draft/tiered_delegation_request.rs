use super::*;

use acme_kit::text;

/// Where a tiered delegation request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredDelegationRequestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TieredDelegationRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TieredDelegationRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tiered delegation request, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationRequest {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub port: i64,
    pub country: String,
    pub minimum: f64,
    pub title: String,
    pub external_ref: String,
    pub audience: String,
    pub ends_at: i64,
    pub duration_minutes: i64,
    pub status: TieredDelegationRequestStatus,
}

impl TieredDelegationRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredDelegationRequestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredDelegationRequestStatus) -> bool {
        match self.status {
            TieredDelegationRequestStatus::Created => {
                return next == TieredDelegationRequestStatus::Validated;
            }
            TieredDelegationRequestStatus::Validated => {
                return next == TieredDelegationRequestStatus::Retired;
            }
            TieredDelegationRequestStatus::Published => {
                match next {
                    TieredDelegationRequestStatus::Retired => {
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

    pub fn move_to(&mut self, next: TieredDelegationRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered delegation request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered delegation request", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.quantity as f64 + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredDelegationRequest {
        TieredDelegationRequest {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            port: self.port,
            country: self.country.clone(),
            minimum: self.minimum,
            title: self.title.clone(),
            external_ref: self.external_ref.clone(),
            audience: self.audience.clone(),
            ends_at: self.ends_at,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a tiered delegation request in status created.
pub fn new_tiered_delegation_request(code: &str, name: &str) -> TieredDelegationRequest {
    TieredDelegationRequest {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        port: 0,
        country: "".to_string(),
        minimum: 0.0,
        title: "".to_string(),
        external_ref: "".to_string(),
        audience: "".to_string(),
        ends_at: 0,
        duration_minutes: 0,
        status: TieredDelegationRequestStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_tiered_delegation_request("code-522", "name-171");
        assert_eq!(item.status, TieredDelegationRequestStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_tiered_delegation_request("code-522", "name-171");
        if item.move_to(TieredDelegationRequestStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredDelegationRequestStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_tiered_delegation_request("code-522", "name-171");
        item.status = TieredDelegationRequestStatus::Retired;
        assert!(
            item.move_to(TieredDelegationRequestStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
