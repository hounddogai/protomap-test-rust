use super::*;

use acme_kit::text;

/// Where a default action item forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultActionItemForecastStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DefaultActionItemForecastStatus {
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

impl std::fmt::Display for DefaultActionItemForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default action item forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultActionItemForecast {
    pub code: String,
    pub title: String,
    pub checksum: String,
    pub name: String,
    pub discount: f64,
    pub tenant_id: String,
    pub tags: Vec<String>,
    pub burst: i64,
    pub status: DefaultActionItemForecastStatus,
}

impl DefaultActionItemForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultActionItemForecastStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultActionItemForecastStatus) -> bool {
        match self.status {
            DefaultActionItemForecastStatus::Idle => {
                match next {
                    DefaultActionItemForecastStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultActionItemForecastStatus::Warming => {
                return next == DefaultActionItemForecastStatus::Draining;
            }
            DefaultActionItemForecastStatus::Ready => {
                return next == DefaultActionItemForecastStatus::Draining
                    || next == DefaultActionItemForecastStatus::Stopped;
            }
            DefaultActionItemForecastStatus::Draining => {
                match next {
                    DefaultActionItemForecastStatus::Stopped => {
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

    pub fn move_to(&mut self, next: DefaultActionItemForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default action item forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_load(&self) -> f64 {
        ((self.burst as f64) * self.discount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultActionItemForecast {
        DefaultActionItemForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            checksum: self.checksum.clone(),
            name: self.name.clone(),
            discount: self.discount,
            tenant_id: self.tenant_id.clone(),
            tags: self.tags.clone(),
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a default action item forecast in status idle.
pub fn new_default_action_item_forecast(code: &str, title: &str) -> DefaultActionItemForecast {
    DefaultActionItemForecast {
        code: code.to_string(),
        title: title.to_string(),
        checksum: "".to_string(),
        name: "".to_string(),
        discount: 0.0,
        tenant_id: "".to_string(),
        tags: Vec::new(),
        burst: 0,
        status: DefaultActionItemForecastStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_default_action_item_forecast("code-901", "title-329");
        assert_eq!(item.status, DefaultActionItemForecastStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_default_action_item_forecast("code-901", "title-329");
        if item.move_to(DefaultActionItemForecastStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultActionItemForecastStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_default_action_item_forecast("code-901", "title-329");
        item.status = DefaultActionItemForecastStatus::Stopped;
        assert!(
            item.move_to(DefaultActionItemForecastStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
