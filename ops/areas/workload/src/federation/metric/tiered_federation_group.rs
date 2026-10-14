use super::*;

use acme_kit::text;

/// Where a tiered federation group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredFederationGroupStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TieredFederationGroupStatus {
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

impl std::fmt::Display for TieredFederationGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered federation group and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredFederationGroup {
    pub code: String,
    pub label: String,
    pub expires_at: i64,
    pub attempts: i64,
    pub currency: String,
    pub ratio: f64,
    pub status: TieredFederationGroupStatus,
}

impl TieredFederationGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredFederationGroupStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredFederationGroupStatus) -> bool {
        match self.status {
            TieredFederationGroupStatus::Draft => {
                return next == TieredFederationGroupStatus::Active || next == TieredFederationGroupStatus::Closed;
            }
            TieredFederationGroupStatus::Pending => {
                match next {
                    TieredFederationGroupStatus::Suspended | TieredFederationGroupStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredFederationGroupStatus::Active => {
                match next {
                    TieredFederationGroupStatus::Suspended | TieredFederationGroupStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredFederationGroupStatus::Suspended => {
                match next {
                    TieredFederationGroupStatus::Closed => {
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

    pub fn move_to(&mut self, next: TieredFederationGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered federation group {} cannot move from {} to {}",
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
        format!("{} {}", "tiered federation group", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.attempts as f64 + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredFederationGroup {
        TieredFederationGroup {
            code: self.code.clone(),
            label: self.label.clone(),
            expires_at: self.expires_at,
            attempts: self.attempts,
            currency: self.currency.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a tiered federation group in status draft.
pub fn new_tiered_federation_group(code: &str, label: &str) -> TieredFederationGroup {
    TieredFederationGroup {
        code: code.to_string(),
        label: label.to_string(),
        expires_at: 0,
        attempts: 0,
        currency: "".to_string(),
        ratio: 0.0,
        status: TieredFederationGroupStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_tiered_federation_group("code-312", "label-502");
        assert_eq!(item.status, TieredFederationGroupStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_tiered_federation_group("code-312", "label-502");
        if item.move_to(TieredFederationGroupStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredFederationGroupStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_tiered_federation_group("code-312", "label-502");
        item.status = TieredFederationGroupStatus::Closed;
        assert!(
            item.move_to(TieredFederationGroupStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
