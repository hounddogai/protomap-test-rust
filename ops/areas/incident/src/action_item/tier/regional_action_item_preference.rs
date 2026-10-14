use super::*;

use acme_kit::text;

/// Where a regional action item preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalActionItemPreferenceStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionalActionItemPreferenceStatus {
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

impl std::fmt::Display for RegionalActionItemPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional action item preference. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalActionItemPreference {
    pub code: String,
    pub name: String,
    pub region: String,
    pub issuer: String,
    pub threshold: f64,
    pub weight: f64,
    pub title: String,
    pub label: String,
    pub visible: bool,
    pub quota_units: i64,
    pub reviewer: String,
    pub status: RegionalActionItemPreferenceStatus,
}

impl RegionalActionItemPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalActionItemPreferenceStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalActionItemPreferenceStatus) -> bool {
        match self.status {
            RegionalActionItemPreferenceStatus::Idle => {
                match next {
                    RegionalActionItemPreferenceStatus::Ready | RegionalActionItemPreferenceStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalActionItemPreferenceStatus::Warming => {
                return next == RegionalActionItemPreferenceStatus::Ready
                    || next == RegionalActionItemPreferenceStatus::Draining;
            }
            RegionalActionItemPreferenceStatus::Ready => {
                match next {
                    RegionalActionItemPreferenceStatus::Draining | RegionalActionItemPreferenceStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalActionItemPreferenceStatus::Draining => {
                match next {
                    RegionalActionItemPreferenceStatus::Stopped => {
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

    pub fn move_to(&mut self, next: RegionalActionItemPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional action item preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        (self.weight * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalActionItemPreference {
        RegionalActionItemPreference {
            code: self.code.clone(),
            name: self.name.clone(),
            region: self.region.clone(),
            issuer: self.issuer.clone(),
            threshold: self.threshold,
            weight: self.weight,
            title: self.title.clone(),
            label: self.label.clone(),
            visible: self.visible,
            quota_units: self.quota_units,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a regional action item preference in status idle.
pub fn new_regional_action_item_preference(code: &str, name: &str) -> RegionalActionItemPreference {
    RegionalActionItemPreference {
        code: code.to_string(),
        name: name.to_string(),
        region: "".to_string(),
        issuer: "".to_string(),
        threshold: 0.0,
        weight: 0.0,
        title: "".to_string(),
        label: "".to_string(),
        visible: false,
        quota_units: 0,
        reviewer: "".to_string(),
        status: RegionalActionItemPreferenceStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_regional_action_item_preference("code-99", "name-628");
        assert_eq!(item.status, RegionalActionItemPreferenceStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_regional_action_item_preference("code-99", "name-628");
        if item.move_to(RegionalActionItemPreferenceStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalActionItemPreferenceStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_regional_action_item_preference("code-99", "name-628");
        item.status = RegionalActionItemPreferenceStatus::Stopped;
        assert!(
            item.move_to(RegionalActionItemPreferenceStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
