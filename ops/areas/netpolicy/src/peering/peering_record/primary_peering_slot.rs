use super::*;

use acme_kit::text;

/// Where a primary peering slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryPeeringSlotStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PrimaryPeeringSlotStatus {
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

impl std::fmt::Display for PrimaryPeeringSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary peering slot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPeeringSlot {
    pub code: String,
    pub title: String,
    pub country: String,
    pub ends_at: i64,
    pub threshold: f64,
    pub label: String,
    pub channel: String,
    pub budget: f64,
    pub status: PrimaryPeeringSlotStatus,
}

impl PrimaryPeeringSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryPeeringSlotStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryPeeringSlotStatus) -> bool {
        match self.status {
            PrimaryPeeringSlotStatus::Draft => {
                return next == PrimaryPeeringSlotStatus::Pending;
            }
            PrimaryPeeringSlotStatus::Pending => {
                return next == PrimaryPeeringSlotStatus::Active || next == PrimaryPeeringSlotStatus::Closed;
            }
            PrimaryPeeringSlotStatus::Active => {
                return next == PrimaryPeeringSlotStatus::Suspended || next == PrimaryPeeringSlotStatus::Closed;
            }
            PrimaryPeeringSlotStatus::Suspended => {
                match next {
                    PrimaryPeeringSlotStatus::Closed => {
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

    pub fn move_to(&mut self, next: PrimaryPeeringSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary peering slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.budget - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryPeeringSlot {
        PrimaryPeeringSlot {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            ends_at: self.ends_at,
            threshold: self.threshold,
            label: self.label.clone(),
            channel: self.channel.clone(),
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a primary peering slot in status draft.
pub fn new_primary_peering_slot(code: &str, title: &str) -> PrimaryPeeringSlot {
    PrimaryPeeringSlot {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        ends_at: 0,
        threshold: 0.0,
        label: "".to_string(),
        channel: "".to_string(),
        budget: 0.0,
        status: PrimaryPeeringSlotStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_primary_peering_slot("code-662", "title-780");
        assert_eq!(item.status, PrimaryPeeringSlotStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_primary_peering_slot("code-662", "title-780");
        if item.move_to(PrimaryPeeringSlotStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryPeeringSlotStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_primary_peering_slot("code-662", "title-780");
        item.status = PrimaryPeeringSlotStatus::Closed;
        assert!(
            item.move_to(PrimaryPeeringSlotStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
