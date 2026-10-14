use super::*;

use acme_kit::text;

/// Where a allowlist template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistTemplateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl AllowlistTemplateStatus {
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

impl std::fmt::Display for AllowlistTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A allowlist template as the allowlist team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistTemplate {
    pub code: String,
    pub name: String,
    pub host: String,
    pub starts_at: i64,
    pub channel: String,
    pub locale: String,
    pub floor_value: f64,
    pub segment: String,
    pub expires_at: i64,
    pub source: String,
    pub status: AllowlistTemplateStatus,
}

impl AllowlistTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistTemplateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistTemplateStatus) -> bool {
        match self.status {
            AllowlistTemplateStatus::Idle => {
                match next {
                    AllowlistTemplateStatus::Draining | AllowlistTemplateStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistTemplateStatus::Warming => {
                match next {
                    AllowlistTemplateStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistTemplateStatus::Ready => {
                match next {
                    AllowlistTemplateStatus::Draining | AllowlistTemplateStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistTemplateStatus::Draining => {
                return next == AllowlistTemplateStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AllowlistTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist template {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.expires_at as f64 + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AllowlistTemplate {
        AllowlistTemplate {
            code: self.code.clone(),
            name: self.name.clone(),
            host: self.host.clone(),
            starts_at: self.starts_at,
            channel: self.channel.clone(),
            locale: self.locale.clone(),
            floor_value: self.floor_value,
            segment: self.segment.clone(),
            expires_at: self.expires_at,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a allowlist template in status idle.
pub fn new_allowlist_template(code: &str, name: &str) -> AllowlistTemplate {
    AllowlistTemplate {
        code: code.to_string(),
        name: name.to_string(),
        host: "".to_string(),
        starts_at: 0,
        channel: "".to_string(),
        locale: "".to_string(),
        floor_value: 0.0,
        segment: "".to_string(),
        expires_at: 0,
        source: "".to_string(),
        status: AllowlistTemplateStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_allowlist_template("code-634", "name-672");
        assert_eq!(item.status, AllowlistTemplateStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_allowlist_template("code-634", "name-672");
        if item.move_to(AllowlistTemplateStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AllowlistTemplateStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_allowlist_template("code-634", "name-672");
        item.status = AllowlistTemplateStatus::Stopped;
        assert!(
            item.move_to(AllowlistTemplateStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
