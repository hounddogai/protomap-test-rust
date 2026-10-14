use super::*;

use acme_kit::text;

/// Where a strict escalation tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictEscalationTierStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl StrictEscalationTierStatus {
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

impl std::fmt::Display for StrictEscalationTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict escalation tier and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictEscalationTier {
    pub code: String,
    pub label: String,
    pub revision: i64,
    pub updated_at: i64,
    pub segment: String,
    pub description: String,
    pub owner_id: String,
    pub status: StrictEscalationTierStatus,
}

impl StrictEscalationTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictEscalationTierStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictEscalationTierStatus) -> bool {
        match self.status {
            StrictEscalationTierStatus::Draft => {
                match next {
                    StrictEscalationTierStatus::Pending | StrictEscalationTierStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictEscalationTierStatus::Pending => {
                match next {
                    StrictEscalationTierStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictEscalationTierStatus::Active => {
                match next {
                    StrictEscalationTierStatus::Suspended | StrictEscalationTierStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictEscalationTierStatus::Suspended => {
                match next {
                    StrictEscalationTierStatus::Closed => {
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

    pub fn move_to(&mut self, next: StrictEscalationTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict escalation tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.revision as f64 + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictEscalationTier {
        StrictEscalationTier {
            code: self.code.clone(),
            label: self.label.clone(),
            revision: self.revision,
            updated_at: self.updated_at,
            segment: self.segment.clone(),
            description: self.description.clone(),
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a strict escalation tier in status draft.
pub fn new_strict_escalation_tier(code: &str, label: &str) -> StrictEscalationTier {
    StrictEscalationTier {
        code: code.to_string(),
        label: label.to_string(),
        revision: 0,
        updated_at: 0,
        segment: "".to_string(),
        description: "".to_string(),
        owner_id: "".to_string(),
        status: StrictEscalationTierStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_strict_escalation_tier("code-298", "label-835");
        assert_eq!(item.status, StrictEscalationTierStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_strict_escalation_tier("code-298", "label-835");
        if item.move_to(StrictEscalationTierStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictEscalationTierStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_strict_escalation_tier("code-298", "label-835");
        item.status = StrictEscalationTierStatus::Closed;
        assert!(
            item.move_to(StrictEscalationTierStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
