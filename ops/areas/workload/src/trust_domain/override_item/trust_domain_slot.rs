use super::*;

use acme_kit::text;

/// Where a trust domain slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainSlotStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TrustDomainSlotStatus {
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

impl std::fmt::Display for TrustDomainSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One trust domain slot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainSlot {
    pub code: String,
    pub label: String,
    pub tenant_id: String,
    pub checksum: String,
    pub key_id: String,
    pub host: String,
    pub description: String,
    pub margin: f64,
    pub status: TrustDomainSlotStatus,
}

impl TrustDomainSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainSlotStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainSlotStatus) -> bool {
        match self.status {
            TrustDomainSlotStatus::Draft => {
                match next {
                    TrustDomainSlotStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainSlotStatus::Pending => {
                return next == TrustDomainSlotStatus::Suspended;
            }
            TrustDomainSlotStatus::Active => {
                match next {
                    TrustDomainSlotStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainSlotStatus::Suspended => {
                return next == TrustDomainSlotStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TrustDomainSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "trust domain slot", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.margin
    }

    pub fn copy(&self) -> TrustDomainSlot {
        TrustDomainSlot {
            code: self.code.clone(),
            label: self.label.clone(),
            tenant_id: self.tenant_id.clone(),
            checksum: self.checksum.clone(),
            key_id: self.key_id.clone(),
            host: self.host.clone(),
            description: self.description.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a trust domain slot in status draft.
pub fn new_trust_domain_slot(code: &str, label: &str) -> TrustDomainSlot {
    TrustDomainSlot {
        code: code.to_string(),
        label: label.to_string(),
        tenant_id: "".to_string(),
        checksum: "".to_string(),
        key_id: "".to_string(),
        host: "".to_string(),
        description: "".to_string(),
        margin: 0.0,
        status: TrustDomainSlotStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_trust_domain_slot("code-80", "label-735");
        assert_eq!(item.status, TrustDomainSlotStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_trust_domain_slot("code-80", "label-735");
        if item.move_to(TrustDomainSlotStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainSlotStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_trust_domain_slot("code-80", "label-735");
        item.status = TrustDomainSlotStatus::Closed;
        assert!(
            item.move_to(TrustDomainSlotStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
