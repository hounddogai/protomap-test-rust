use super::*;

use acme_kit::text;

/// Where a primary peering tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryPeeringTierStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PrimaryPeeringTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for PrimaryPeeringTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary peering tier and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPeeringTier {
    pub code: String,
    pub label: String,
    pub segment: String,
    pub budget: f64,
    pub channel: String,
    pub discount: f64,
    pub key_id: String,
    pub phone: String,
    pub maximum: f64,
    pub weight: f64,
    pub status: PrimaryPeeringTierStatus,
}

impl PrimaryPeeringTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryPeeringTierStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryPeeringTierStatus) -> bool {
        match self.status {
            PrimaryPeeringTierStatus::New => {
                match next {
                    PrimaryPeeringTierStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryPeeringTierStatus::Queued => {
                match next {
                    PrimaryPeeringTierStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryPeeringTierStatus::Running => {
                return next == PrimaryPeeringTierStatus::Done || next == PrimaryPeeringTierStatus::Failed;
            }
            PrimaryPeeringTierStatus::Done => {
                match next {
                    PrimaryPeeringTierStatus::Failed => {
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

    pub fn move_to(&mut self, next: PrimaryPeeringTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary peering tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "primary peering tier", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        (self.maximum * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryPeeringTier {
        PrimaryPeeringTier {
            code: self.code.clone(),
            label: self.label.clone(),
            segment: self.segment.clone(),
            budget: self.budget,
            channel: self.channel.clone(),
            discount: self.discount,
            key_id: self.key_id.clone(),
            phone: self.phone.clone(),
            maximum: self.maximum,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a primary peering tier in status new.
pub fn new_primary_peering_tier(code: &str, label: &str) -> PrimaryPeeringTier {
    PrimaryPeeringTier {
        code: code.to_string(),
        label: label.to_string(),
        segment: "".to_string(),
        budget: 0.0,
        channel: "".to_string(),
        discount: 0.0,
        key_id: "".to_string(),
        phone: "".to_string(),
        maximum: 0.0,
        weight: 0.0,
        status: PrimaryPeeringTierStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_primary_peering_tier("code-164", "label-859");
        assert_eq!(item.status, PrimaryPeeringTierStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_primary_peering_tier("code-164", "label-859");
        if item.move_to(PrimaryPeeringTierStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryPeeringTierStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_primary_peering_tier("code-164", "label-859");
        item.status = PrimaryPeeringTierStatus::Failed;
        assert!(
            item.move_to(PrimaryPeeringTierStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
