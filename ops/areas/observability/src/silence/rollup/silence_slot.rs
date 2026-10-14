use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a silence slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceSlotStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SilenceSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SilenceSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A silence slot as the silence team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceSlot {
    pub code: String,
    pub label: String,
    pub ends_at: i64,
    pub version: i64,
    pub batch_size: i64,
    pub attributes: BTreeMap<String, String>,
    pub title: String,
    pub rotation_days: i64,
    pub archived: bool,
    pub fingerprint: String,
    pub margin: f64,
    pub status: SilenceSlotStatus,
}

impl SilenceSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceSlotStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceSlotStatus) -> bool {
        match self.status {
            SilenceSlotStatus::Requested => {
                return next == SilenceSlotStatus::Revoked;
            }
            SilenceSlotStatus::Granted => {
                return next == SilenceSlotStatus::Revoked;
            }
            SilenceSlotStatus::Revoked => {
                match next {
                    SilenceSlotStatus::Expired => {
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

    pub fn move_to(&mut self, next: SilenceSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
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

    pub fn effective_load(&self) -> f64 {
        (self.margin * (self.version as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SilenceSlot {
        SilenceSlot {
            code: self.code.clone(),
            label: self.label.clone(),
            ends_at: self.ends_at,
            version: self.version,
            batch_size: self.batch_size,
            attributes: self.attributes.clone(),
            title: self.title.clone(),
            rotation_days: self.rotation_days,
            archived: self.archived,
            fingerprint: self.fingerprint.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a silence slot in status requested.
pub fn new_silence_slot(code: &str, label: &str) -> SilenceSlot {
    SilenceSlot {
        code: code.to_string(),
        label: label.to_string(),
        ends_at: 0,
        version: 0,
        batch_size: 0,
        attributes: BTreeMap::new(),
        title: "".to_string(),
        rotation_days: 0,
        archived: false,
        fingerprint: "".to_string(),
        margin: 0.0,
        status: SilenceSlotStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_silence_slot("code-741", "label-526");
        assert_eq!(item.status, SilenceSlotStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_silence_slot("code-741", "label-526");
        if item.move_to(SilenceSlotStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SilenceSlotStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_silence_slot("code-741", "label-526");
        item.status = SilenceSlotStatus::Expired;
        assert!(
            item.move_to(SilenceSlotStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
