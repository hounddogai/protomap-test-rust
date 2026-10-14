use super::*;

use acme_kit::text;

/// Where a rolling quote bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingQuoteBundleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RollingQuoteBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RollingQuoteBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling quote bundle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingQuoteBundle {
    pub code: String,
    pub name: String,
    pub key_id: String,
    pub cipher: String,
    pub external_ref: String,
    pub score: f64,
    pub status: RollingQuoteBundleStatus,
}

impl RollingQuoteBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingQuoteBundleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingQuoteBundleStatus) -> bool {
        match self.status {
            RollingQuoteBundleStatus::Created => {
                return next == RollingQuoteBundleStatus::Validated;
            }
            RollingQuoteBundleStatus::Validated => {
                match next {
                    RollingQuoteBundleStatus::Published | RollingQuoteBundleStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingQuoteBundleStatus::Published => {
                return next == RollingQuoteBundleStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingQuoteBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling quote bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> RollingQuoteBundle {
        RollingQuoteBundle {
            code: self.code.clone(),
            name: self.name.clone(),
            key_id: self.key_id.clone(),
            cipher: self.cipher.clone(),
            external_ref: self.external_ref.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rolling quote bundle in status created.
pub fn new_rolling_quote_bundle(code: &str, name: &str) -> RollingQuoteBundle {
    RollingQuoteBundle {
        code: code.to_string(),
        name: name.to_string(),
        key_id: "".to_string(),
        cipher: "".to_string(),
        external_ref: "".to_string(),
        score: 0.0,
        status: RollingQuoteBundleStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_rolling_quote_bundle("code-314", "name-273");
        assert_eq!(item.status, RollingQuoteBundleStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_rolling_quote_bundle("code-314", "name-273");
        if item.move_to(RollingQuoteBundleStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingQuoteBundleStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_rolling_quote_bundle("code-314", "name-273");
        item.status = RollingQuoteBundleStatus::Retired;
        assert!(
            item.move_to(RollingQuoteBundleStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
