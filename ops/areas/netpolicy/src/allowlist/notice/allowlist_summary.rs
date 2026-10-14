use super::*;

use acme_kit::text;

/// Where a allowlist summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistSummaryStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl AllowlistSummaryStatus {
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

impl std::fmt::Display for AllowlistSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A allowlist summary as the allowlist team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistSummary {
    pub code: String,
    pub name: String,
    pub country: String,
    pub phone: String,
    pub cluster: String,
    pub source: String,
    pub score: f64,
    pub namespace: String,
    pub status: AllowlistSummaryStatus,
}

impl AllowlistSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistSummaryStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistSummaryStatus) -> bool {
        match self.status {
            AllowlistSummaryStatus::Idle => {
                match next {
                    AllowlistSummaryStatus::Warming | AllowlistSummaryStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistSummaryStatus::Warming => {
                match next {
                    AllowlistSummaryStatus::Ready | AllowlistSummaryStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistSummaryStatus::Ready => {
                match next {
                    AllowlistSummaryStatus::Draining | AllowlistSummaryStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistSummaryStatus::Draining => {
                return next == AllowlistSummaryStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AllowlistSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> AllowlistSummary {
        AllowlistSummary {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            phone: self.phone.clone(),
            cluster: self.cluster.clone(),
            source: self.source.clone(),
            score: self.score,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a allowlist summary in status idle.
pub fn new_allowlist_summary(code: &str, name: &str) -> AllowlistSummary {
    AllowlistSummary {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        phone: "".to_string(),
        cluster: "".to_string(),
        source: "".to_string(),
        score: 0.0,
        namespace: "".to_string(),
        status: AllowlistSummaryStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_allowlist_summary("code-954", "name-727");
        assert_eq!(item.status, AllowlistSummaryStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_allowlist_summary("code-954", "name-727");
        if item.move_to(AllowlistSummaryStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AllowlistSummaryStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_allowlist_summary("code-954", "name-727");
        item.status = AllowlistSummaryStatus::Stopped;
        assert!(
            item.move_to(AllowlistSummaryStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
