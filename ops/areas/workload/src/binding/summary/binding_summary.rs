use super::*;

use acme_kit::text;

/// Where a binding summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BindingSummaryStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BindingSummaryStatus {
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

impl std::fmt::Display for BindingSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One binding summary, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BindingSummary {
    pub code: String,
    pub label: String,
    pub weight: f64,
    pub active: bool,
    pub version: i64,
    pub starts_at: i64,
    pub amount: f64,
    pub ceiling: f64,
    pub ratio: f64,
    pub margin: f64,
    pub country: String,
    pub status: BindingSummaryStatus,
}

impl BindingSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BindingSummaryStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BindingSummaryStatus) -> bool {
        match self.status {
            BindingSummaryStatus::Idle => {
                match next {
                    BindingSummaryStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingSummaryStatus::Warming => {
                match next {
                    BindingSummaryStatus::Draining | BindingSummaryStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingSummaryStatus::Ready => {
                match next {
                    BindingSummaryStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BindingSummaryStatus::Draining => {
                match next {
                    BindingSummaryStatus::Stopped => {
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

    pub fn move_to(&mut self, next: BindingSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "binding summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "binding summary", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.margin - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BindingSummary {
        BindingSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            weight: self.weight,
            active: self.active,
            version: self.version,
            starts_at: self.starts_at,
            amount: self.amount,
            ceiling: self.ceiling,
            ratio: self.ratio,
            margin: self.margin,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a binding summary in status idle.
pub fn new_binding_summary(code: &str, label: &str) -> BindingSummary {
    BindingSummary {
        code: code.to_string(),
        label: label.to_string(),
        weight: 0.0,
        active: false,
        version: 0,
        starts_at: 0,
        amount: 0.0,
        ceiling: 0.0,
        ratio: 0.0,
        margin: 0.0,
        country: "".to_string(),
        status: BindingSummaryStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_binding_summary("code-857", "label-744");
        assert_eq!(item.status, BindingSummaryStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_binding_summary("code-857", "label-744");
        if item.move_to(BindingSummaryStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BindingSummaryStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_binding_summary("code-857", "label-744");
        item.status = BindingSummaryStatus::Stopped;
        assert!(
            item.move_to(BindingSummaryStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
