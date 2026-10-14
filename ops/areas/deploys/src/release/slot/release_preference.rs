use super::*;

use acme_kit::text;

/// Where a release preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleasePreferenceStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ReleasePreferenceStatus {
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

impl std::fmt::Display for ReleasePreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One release preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleasePreference {
    pub code: String,
    pub label: String,
    pub phone: String,
    pub owner_id: String,
    pub tags: Vec<String>,
    pub principal: String,
    pub region: String,
    pub active: bool,
    pub version: i64,
    pub approved: bool,
    pub duration_minutes: i64,
    pub status: ReleasePreferenceStatus,
}

impl ReleasePreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleasePreferenceStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleasePreferenceStatus) -> bool {
        match self.status {
            ReleasePreferenceStatus::Draft => {
                match next {
                    ReleasePreferenceStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleasePreferenceStatus::Pending => {
                return next == ReleasePreferenceStatus::Active || next == ReleasePreferenceStatus::Suspended;
            }
            ReleasePreferenceStatus::Active => {
                match next {
                    ReleasePreferenceStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleasePreferenceStatus::Suspended => {
                return next == ReleasePreferenceStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReleasePreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.version as f64 + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReleasePreference {
        ReleasePreference {
            code: self.code.clone(),
            label: self.label.clone(),
            phone: self.phone.clone(),
            owner_id: self.owner_id.clone(),
            tags: self.tags.clone(),
            principal: self.principal.clone(),
            region: self.region.clone(),
            active: self.active,
            version: self.version,
            approved: self.approved,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a release preference in status draft.
pub fn new_release_preference(code: &str, label: &str) -> ReleasePreference {
    ReleasePreference {
        code: code.to_string(),
        label: label.to_string(),
        phone: "".to_string(),
        owner_id: "".to_string(),
        tags: Vec::new(),
        principal: "".to_string(),
        region: "".to_string(),
        active: false,
        version: 0,
        approved: false,
        duration_minutes: 0,
        status: ReleasePreferenceStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_release_preference("code-916", "label-168");
        assert_eq!(item.status, ReleasePreferenceStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_release_preference("code-916", "label-168");
        if item.move_to(ReleasePreferenceStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReleasePreferenceStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_release_preference("code-916", "label-168");
        item.status = ReleasePreferenceStatus::Closed;
        assert!(
            item.move_to(ReleasePreferenceStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
