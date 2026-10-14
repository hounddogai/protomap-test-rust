use super::*;

use acme_kit::text;

/// How screens and exports show a batched sink exception.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSinkExceptionView {
    pub code: String,
    pub title: String,
    pub amount_text: String,
    pub attempts_text: String,
    pub region: String,
    pub floor_value: String,
    pub audience: String,
    pub grace_days: String,
    pub rate: String,
    pub host: String,
    pub maximum: String,
    pub status_label: String,
}

pub fn to_batched_sink_exception_view(item: &BatchedSinkException) -> BatchedSinkExceptionView {
    BatchedSinkExceptionView {
        code: item.code.clone(),
        title: item.title.clone(),
        amount_text: format!("{:.2}", item.amount),
        attempts_text: item.attempts.to_string(),
        region: item.region.clone(),
        floor_value: format!("{:.2}", item.floor_value),
        audience: item.audience.clone(),
        grace_days: item.grace_days.to_string(),
        rate: format!("{:.2}", item.rate),
        host: text::truncate(&item.host, 64),
        maximum: format!("{:.2}", item.maximum),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_batched_sink_exception_views(items: &[BatchedSinkException]) -> Vec<BatchedSinkExceptionView> {
    let mut views: Vec<BatchedSinkExceptionView> = Vec::new();
    for item in items.iter() {
        views.push(to_batched_sink_exception_view(item));
    }
    views
}

pub fn sum_batched_sink_exception_view(view: &BatchedSinkExceptionView) -> Result<f64, String> {
    let parsed_amount = view
        .amount_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse amount: {err}"))?;
    let parsed_attempts = view
        .attempts_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse attempts: {err}"))?;
    let parsed_floor_value = view
        .floor_value
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    Ok(parsed_amount + (parsed_attempts as f64) + parsed_floor_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_batched_sink_exception("code-697", "title-169");
        let view = to_batched_sink_exception_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_batched_sink_exception_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
