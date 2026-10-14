use super::*;

use acme_kit::text;

/// Where a seasonal token metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalTokenMetricStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SeasonalTokenMetricStatus {
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

impl std::fmt::Display for SeasonalTokenMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a seasonal token metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTokenMetric {
    pub code: String,
    pub title: String,
    pub maximum: f64,
    pub locale: String,
    pub checksum: String,
    pub cluster: String,
    pub issuer: String,
    pub status: SeasonalTokenMetricStatus,
}

impl SeasonalTokenMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalTokenMetricStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalTokenMetricStatus) -> bool {
        match self.status {
            SeasonalTokenMetricStatus::Idle => {
                match next {
                    SeasonalTokenMetricStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalTokenMetricStatus::Warming => {
                match next {
                    SeasonalTokenMetricStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalTokenMetricStatus::Ready => {
                match next {
                    SeasonalTokenMetricStatus::Draining | SeasonalTokenMetricStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalTokenMetricStatus::Draining => {
                match next {
                    SeasonalTokenMetricStatus::Stopped => {
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

    pub fn move_to(&mut self, next: SeasonalTokenMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal token metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.maximum
    }

    pub fn copy(&self) -> SeasonalTokenMetric {
        SeasonalTokenMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            maximum: self.maximum,
            locale: self.locale.clone(),
            checksum: self.checksum.clone(),
            cluster: self.cluster.clone(),
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a seasonal token metric in status idle.
pub fn new_seasonal_token_metric(code: &str, title: &str) -> SeasonalTokenMetric {
    SeasonalTokenMetric {
        code: code.to_string(),
        title: title.to_string(),
        maximum: 0.0,
        locale: "".to_string(),
        checksum: "".to_string(),
        cluster: "".to_string(),
        issuer: "".to_string(),
        status: SeasonalTokenMetricStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_seasonal_token_metric("code-737", "title-386");
        assert_eq!(item.status, SeasonalTokenMetricStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_seasonal_token_metric("code-737", "title-386");
        if item.move_to(SeasonalTokenMetricStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalTokenMetricStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_seasonal_token_metric("code-737", "title-386");
        item.status = SeasonalTokenMetricStatus::Stopped;
        assert!(
            item.move_to(SeasonalTokenMetricStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
