use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a default freeze profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultFreezeProfileStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl DefaultFreezeProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for DefaultFreezeProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default freeze profile, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultFreezeProfile {
    pub code: String,
    pub name: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub external_ref: String,
    pub score: f64,
    pub retries: i64,
    pub contact_email: String,
    pub fingerprint: String,
    pub ends_at: i64,
    pub quantity: i64,
    pub status: DefaultFreezeProfileStatus,
}

impl DefaultFreezeProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultFreezeProfileStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultFreezeProfileStatus) -> bool {
        match self.status {
            DefaultFreezeProfileStatus::Created => {
                return next == DefaultFreezeProfileStatus::Published || next == DefaultFreezeProfileStatus::Retired;
            }
            DefaultFreezeProfileStatus::Validated => {
                match next {
                    DefaultFreezeProfileStatus::Published | DefaultFreezeProfileStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultFreezeProfileStatus::Published => {
                return next == DefaultFreezeProfileStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultFreezeProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default freeze profile {} cannot move from {} to {}",
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
        format!("{} {}", "default freeze profile", self.code)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn net_volume(&self) -> f64 {
        ((self.retries as f64) * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultFreezeProfile {
        DefaultFreezeProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            external_ref: self.external_ref.clone(),
            score: self.score,
            retries: self.retries,
            contact_email: self.contact_email.clone(),
            fingerprint: self.fingerprint.clone(),
            ends_at: self.ends_at,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a default freeze profile in status created.
pub fn new_default_freeze_profile(code: &str, name: &str) -> DefaultFreezeProfile {
    DefaultFreezeProfile {
        code: code.to_string(),
        name: name.to_string(),
        label: "".to_string(),
        attributes: BTreeMap::new(),
        external_ref: "".to_string(),
        score: 0.0,
        retries: 0,
        contact_email: "".to_string(),
        fingerprint: "".to_string(),
        ends_at: 0,
        quantity: 0,
        status: DefaultFreezeProfileStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_default_freeze_profile("code-651", "name-124");
        assert_eq!(item.status, DefaultFreezeProfileStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_default_freeze_profile("code-651", "name-124");
        if item.move_to(DefaultFreezeProfileStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultFreezeProfileStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_default_freeze_profile("code-651", "name-124");
        item.status = DefaultFreezeProfileStatus::Retired;
        assert!(
            item.move_to(DefaultFreezeProfileStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
