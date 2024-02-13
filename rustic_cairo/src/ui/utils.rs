use ratatui::{prelude::*, widgets::*};
use time::Duration;

/// Returns a ListItem with given data (used to avoid code duplicates)
pub fn create_event_item(
    time: Duration,
    style: Style,
    event_type: String,
    message: String,
) -> ListItem<'static> {
    ListItem::new(vec![Line::from(vec![
        Span::from(format!(
            "{:0>2}d {:0>2}h {:0>2}m {:0>2}s",
            time.whole_days(),
            time.whole_hours(),
            time.whole_minutes(),
            time.whole_seconds(),
        )),
        ": ".into(),
        Span::styled(event_type.clone(), style),
        message.clone().into(),
    ])])
}
