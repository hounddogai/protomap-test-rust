use super::*;

use acme_kit::text;

/// Where a binding preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingPreferenceStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BindingPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BindingPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One binding preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingPreference {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub phone: String,
    pub created_at: i64,
    pub score: f64,
    pub rotation_days: i64,
    pub cost: f64,
    pub status: BindingPreferenceStatus,
}

impl BindingPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingPreferenceStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingPreferenceStatus) -> bool {
        match self.status {
            BindingPreferenceStatus::Proposed => {
                return next == BindingPreferenceStatus::Approved || next == BindingPreferenceStatus::Withdrawn;
            }
            BindingPreferenceStatus::Approved => {
                match next {
                    BindingPreferenceStatus::Rejected | BindingPreferenceStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingPreferenceStatus::Rejected => {
                return next == BindingPreferenceStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BindingPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.score + (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingPreference {
        BindingPreference {
            code: self.code.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            phone: self.phone.clone(),
            created_at: self.created_at,
            score: self.score,
            rotation_days: self.rotation_days,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a binding preference in status proposed.
pub fn new_binding_preference(code: &str, name: &str) -> BindingPreference {
    BindingPreference {
        code: code.to_string(),
        name: name.to_string(),
        owner_id: "".to_string(),
        phone: "".to_string(),
        created_at: 0,
        score: 0.0,
        rotation_days: 0,
        cost: 0.0,
        status: BindingPreferenceStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_binding_preference("code-363", "name-469");
        assert_eq!(item.status, BindingPreferenceStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_binding_preference("code-363", "name-469");
        if item.move_to(BindingPreferenceStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BindingPreferenceStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_binding_preference("code-363", "name-469");
        item.status = BindingPreferenceStatus::Withdrawn;
        assert!(
            item.move_to(BindingPreferenceStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
