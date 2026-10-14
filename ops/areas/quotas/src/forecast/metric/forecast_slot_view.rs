use super::*;

/// How screens and exports show a forecast slot.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastSlotView {
    pub code_text: String,
    pub title: String,
    pub country_text: String,
    pub floor_value_text: String,
    pub budget: String,
    pub sequence_text: String,
    pub region: String,
    pub ratio: String,
    pub channel: String,
    pub contact_email_text: String,
    pub status_label: String,
}

pub fn to_forecast_slot_view(item: &ForecastSlot) -> ForecastSlotView {
    ForecastSlotView {
        code_text: item.code.clone(),
        title: item.title.clone(),
        country_text: item.country.clone(),
        floor_value_text: format!("{:.2}", item.floor_value),
        budget: format!("{:.2}", item.budget),
        sequence_text: item.sequence.to_string(),
        region: item.region.clone(),
        ratio: format!("{:.2}", item.ratio),
        channel: item.channel.clone(),
        contact_email_text: item.contact_email.clone(),
        status_label: format!("{}", item.status).to_uppercase(),
    }
}

pub fn to_forecast_slot_views(items: &[ForecastSlot]) -> Vec<ForecastSlotView> {
    let mut views: Vec<ForecastSlotView> = Vec::new();
    for item in items.iter() {
        views.push(to_forecast_slot_view(item));
    }
    views
}

pub fn sum_forecast_slot_view(view: &ForecastSlotView) -> Result<f64, String> {
    let parsed_floor_value = view
        .floor_value_text
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse floor value: {err}"))?;
    let parsed_budget = view
        .budget
        .trim()
        .parse::<f64>()
        .map_err(|err| format!("parse budget: {err}"))?;
    let parsed_sequence = view
        .sequence_text
        .trim()
        .parse::<i64>()
        .map_err(|err| format!("parse sequence: {err}"))?;
    Ok(parsed_floor_value + parsed_budget + (parsed_sequence as f64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shows_the_status_in_capitals() {
        let item = new_forecast_slot("code-79", "title-537");
        let view = to_forecast_slot_view(&item);
        assert_eq!(
            view.status_label,
            format!("{}", item.status).to_uppercase(),
            "status label"
        );
        assert_eq!(
            to_forecast_slot_views(&vec![item.clone(), item.clone()]).len() as i64,
            2,
            "views"
        );
    }
}
