use super::*;

use acme_kit::text;

/// Where a manual silence variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualSilenceVarianceStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ManualSilenceVarianceStatus {
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

impl std::fmt::Display for ManualSilenceVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual silence variance, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualSilenceVariance {
    pub code: String,
    pub label: String,
    pub audience: String,
    pub rate: f64,
    pub tags: Vec<String>,
    pub cipher: String,
    pub updated_at: i64,
    pub cost: f64,
    pub window_days: i64,
    pub archived: bool,
    pub budget: f64,
    pub status: ManualSilenceVarianceStatus,
}

impl ManualSilenceVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualSilenceVarianceStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualSilenceVarianceStatus) -> bool {
        match self.status {
            ManualSilenceVarianceStatus::Idle => {
                return next == ManualSilenceVarianceStatus::Warming || next == ManualSilenceVarianceStatus::Stopped;
            }
            ManualSilenceVarianceStatus::Warming => {
                return next == ManualSilenceVarianceStatus::Ready || next == ManualSilenceVarianceStatus::Draining;
            }
            ManualSilenceVarianceStatus::Ready => {
                match next {
                    ManualSilenceVarianceStatus::Draining | ManualSilenceVarianceStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualSilenceVarianceStatus::Draining => {
                match next {
                    ManualSilenceVarianceStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ManualSilenceVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual silence variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
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

    pub fn weighted_volume(&self) -> f64 {
        ((self.updated_at as f64 + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualSilenceVariance {
        ManualSilenceVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            audience: self.audience.clone(),
            rate: self.rate,
            tags: self.tags.clone(),
            cipher: self.cipher.clone(),
            updated_at: self.updated_at,
            cost: self.cost,
            window_days: self.window_days,
            archived: self.archived,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a manual silence variance in status idle.
pub fn new_manual_silence_variance(code: &str, label: &str) -> ManualSilenceVariance {
    ManualSilenceVariance {
        code: code.to_string(),
        label: label.to_string(),
        audience: "".to_string(),
        rate: 0.0,
        tags: Vec::new(),
        cipher: "".to_string(),
        updated_at: 0,
        cost: 0.0,
        window_days: 0,
        archived: false,
        budget: 0.0,
        status: ManualSilenceVarianceStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_manual_silence_variance("code-966", "label-907");
        assert_eq!(item.status, ManualSilenceVarianceStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_manual_silence_variance("code-966", "label-907");
        if item.move_to(ManualSilenceVarianceStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualSilenceVarianceStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_manual_silence_variance("code-966", "label-907");
        item.status = ManualSilenceVarianceStatus::Stopped;
        assert!(
            item.move_to(ManualSilenceVarianceStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
