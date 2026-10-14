use super::*;

use acme_kit::text;

/// Where a seasonal federation target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalFederationTargetStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SeasonalFederationTargetStatus {
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

impl std::fmt::Display for SeasonalFederationTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal federation target. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFederationTarget {
    pub code: String,
    pub name: String,
    pub visible: bool,
    pub description: String,
    pub title: String,
    pub retries: i64,
    pub cluster: String,
    pub minimum: f64,
    pub label: String,
    pub tags: Vec<String>,
    pub duration_minutes: i64,
    pub status: SeasonalFederationTargetStatus,
}

impl SeasonalFederationTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalFederationTargetStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalFederationTargetStatus) -> bool {
        match self.status {
            SeasonalFederationTargetStatus::Idle => {
                match next {
                    SeasonalFederationTargetStatus::Ready | SeasonalFederationTargetStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalFederationTargetStatus::Warming => {
                match next {
                    SeasonalFederationTargetStatus::Draining | SeasonalFederationTargetStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalFederationTargetStatus::Ready => {
                match next {
                    SeasonalFederationTargetStatus::Draining | SeasonalFederationTargetStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalFederationTargetStatus::Draining => {
                match next {
                    SeasonalFederationTargetStatus::Stopped => {
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

    pub fn move_to(&mut self, next: SeasonalFederationTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal federation target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
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
        ((self.retries as f64 - self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalFederationTarget {
        SeasonalFederationTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            visible: self.visible,
            description: self.description.clone(),
            title: self.title.clone(),
            retries: self.retries,
            cluster: self.cluster.clone(),
            minimum: self.minimum,
            label: self.label.clone(),
            tags: self.tags.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a seasonal federation target in status idle.
pub fn new_seasonal_federation_target(code: &str, name: &str) -> SeasonalFederationTarget {
    SeasonalFederationTarget {
        code: code.to_string(),
        name: name.to_string(),
        visible: false,
        description: "".to_string(),
        title: "".to_string(),
        retries: 0,
        cluster: "".to_string(),
        minimum: 0.0,
        label: "".to_string(),
        tags: Vec::new(),
        duration_minutes: 0,
        status: SeasonalFederationTargetStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_seasonal_federation_target("code-38", "name-915");
        assert_eq!(item.status, SeasonalFederationTargetStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_seasonal_federation_target("code-38", "name-915");
        if item.move_to(SeasonalFederationTargetStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalFederationTargetStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_seasonal_federation_target("code-38", "name-915");
        item.status = SeasonalFederationTargetStatus::Stopped;
        assert!(
            item.move_to(SeasonalFederationTargetStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
