use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a fallback release template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackReleaseTemplateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FallbackReleaseTemplateStatus {
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

impl std::fmt::Display for FallbackReleaseTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback release template as the release team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReleaseTemplate {
    pub code: String,
    pub label: String,
    pub namespace: String,
    pub ends_at: i64,
    pub description: String,
    pub active: bool,
    pub cost: f64,
    pub attributes: BTreeMap<String, String>,
    pub retries: i64,
    pub score: f64,
    pub ceiling: f64,
    pub status: FallbackReleaseTemplateStatus,
}

impl FallbackReleaseTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackReleaseTemplateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackReleaseTemplateStatus) -> bool {
        match self.status {
            FallbackReleaseTemplateStatus::Idle => {
                match next {
                    FallbackReleaseTemplateStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackReleaseTemplateStatus::Warming => {
                match next {
                    FallbackReleaseTemplateStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackReleaseTemplateStatus::Ready => {
                match next {
                    FallbackReleaseTemplateStatus::Draining | FallbackReleaseTemplateStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackReleaseTemplateStatus::Draining => {
                match next {
                    FallbackReleaseTemplateStatus::Stopped => {
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

    pub fn move_to(&mut self, next: FallbackReleaseTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback release template {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.label)
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

    pub fn net_exposure(&self) -> f64 {
        (self.score * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackReleaseTemplate {
        FallbackReleaseTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            namespace: self.namespace.clone(),
            ends_at: self.ends_at,
            description: self.description.clone(),
            active: self.active,
            cost: self.cost,
            attributes: self.attributes.clone(),
            retries: self.retries,
            score: self.score,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a fallback release template in status idle.
pub fn new_fallback_release_template(code: &str, label: &str) -> FallbackReleaseTemplate {
    FallbackReleaseTemplate {
        code: code.to_string(),
        label: label.to_string(),
        namespace: "".to_string(),
        ends_at: 0,
        description: "".to_string(),
        active: false,
        cost: 0.0,
        attributes: BTreeMap::new(),
        retries: 0,
        score: 0.0,
        ceiling: 0.0,
        status: FallbackReleaseTemplateStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_fallback_release_template("code-502", "label-961");
        assert_eq!(item.status, FallbackReleaseTemplateStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_fallback_release_template("code-502", "label-961");
        if item.move_to(FallbackReleaseTemplateStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackReleaseTemplateStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_fallback_release_template("code-502", "label-961");
        item.status = FallbackReleaseTemplateStatus::Stopped;
        assert!(
            item.move_to(FallbackReleaseTemplateStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
