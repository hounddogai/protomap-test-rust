use super::*;

use acme_kit::text;

/// Where a peering bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringBundleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PeeringBundleStatus {
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

impl std::fmt::Display for PeeringBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One peering bundle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringBundle {
    pub code: String,
    pub title: String,
    pub external_ref: String,
    pub namespace: String,
    pub ratio: f64,
    pub currency: String,
    pub quantity: i64,
    pub status: PeeringBundleStatus,
}

impl PeeringBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringBundleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringBundleStatus) -> bool {
        match self.status {
            PeeringBundleStatus::Draft => {
                return next == PeeringBundleStatus::Closed;
            }
            PeeringBundleStatus::Pending => {
                return next == PeeringBundleStatus::Active || next == PeeringBundleStatus::Closed;
            }
            PeeringBundleStatus::Active => {
                match next {
                    PeeringBundleStatus::Suspended | PeeringBundleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringBundleStatus::Suspended => {
                return next == PeeringBundleStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PeeringBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_load(&self) -> f64 {
        (self.ratio * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringBundle {
        PeeringBundle {
            code: self.code.clone(),
            title: self.title.clone(),
            external_ref: self.external_ref.clone(),
            namespace: self.namespace.clone(),
            ratio: self.ratio,
            currency: self.currency.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a peering bundle in status draft.
pub fn new_peering_bundle(code: &str, title: &str) -> PeeringBundle {
    PeeringBundle {
        code: code.to_string(),
        title: title.to_string(),
        external_ref: "".to_string(),
        namespace: "".to_string(),
        ratio: 0.0,
        currency: "".to_string(),
        quantity: 0,
        status: PeeringBundleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_peering_bundle("code-121", "title-924");
        assert_eq!(item.status, PeeringBundleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_peering_bundle("code-121", "title-924");
        if item.move_to(PeeringBundleStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringBundleStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_peering_bundle("code-121", "title-924");
        item.status = PeeringBundleStatus::Closed;
        assert!(
            item.move_to(PeeringBundleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
