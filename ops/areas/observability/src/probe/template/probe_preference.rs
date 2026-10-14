use super::*;

use acme_kit::text;

/// Where a probe preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbePreferenceStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ProbePreferenceStatus {
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

impl std::fmt::Display for ProbePreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a probe preference and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbePreference {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub maximum: f64,
    pub priority: i64,
    pub scope: String,
    pub external_ref: String,
    pub active: bool,
    pub key_id: String,
    pub version: i64,
    pub status: ProbePreferenceStatus,
}

impl ProbePreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbePreferenceStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbePreferenceStatus) -> bool {
        match self.status {
            ProbePreferenceStatus::Draft => {
                match next {
                    ProbePreferenceStatus::Pending | ProbePreferenceStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbePreferenceStatus::Pending => {
                return next == ProbePreferenceStatus::Active;
            }
            ProbePreferenceStatus::Active => {
                match next {
                    ProbePreferenceStatus::Suspended | ProbePreferenceStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbePreferenceStatus::Suspended => {
                match next {
                    ProbePreferenceStatus::Closed => {
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

    pub fn move_to(&mut self, next: ProbePreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "probe preference", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.maximum + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbePreference {
        ProbePreference {
            code: self.code.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            maximum: self.maximum,
            priority: self.priority,
            scope: self.scope.clone(),
            external_ref: self.external_ref.clone(),
            active: self.active,
            key_id: self.key_id.clone(),
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a probe preference in status draft.
pub fn new_probe_preference(code: &str, name: &str) -> ProbePreference {
    ProbePreference {
        code: code.to_string(),
        name: name.to_string(),
        summary: "".to_string(),
        maximum: 0.0,
        priority: 0,
        scope: "".to_string(),
        external_ref: "".to_string(),
        active: false,
        key_id: "".to_string(),
        version: 0,
        status: ProbePreferenceStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_probe_preference("code-409", "name-3");
        assert_eq!(item.status, ProbePreferenceStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_probe_preference("code-409", "name-3");
        if item.move_to(ProbePreferenceStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ProbePreferenceStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_probe_preference("code-409", "name-3");
        item.status = ProbePreferenceStatus::Closed;
        assert!(
            item.move_to(ProbePreferenceStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
