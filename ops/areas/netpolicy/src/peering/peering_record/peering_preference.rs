use super::*;

use acme_kit::text;

/// Where a peering preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringPreferenceStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PeeringPreferenceStatus {
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

impl std::fmt::Display for PeeringPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One peering preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringPreference {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub ceiling: f64,
    pub notes: String,
    pub timezone: String,
    pub external_ref: String,
    pub summary: String,
    pub burst: i64,
    pub cost: f64,
    pub status: PeeringPreferenceStatus,
}

impl PeeringPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringPreferenceStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringPreferenceStatus) -> bool {
        match self.status {
            PeeringPreferenceStatus::Draft => {
                return next == PeeringPreferenceStatus::Suspended;
            }
            PeeringPreferenceStatus::Pending => {
                match next {
                    PeeringPreferenceStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringPreferenceStatus::Active => {
                match next {
                    PeeringPreferenceStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringPreferenceStatus::Suspended => {
                return next == PeeringPreferenceStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PeeringPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.port as f64) * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringPreference {
        PeeringPreference {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            ceiling: self.ceiling,
            notes: self.notes.clone(),
            timezone: self.timezone.clone(),
            external_ref: self.external_ref.clone(),
            summary: self.summary.clone(),
            burst: self.burst,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a peering preference in status draft.
pub fn new_peering_preference(code: &str, name: &str) -> PeeringPreference {
    PeeringPreference {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        ceiling: 0.0,
        notes: "".to_string(),
        timezone: "".to_string(),
        external_ref: "".to_string(),
        summary: "".to_string(),
        burst: 0,
        cost: 0.0,
        status: PeeringPreferenceStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_peering_preference("code-132", "name-950");
        assert_eq!(item.status, PeeringPreferenceStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_peering_preference("code-132", "name-950");
        if item.move_to(PeeringPreferenceStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringPreferenceStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_peering_preference("code-132", "name-950");
        item.status = PeeringPreferenceStatus::Closed;
        assert!(
            item.move_to(PeeringPreferenceStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
