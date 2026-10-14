use super::*;

use acme_kit::text;

/// Where a pending federation variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingFederationVarianceStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PendingFederationVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PendingFederationVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending federation variance, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingFederationVariance {
    pub code: String,
    pub label: String,
    pub description: String,
    pub cipher: String,
    pub rate: f64,
    pub minimum: f64,
    pub duration_minutes: i64,
    pub status: PendingFederationVarianceStatus,
}

impl PendingFederationVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingFederationVarianceStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingFederationVarianceStatus) -> bool {
        match self.status {
            PendingFederationVarianceStatus::Created => {
                match next {
                    PendingFederationVarianceStatus::Published | PendingFederationVarianceStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingFederationVarianceStatus::Validated => {
                match next {
                    PendingFederationVarianceStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingFederationVarianceStatus::Published => {
                return next == PendingFederationVarianceStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingFederationVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending federation variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.minimum + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingFederationVariance {
        PendingFederationVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            description: self.description.clone(),
            cipher: self.cipher.clone(),
            rate: self.rate,
            minimum: self.minimum,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a pending federation variance in status created.
pub fn new_pending_federation_variance(code: &str, label: &str) -> PendingFederationVariance {
    PendingFederationVariance {
        code: code.to_string(),
        label: label.to_string(),
        description: "".to_string(),
        cipher: "".to_string(),
        rate: 0.0,
        minimum: 0.0,
        duration_minutes: 0,
        status: PendingFederationVarianceStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_pending_federation_variance("code-212", "label-770");
        assert_eq!(item.status, PendingFederationVarianceStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_pending_federation_variance("code-212", "label-770");
        if item.move_to(PendingFederationVarianceStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingFederationVarianceStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_pending_federation_variance("code-212", "label-770");
        item.status = PendingFederationVarianceStatus::Retired;
        assert!(
            item.move_to(PendingFederationVarianceStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
