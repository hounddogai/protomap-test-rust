use super::*;

use acme_kit::text;

/// Where a peering threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringThresholdStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PeeringThresholdStatus {
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

impl std::fmt::Display for PeeringThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One peering threshold, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringThreshold {
    pub code: String,
    pub title: String,
    pub burst: i64,
    pub key_id: String,
    pub quantity: i64,
    pub window_days: i64,
    pub limit: i64,
    pub status: PeeringThresholdStatus,
}

impl PeeringThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringThresholdStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringThresholdStatus) -> bool {
        match self.status {
            PeeringThresholdStatus::Draft => {
                return next == PeeringThresholdStatus::Active || next == PeeringThresholdStatus::Closed;
            }
            PeeringThresholdStatus::Pending => {
                match next {
                    PeeringThresholdStatus::Active | PeeringThresholdStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringThresholdStatus::Active => {
                match next {
                    PeeringThresholdStatus::Suspended | PeeringThresholdStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringThresholdStatus::Suspended => {
                return next == PeeringThresholdStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PeeringThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.quantity as f64 + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringThreshold {
        PeeringThreshold {
            code: self.code.clone(),
            title: self.title.clone(),
            burst: self.burst,
            key_id: self.key_id.clone(),
            quantity: self.quantity,
            window_days: self.window_days,
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a peering threshold in status draft.
pub fn new_peering_threshold(code: &str, title: &str) -> PeeringThreshold {
    PeeringThreshold {
        code: code.to_string(),
        title: title.to_string(),
        burst: 0,
        key_id: "".to_string(),
        quantity: 0,
        window_days: 0,
        limit: 0,
        status: PeeringThresholdStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_peering_threshold("code-365", "title-993");
        assert_eq!(item.status, PeeringThresholdStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_peering_threshold("code-365", "title-993");
        if item.move_to(PeeringThresholdStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringThresholdStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_peering_threshold("code-365", "title-993");
        item.status = PeeringThresholdStatus::Closed;
        assert!(
            item.move_to(PeeringThresholdStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
