use super::*;

use acme_kit::text;

/// Where a postmortem forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemForecastStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PostmortemForecastStatus {
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

impl std::fmt::Display for PostmortemForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One postmortem forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemForecast {
    pub code: String,
    pub title: String,
    pub margin: f64,
    pub label: String,
    pub amount: f64,
    pub owner_id: String,
    pub status: PostmortemForecastStatus,
}

impl PostmortemForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemForecastStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemForecastStatus) -> bool {
        match self.status {
            PostmortemForecastStatus::Idle => {
                match next {
                    PostmortemForecastStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemForecastStatus::Warming => {
                return next == PostmortemForecastStatus::Draining || next == PostmortemForecastStatus::Stopped;
            }
            PostmortemForecastStatus::Ready => {
                match next {
                    PostmortemForecastStatus::Draining | PostmortemForecastStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemForecastStatus::Draining => {
                return next == PostmortemForecastStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PostmortemForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        (self.margin * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemForecast {
        PostmortemForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            margin: self.margin,
            label: self.label.clone(),
            amount: self.amount,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a postmortem forecast in status idle.
pub fn new_postmortem_forecast(code: &str, title: &str) -> PostmortemForecast {
    PostmortemForecast {
        code: code.to_string(),
        title: title.to_string(),
        margin: 0.0,
        label: "".to_string(),
        amount: 0.0,
        owner_id: "".to_string(),
        status: PostmortemForecastStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_postmortem_forecast("code-321", "title-824");
        assert_eq!(item.status, PostmortemForecastStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_postmortem_forecast("code-321", "title-824");
        if item.move_to(PostmortemForecastStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PostmortemForecastStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_postmortem_forecast("code-321", "title-824");
        item.status = PostmortemForecastStatus::Stopped;
        assert!(
            item.move_to(PostmortemForecastStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
