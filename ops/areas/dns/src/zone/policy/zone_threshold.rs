use super::*;

use acme_kit::text;

/// Where a zone threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneThresholdStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ZoneThresholdStatus {
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

impl std::fmt::Display for ZoneThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One zone threshold, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneThreshold {
    pub code: String,
    pub label: String,
    pub namespace: String,
    pub owner_id: String,
    pub principal: String,
    pub quantity: i64,
    pub status: ZoneThresholdStatus,
}

impl ZoneThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneThresholdStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneThresholdStatus) -> bool {
        match self.status {
            ZoneThresholdStatus::Draft => {
                match next {
                    ZoneThresholdStatus::Pending | ZoneThresholdStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneThresholdStatus::Pending => {
                return next == ZoneThresholdStatus::Active || next == ZoneThresholdStatus::Closed;
            }
            ZoneThresholdStatus::Active => {
                return next == ZoneThresholdStatus::Suspended || next == ZoneThresholdStatus::Closed;
            }
            ZoneThresholdStatus::Suspended => {
                return next == ZoneThresholdStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ZoneThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> ZoneThreshold {
        ZoneThreshold {
            code: self.code.clone(),
            label: self.label.clone(),
            namespace: self.namespace.clone(),
            owner_id: self.owner_id.clone(),
            principal: self.principal.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a zone threshold in status draft.
pub fn new_zone_threshold(code: &str, label: &str) -> ZoneThreshold {
    ZoneThreshold {
        code: code.to_string(),
        label: label.to_string(),
        namespace: "".to_string(),
        owner_id: "".to_string(),
        principal: "".to_string(),
        quantity: 0,
        status: ZoneThresholdStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_zone_threshold("code-148", "label-19");
        assert_eq!(item.status, ZoneThresholdStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_zone_threshold("code-148", "label-19");
        if item.move_to(ZoneThresholdStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ZoneThresholdStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_zone_threshold("code-148", "label-19");
        item.status = ZoneThresholdStatus::Closed;
        assert!(
            item.move_to(ZoneThresholdStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
