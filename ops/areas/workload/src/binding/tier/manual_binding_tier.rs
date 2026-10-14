use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a manual binding tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualBindingTierStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ManualBindingTierStatus {
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

impl std::fmt::Display for ManualBindingTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual binding tier and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualBindingTier {
    pub code: String,
    pub label: String,
    pub starts_at: i64,
    pub tags: Vec<String>,
    pub unit: String,
    pub tenant_id: String,
    pub rate: f64,
    pub name: String,
    pub threshold: f64,
    pub reviewer: String,
    pub attributes: BTreeMap<String, String>,
    pub status: ManualBindingTierStatus,
}

impl ManualBindingTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualBindingTierStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualBindingTierStatus) -> bool {
        match self.status {
            ManualBindingTierStatus::Idle => {
                return next == ManualBindingTierStatus::Stopped;
            }
            ManualBindingTierStatus::Warming => {
                return next == ManualBindingTierStatus::Ready || next == ManualBindingTierStatus::Draining;
            }
            ManualBindingTierStatus::Ready => {
                return next == ManualBindingTierStatus::Draining || next == ManualBindingTierStatus::Stopped;
            }
            ManualBindingTierStatus::Draining => {
                return next == ManualBindingTierStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualBindingTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual binding tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
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

    pub fn net_value(&self) -> f64 {
        ((self.threshold - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualBindingTier {
        ManualBindingTier {
            code: self.code.clone(),
            label: self.label.clone(),
            starts_at: self.starts_at,
            tags: self.tags.clone(),
            unit: self.unit.clone(),
            tenant_id: self.tenant_id.clone(),
            rate: self.rate,
            name: self.name.clone(),
            threshold: self.threshold,
            reviewer: self.reviewer.clone(),
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a manual binding tier in status idle.
pub fn new_manual_binding_tier(code: &str, label: &str) -> ManualBindingTier {
    ManualBindingTier {
        code: code.to_string(),
        label: label.to_string(),
        starts_at: 0,
        tags: Vec::new(),
        unit: "".to_string(),
        tenant_id: "".to_string(),
        rate: 0.0,
        name: "".to_string(),
        threshold: 0.0,
        reviewer: "".to_string(),
        attributes: BTreeMap::new(),
        status: ManualBindingTierStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_manual_binding_tier("code-981", "label-524");
        assert_eq!(item.status, ManualBindingTierStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_manual_binding_tier("code-981", "label-524");
        if item.move_to(ManualBindingTierStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualBindingTierStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_manual_binding_tier("code-981", "label-524");
        item.status = ManualBindingTierStatus::Stopped;
        assert!(
            item.move_to(ManualBindingTierStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
