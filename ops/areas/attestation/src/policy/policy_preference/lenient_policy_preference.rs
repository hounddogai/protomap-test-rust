use super::*;

use acme_kit::text;

/// Where a lenient policy preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientPolicyPreferenceStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LenientPolicyPreferenceStatus {
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

impl std::fmt::Display for LenientPolicyPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One lenient policy preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientPolicyPreference {
    pub code: String,
    pub label: String,
    pub region: String,
    pub tags: Vec<String>,
    pub starts_at: i64,
    pub ends_at: i64,
    pub checksum: String,
    pub channel: String,
    pub ceiling: f64,
    pub locale: String,
    pub phone: String,
    pub status: LenientPolicyPreferenceStatus,
}

impl LenientPolicyPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientPolicyPreferenceStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientPolicyPreferenceStatus) -> bool {
        match self.status {
            LenientPolicyPreferenceStatus::Idle => {
                return next == LenientPolicyPreferenceStatus::Warming;
            }
            LenientPolicyPreferenceStatus::Warming => {
                return next == LenientPolicyPreferenceStatus::Ready || next == LenientPolicyPreferenceStatus::Stopped;
            }
            LenientPolicyPreferenceStatus::Ready => {
                match next {
                    LenientPolicyPreferenceStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientPolicyPreferenceStatus::Draining => {
                match next {
                    LenientPolicyPreferenceStatus::Stopped => {
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

    pub fn move_to(&mut self, next: LenientPolicyPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient policy preference {} cannot move from {} to {}",
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

    pub fn net_load(&self) -> f64 {
        ((self.ends_at as f64 - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientPolicyPreference {
        LenientPolicyPreference {
            code: self.code.clone(),
            label: self.label.clone(),
            region: self.region.clone(),
            tags: self.tags.clone(),
            starts_at: self.starts_at,
            ends_at: self.ends_at,
            checksum: self.checksum.clone(),
            channel: self.channel.clone(),
            ceiling: self.ceiling,
            locale: self.locale.clone(),
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a lenient policy preference in status idle.
pub fn new_lenient_policy_preference(code: &str, label: &str) -> LenientPolicyPreference {
    LenientPolicyPreference {
        code: code.to_string(),
        label: label.to_string(),
        region: "".to_string(),
        tags: Vec::new(),
        starts_at: 0,
        ends_at: 0,
        checksum: "".to_string(),
        channel: "".to_string(),
        ceiling: 0.0,
        locale: "".to_string(),
        phone: "".to_string(),
        status: LenientPolicyPreferenceStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_lenient_policy_preference("code-123", "label-71");
        assert_eq!(item.status, LenientPolicyPreferenceStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_lenient_policy_preference("code-123", "label-71");
        if item.move_to(LenientPolicyPreferenceStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientPolicyPreferenceStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_lenient_policy_preference("code-123", "label-71");
        item.status = LenientPolicyPreferenceStatus::Stopped;
        assert!(
            item.move_to(LenientPolicyPreferenceStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
