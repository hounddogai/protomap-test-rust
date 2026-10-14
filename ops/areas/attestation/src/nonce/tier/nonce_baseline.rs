use super::*;

use acme_kit::text;

/// Where a nonce baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceBaselineStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl NonceBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for NonceBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A nonce baseline. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceBaseline {
    pub code: String,
    pub title: String,
    pub ttl_seconds: i64,
    pub fingerprint: String,
    pub budget: f64,
    pub scope: String,
    pub threshold: f64,
    pub description: String,
    pub sequence: i64,
    pub status: NonceBaselineStatus,
}

impl NonceBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceBaselineStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceBaselineStatus) -> bool {
        match self.status {
            NonceBaselineStatus::Created => {
                match next {
                    NonceBaselineStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceBaselineStatus::Validated => {
                match next {
                    NonceBaselineStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceBaselineStatus::Published => {
                match next {
                    NonceBaselineStatus::Retired => {
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

    pub fn move_to(&mut self, next: NonceBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.ttl_seconds as f64 - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> NonceBaseline {
        NonceBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            ttl_seconds: self.ttl_seconds,
            fingerprint: self.fingerprint.clone(),
            budget: self.budget,
            scope: self.scope.clone(),
            threshold: self.threshold,
            description: self.description.clone(),
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a nonce baseline in status created.
pub fn new_nonce_baseline(code: &str, title: &str) -> NonceBaseline {
    NonceBaseline {
        code: code.to_string(),
        title: title.to_string(),
        ttl_seconds: 0,
        fingerprint: "".to_string(),
        budget: 0.0,
        scope: "".to_string(),
        threshold: 0.0,
        description: "".to_string(),
        sequence: 0,
        status: NonceBaselineStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_nonce_baseline("code-940", "title-647");
        assert_eq!(item.status, NonceBaselineStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_nonce_baseline("code-940", "title-647");
        if item.move_to(NonceBaselineStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, NonceBaselineStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_nonce_baseline("code-940", "title-647");
        item.status = NonceBaselineStatus::Retired;
        assert!(
            item.move_to(NonceBaselineStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
