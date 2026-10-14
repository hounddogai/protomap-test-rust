use super::*;

use acme_kit::text;

/// Where a regional canary summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalCanarySummaryStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionalCanarySummaryStatus {
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

impl std::fmt::Display for RegionalCanarySummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional canary summary. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalCanarySummary {
    pub code: String,
    pub name: String,
    pub region: String,
    pub approved: bool,
    pub starts_at: i64,
    pub floor_value: f64,
    pub visible: bool,
    pub principal: String,
    pub country: String,
    pub status: RegionalCanarySummaryStatus,
}

impl RegionalCanarySummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalCanarySummaryStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalCanarySummaryStatus) -> bool {
        match self.status {
            RegionalCanarySummaryStatus::Idle => {
                return next == RegionalCanarySummaryStatus::Warming;
            }
            RegionalCanarySummaryStatus::Warming => {
                match next {
                    RegionalCanarySummaryStatus::Ready | RegionalCanarySummaryStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalCanarySummaryStatus::Ready => {
                return next == RegionalCanarySummaryStatus::Stopped;
            }
            RegionalCanarySummaryStatus::Draining => {
                return next == RegionalCanarySummaryStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalCanarySummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional canary summary {} cannot move from {} to {}",
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

    pub fn net_volume(&self) -> f64 {
        (self.floor_value * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalCanarySummary {
        RegionalCanarySummary {
            code: self.code.clone(),
            name: self.name.clone(),
            region: self.region.clone(),
            approved: self.approved,
            starts_at: self.starts_at,
            floor_value: self.floor_value,
            visible: self.visible,
            principal: self.principal.clone(),
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a regional canary summary in status idle.
pub fn new_regional_canary_summary(code: &str, name: &str) -> RegionalCanarySummary {
    RegionalCanarySummary {
        code: code.to_string(),
        name: name.to_string(),
        region: "".to_string(),
        approved: false,
        starts_at: 0,
        floor_value: 0.0,
        visible: false,
        principal: "".to_string(),
        country: "".to_string(),
        status: RegionalCanarySummaryStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_regional_canary_summary("code-709", "name-626");
        assert_eq!(item.status, RegionalCanarySummaryStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_regional_canary_summary("code-709", "name-626");
        if item.move_to(RegionalCanarySummaryStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalCanarySummaryStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_regional_canary_summary("code-709", "name-626");
        item.status = RegionalCanarySummaryStatus::Stopped;
        assert!(
            item.move_to(RegionalCanarySummaryStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
