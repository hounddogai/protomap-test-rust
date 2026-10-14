use super::*;

use acme_kit::text;

/// Where a federation exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationExceptionStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FederationExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FederationExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a federation exception and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationException {
    pub code: String,
    pub title: String,
    pub burst: i64,
    pub minimum: f64,
    pub grace_days: i64,
    pub port: i64,
    pub ttl_seconds: i64,
    pub quantity: i64,
    pub maximum: f64,
    pub status: FederationExceptionStatus,
}

impl FederationException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationExceptionStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationExceptionStatus) -> bool {
        match self.status {
            FederationExceptionStatus::Created => {
                match next {
                    FederationExceptionStatus::Published | FederationExceptionStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationExceptionStatus::Validated => {
                match next {
                    FederationExceptionStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationExceptionStatus::Published => {
                return next == FederationExceptionStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FederationExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.burst as f64 - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationException {
        FederationException {
            code: self.code.clone(),
            title: self.title.clone(),
            burst: self.burst,
            minimum: self.minimum,
            grace_days: self.grace_days,
            port: self.port,
            ttl_seconds: self.ttl_seconds,
            quantity: self.quantity,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a federation exception in status created.
pub fn new_federation_exception(code: &str, title: &str) -> FederationException {
    FederationException {
        code: code.to_string(),
        title: title.to_string(),
        burst: 0,
        minimum: 0.0,
        grace_days: 0,
        port: 0,
        ttl_seconds: 0,
        quantity: 0,
        maximum: 0.0,
        status: FederationExceptionStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_federation_exception("code-905", "title-319");
        assert_eq!(item.status, FederationExceptionStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_federation_exception("code-905", "title-319");
        if item.move_to(FederationExceptionStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationExceptionStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_federation_exception("code-905", "title-319");
        item.status = FederationExceptionStatus::Retired;
        assert!(
            item.move_to(FederationExceptionStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
