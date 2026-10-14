use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a weighted credential preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedCredentialPreferenceStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl WeightedCredentialPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for WeightedCredentialPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a weighted credential preference and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCredentialPreference {
    pub code: String,
    pub name: String,
    pub attributes: BTreeMap<String, String>,
    pub discount: f64,
    pub maximum: f64,
    pub priority: i64,
    pub expires_at: i64,
    pub status: WeightedCredentialPreferenceStatus,
}

impl WeightedCredentialPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedCredentialPreferenceStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedCredentialPreferenceStatus) -> bool {
        match self.status {
            WeightedCredentialPreferenceStatus::Idle => {
                match next {
                    WeightedCredentialPreferenceStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedCredentialPreferenceStatus::Warming => {
                match next {
                    WeightedCredentialPreferenceStatus::Ready | WeightedCredentialPreferenceStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedCredentialPreferenceStatus::Ready => {
                return next == WeightedCredentialPreferenceStatus::Draining;
            }
            WeightedCredentialPreferenceStatus::Draining => {
                match next {
                    WeightedCredentialPreferenceStatus::Stopped => {
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

    pub fn move_to(&mut self, next: WeightedCredentialPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted credential preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "weighted credential preference", self.code)
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

    pub fn adjusted_volume(&self) -> f64 {
        (self.maximum * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedCredentialPreference {
        WeightedCredentialPreference {
            code: self.code.clone(),
            name: self.name.clone(),
            attributes: self.attributes.clone(),
            discount: self.discount,
            maximum: self.maximum,
            priority: self.priority,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a weighted credential preference in status idle.
pub fn new_weighted_credential_preference(code: &str, name: &str) -> WeightedCredentialPreference {
    WeightedCredentialPreference {
        code: code.to_string(),
        name: name.to_string(),
        attributes: BTreeMap::new(),
        discount: 0.0,
        maximum: 0.0,
        priority: 0,
        expires_at: 0,
        status: WeightedCredentialPreferenceStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_weighted_credential_preference("code-562", "name-795");
        assert_eq!(item.status, WeightedCredentialPreferenceStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_weighted_credential_preference("code-562", "name-795");
        if item.move_to(WeightedCredentialPreferenceStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedCredentialPreferenceStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_weighted_credential_preference("code-562", "name-795");
        item.status = WeightedCredentialPreferenceStatus::Stopped;
        assert!(
            item.move_to(WeightedCredentialPreferenceStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
