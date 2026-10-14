use super::*;

use acme_kit::text;

/// Where a tiered credential threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredCredentialThresholdStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TieredCredentialThresholdStatus {
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

impl std::fmt::Display for TieredCredentialThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered credential threshold and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredCredentialThreshold {
    pub code: String,
    pub label: String,
    pub threshold: f64,
    pub timezone: String,
    pub scope: String,
    pub pinned: bool,
    pub batch_size: i64,
    pub phone: String,
    pub revision: i64,
    pub rotation_days: i64,
    pub status: TieredCredentialThresholdStatus,
}

impl TieredCredentialThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredCredentialThresholdStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredCredentialThresholdStatus) -> bool {
        match self.status {
            TieredCredentialThresholdStatus::Draft => {
                return next == TieredCredentialThresholdStatus::Active
                    || next == TieredCredentialThresholdStatus::Suspended;
            }
            TieredCredentialThresholdStatus::Pending => {
                match next {
                    TieredCredentialThresholdStatus::Suspended | TieredCredentialThresholdStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredCredentialThresholdStatus::Active => {
                match next {
                    TieredCredentialThresholdStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredCredentialThresholdStatus::Suspended => {
                return next == TieredCredentialThresholdStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredCredentialThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered credential threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.revision as f64 + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredCredentialThreshold {
        TieredCredentialThreshold {
            code: self.code.clone(),
            label: self.label.clone(),
            threshold: self.threshold,
            timezone: self.timezone.clone(),
            scope: self.scope.clone(),
            pinned: self.pinned,
            batch_size: self.batch_size,
            phone: self.phone.clone(),
            revision: self.revision,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a tiered credential threshold in status draft.
pub fn new_tiered_credential_threshold(code: &str, label: &str) -> TieredCredentialThreshold {
    TieredCredentialThreshold {
        code: code.to_string(),
        label: label.to_string(),
        threshold: 0.0,
        timezone: "".to_string(),
        scope: "".to_string(),
        pinned: false,
        batch_size: 0,
        phone: "".to_string(),
        revision: 0,
        rotation_days: 0,
        status: TieredCredentialThresholdStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_tiered_credential_threshold("code-57", "label-99");
        assert_eq!(item.status, TieredCredentialThresholdStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_tiered_credential_threshold("code-57", "label-99");
        if item.move_to(TieredCredentialThresholdStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredCredentialThresholdStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_tiered_credential_threshold("code-57", "label-99");
        item.status = TieredCredentialThresholdStatus::Closed;
        assert!(
            item.move_to(TieredCredentialThresholdStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
