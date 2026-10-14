use super::*;

use acme_kit::text;

/// Where a rolling federation request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingFederationRequestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RollingFederationRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RollingFederationRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling federation request, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingFederationRequest {
    pub code: String,
    pub title: String,
    pub maximum: f64,
    pub currency: String,
    pub checksum: String,
    pub status: RollingFederationRequestStatus,
}

impl RollingFederationRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingFederationRequestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingFederationRequestStatus) -> bool {
        match self.status {
            RollingFederationRequestStatus::Created => {
                return next == RollingFederationRequestStatus::Validated
                    || next == RollingFederationRequestStatus::Retired;
            }
            RollingFederationRequestStatus::Validated => {
                match next {
                    RollingFederationRequestStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingFederationRequestStatus::Published => {
                match next {
                    RollingFederationRequestStatus::Retired => {
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

    pub fn move_to(&mut self, next: RollingFederationRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling federation request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "rolling federation request", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.maximum
    }

    pub fn copy(&self) -> RollingFederationRequest {
        RollingFederationRequest {
            code: self.code.clone(),
            title: self.title.clone(),
            maximum: self.maximum,
            currency: self.currency.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rolling federation request in status created.
pub fn new_rolling_federation_request(code: &str, title: &str) -> RollingFederationRequest {
    RollingFederationRequest {
        code: code.to_string(),
        title: title.to_string(),
        maximum: 0.0,
        currency: "".to_string(),
        checksum: "".to_string(),
        status: RollingFederationRequestStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_rolling_federation_request("code-446", "title-598");
        assert_eq!(item.status, RollingFederationRequestStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_rolling_federation_request("code-446", "title-598");
        if item.move_to(RollingFederationRequestStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingFederationRequestStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_rolling_federation_request("code-446", "title-598");
        item.status = RollingFederationRequestStatus::Retired;
        assert!(
            item.move_to(RollingFederationRequestStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
