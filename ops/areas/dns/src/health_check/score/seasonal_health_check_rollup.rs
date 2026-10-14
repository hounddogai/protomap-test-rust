use super::*;

use acme_kit::text;

/// Where a seasonal health check rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalHealthCheckRollupStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SeasonalHealthCheckRollupStatus {
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

impl std::fmt::Display for SeasonalHealthCheckRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal health check rollup. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalHealthCheckRollup {
    pub code: String,
    pub title: String,
    pub margin: f64,
    pub host: String,
    pub issuer: String,
    pub duration_minutes: i64,
    pub tags: Vec<String>,
    pub visible: bool,
    pub source: String,
    pub cipher: String,
    pub status: SeasonalHealthCheckRollupStatus,
}

impl SeasonalHealthCheckRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalHealthCheckRollupStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalHealthCheckRollupStatus) -> bool {
        match self.status {
            SeasonalHealthCheckRollupStatus::Idle => {
                match next {
                    SeasonalHealthCheckRollupStatus::Ready | SeasonalHealthCheckRollupStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalHealthCheckRollupStatus::Warming => {
                return next == SeasonalHealthCheckRollupStatus::Draining;
            }
            SeasonalHealthCheckRollupStatus::Ready => {
                return next == SeasonalHealthCheckRollupStatus::Draining;
            }
            SeasonalHealthCheckRollupStatus::Draining => {
                match next {
                    SeasonalHealthCheckRollupStatus::Stopped => {
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

    pub fn move_to(&mut self, next: SeasonalHealthCheckRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal health check rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
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
        ((self.duration_minutes as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalHealthCheckRollup {
        SeasonalHealthCheckRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            margin: self.margin,
            host: self.host.clone(),
            issuer: self.issuer.clone(),
            duration_minutes: self.duration_minutes,
            tags: self.tags.clone(),
            visible: self.visible,
            source: self.source.clone(),
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a seasonal health check rollup in status idle.
pub fn new_seasonal_health_check_rollup(code: &str, title: &str) -> SeasonalHealthCheckRollup {
    SeasonalHealthCheckRollup {
        code: code.to_string(),
        title: title.to_string(),
        margin: 0.0,
        host: "".to_string(),
        issuer: "".to_string(),
        duration_minutes: 0,
        tags: Vec::new(),
        visible: false,
        source: "".to_string(),
        cipher: "".to_string(),
        status: SeasonalHealthCheckRollupStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_seasonal_health_check_rollup("code-147", "title-220");
        assert_eq!(item.status, SeasonalHealthCheckRollupStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_seasonal_health_check_rollup("code-147", "title-220");
        if item.move_to(SeasonalHealthCheckRollupStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalHealthCheckRollupStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_seasonal_health_check_rollup("code-147", "title-220");
        item.status = SeasonalHealthCheckRollupStatus::Stopped;
        assert!(
            item.move_to(SeasonalHealthCheckRollupStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
