use ratatui::widgets::Widget as RatatuiWidget;

#[allow(dead_code)]
pub enum WidgetData {}

pub trait Widget: RatatuiWidget {
    fn update(data: Vec<WidgetData>);
}
