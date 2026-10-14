use super::*;

use acme_kit::text;

/// Where a default alert claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultAlertClaimStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DefaultAlertClaimStatus {
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

impl std::fmt::Display for DefaultAlertClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default alert claim. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultAlertClaim {
    pub code: String,
    pub label: String,
    pub quantity: i64,
    pub key_id: String,
    pub burst: i64,
    pub status: DefaultAlertClaimStatus,
}

impl DefaultAlertClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultAlertClaimStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultAlertClaimStatus) -> bool {
        match self.status {
            DefaultAlertClaimStatus::Draft => {
                return next == DefaultAlertClaimStatus::Suspended || next == DefaultAlertClaimStatus::Closed;
            }
            DefaultAlertClaimStatus::Pending => {
                match next {
                    DefaultAlertClaimStatus::Active | DefaultAlertClaimStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultAlertClaimStatus::Active => {
                match next {
                    DefaultAlertClaimStatus::Suspended | DefaultAlertClaimStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultAlertClaimStatus::Suspended => {
                return next == DefaultAlertClaimStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultAlertClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default alert claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "default alert claim", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.burst as f64) * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultAlertClaim {
        DefaultAlertClaim {
            code: self.code.clone(),
            label: self.label.clone(),
            quantity: self.quantity,
            key_id: self.key_id.clone(),
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a default alert claim in status draft.
pub fn new_default_alert_claim(code: &str, label: &str) -> DefaultAlertClaim {
    DefaultAlertClaim {
        code: code.to_string(),
        label: label.to_string(),
        quantity: 0,
        key_id: "".to_string(),
        burst: 0,
        status: DefaultAlertClaimStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_default_alert_claim("code-941", "label-910");
        assert_eq!(item.status, DefaultAlertClaimStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_default_alert_claim("code-941", "label-910");
        if item.move_to(DefaultAlertClaimStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultAlertClaimStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_default_alert_claim("code-941", "label-910");
        item.status = DefaultAlertClaimStatus::Closed;
        assert!(
            item.move_to(DefaultAlertClaimStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
