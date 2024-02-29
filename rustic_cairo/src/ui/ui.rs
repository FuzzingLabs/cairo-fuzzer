use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
#[cfg(target_os = "linux")]
use memory_stats::memory_stats;

use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;
use ratatui::{prelude::*, widgets::*};
use std::collections::VecDeque;
use std::io;
use std::io::Stdout;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use crate::ui::utils::create_event_item;
use crate::{fuzzer::error::Error, mutator::types::Type};
use crate::{fuzzer::stats::Stats, mutator::types::Parameters};

#[derive(Debug, Clone)]
pub struct UiEventData {
    pub time: time::Duration,
    pub message: String,
    pub error: Option<Error>,
}

#[derive(Debug, Clone)]
pub enum UiEvent {
    NewCoverage(UiEventData),
    NewCrash(UiEventData),
    DetectorTriggered(UiEventData),
}

// Data to be displayed on the tui
pub struct Ui {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    // Infos (for new coverage, coverages...)
    nb_threads: u8,
    // Idx of displayed thread static
    threads_stats_idx: usize,
    // Seed just to be displayed
    seed: u64,
    // Coverage history for graph
    coverages: Vec<(f64, f64)>,
    // Crashes history for graph
    crashes: Vec<(f64, f64)>,
    // Index for graph tabs
    tab_index: usize,
    // Target infos
    target_module: String,
    target_function: String,
    target_parameters: Vec<Type>,
    max_coverage: usize,
}

impl Ui {
    pub fn new(nb_threads: u8, seed: u64) -> Self {
        let terminal = Self::setup_terminal();

        Ui {
            terminal,
            nb_threads,
            seed,
            threads_stats_idx: 0,
            coverages: vec![],
            crashes: vec![],
            tab_index: 0,
            target_module: String::from(""),
            target_function: String::from(""),
            target_parameters: vec![],
            max_coverage: 0,
        }
    }

    fn setup_terminal() -> Terminal<CrosstermBackend<Stdout>> {
        let mut stdout = io::stdout();
        enable_raw_mode().expect("failed to enable raw mode");
        execute!(stdout, EnterAlternateScreen).expect("unable to enter alternate screen");
        Terminal::new(CrosstermBackend::new(stdout)).expect("creating terminal failed")
    }

    pub fn restore_terminal(&mut self) {
        disable_raw_mode().unwrap();
        execute!(self.terminal.backend_mut(), LeaveAlternateScreen).unwrap();
        self.terminal.show_cursor().unwrap();
    }

    pub fn set_target_infos(
        &mut self,
        target_module: &str,
        target_function: &str,
        target_parameters: &Vec<Type>,
        max_coverage: usize,
    ) {
        self.target_module = target_module.to_string();
        self.target_function = target_function.to_string();
        self.target_parameters = target_parameters.clone();
        self.max_coverage = max_coverage;
    }

    pub fn render(
        &mut self,
        stats: &Stats,
        events: &VecDeque<UiEvent>,
        threads_stats: &Vec<Arc<RwLock<Stats>>>,
    ) -> bool {
        self.terminal
            .draw(|frame| {
                let chunks = Layout::default()
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .margin(1)
                    .direction(Direction::Vertical)
                    .split(frame.size());

                // Draws main block
                let main_block = Block::default().borders(Borders::ALL).title(format!(
                    "Sui Fuzzer, {} threads (q to quit)",
                    self.nb_threads
                ));
                frame.render_widget(main_block, frame.size());

                // Stats block
                let stats_block = Block::default().borders(Borders::ALL).title("Stats");
                Self::draw_stats_block(
                    frame,
                    chunks[0],
                    stats,
                    self.seed,
                    self.max_coverage,
                    self.threads_stats_idx,
                    threads_stats,
                    &mut self.coverages,
                    &mut self.crashes,
                    self.tab_index,
                    &self.target_module,
                    &self.target_function,
                    &self.target_parameters,
                );
                frame.render_widget(stats_block, chunks[0]);

                // Events block
                let events_block = Block::default().borders(Borders::ALL).title(Span::styled(
                    "Events",
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ));
                Self::draw_events_block(frame, chunks[1], stats, events);
                frame.render_widget(events_block, chunks[1]);
            })
            .unwrap();

        if event::poll(Duration::from_millis(250)).unwrap() {
            if let Event::Key(key) = event::read().unwrap() {
                // q to quit the fuzzer
                if KeyCode::Char('q') == key.code {
                    return true;
                }
                // Inputs for worker stats
                if KeyCode::Char('l') == key.code {
                    self.threads_stats_idx = if self.threads_stats_idx >= 1 {
                        (self.threads_stats_idx - 1).into()
                    } else {
                        (self.nb_threads - 1).into()
                    }
                }
                if KeyCode::Char('r') == key.code {
                    self.threads_stats_idx =
                        if (self.threads_stats_idx + 1) < self.nb_threads as usize {
                            (self.threads_stats_idx + 1).into()
                        } else {
                            0
                        }
                }
                // Inputs for graphs
                if KeyCode::Left == key.code {
                    self.tab_index = if self.tab_index == 0 { 1 } else { 0 }
                }
                if KeyCode::Right == key.code {
                    self.tab_index = if self.tab_index == 1 { 0 } else { 1 }
                }
            }
        }
        return false;
    }

    fn draw_stats_block<B>(
        frame: &mut Frame<B>,
        area: Rect,
        stats: &Stats,
        seed: u64,
        max_coverage: usize,
        threads_stats_idx: usize,
        threads_stats: &Vec<Arc<RwLock<Stats>>>,
        coverages: &mut Vec<(f64, f64)>,
        crashes: &mut Vec<(f64, f64)>,
        index: usize,
        target_module: &str,
        target_function: &str,
        target_parameters: &Vec<Type>,
    ) where
        B: Backend,
    {
        let chunks = Layout::default()
            .constraints(
                [
                    Constraint::Percentage(25),
                    Constraint::Percentage(20),
                    Constraint::Percentage(55),
                ]
                .as_ref(),
            )
            .margin(1)
            .direction(Direction::Horizontal)
            .split(area);

        let duration = time::Duration::seconds(stats.secs_since_last_cov.try_into().unwrap());
        let running_duration = time::Duration::seconds(stats.time_running.try_into().unwrap());

        let mut text = vec![
            text::Line::from(format!("Seed: {}", seed)),
            text::Line::from(format!("Crashes: {}", stats.crashes)),
            text::Line::from(format!("Unique crashes: {}", stats.unique_crashes)),
            text::Line::from(format!("Total execs: {}", stats.execs)),
            text::Line::from(format!("Execs/s: {}", stats.execs_per_sec)),
            text::Line::from(format!(
                "Coverage: {}/{}",
                stats.coverage_size, max_coverage
            )),
            text::Line::from(format!(
                "Running for: {}d {}h {}m {}s",
                running_duration.whole_days(),
                running_duration.whole_hours(),
                running_duration.whole_minutes(),
                running_duration.whole_seconds()
            )),
            text::Line::from(format!(
                "Last coverage update: {}d {}h {}m {}s",
                duration.whole_days(),
                duration.whole_hours(),
                duration.whole_minutes(),
                duration.whole_seconds()
            )),
        ];
        // The crate for the memory doesn't work on mac
        if cfg!(target_os = "linux") {
            // Gets memory usage
            let mut mem = 0;
            if let Some(usage) = memory_stats() {
                mem = usage.virtual_mem;
            }
            text.push(text::Line::from(format!(
                "Memory usage: {} MB",
                mem / 1000000
            )));
        }

        let infos_chunk = Layout::default()
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .margin(1)
            .direction(Direction::Vertical)
            .split(chunks[0]);

        let global_stats_block = Block::default().borders(Borders::ALL).title(Span::styled(
            "Fuzzing statistics:",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        ));
        let paragraph = Paragraph::new(text)
            .block(global_stats_block)
            .wrap(Wrap { trim: false });
        frame.render_widget(paragraph, infos_chunk[0]);

        let text = vec![
            text::Line::from(vec![Span::styled("Target: ", Style::new().green())]),
            text::Line::from(format!("{}::{}", target_module, target_function)),
            text::Line::from(vec![Span::styled("Parameters: ", Style::new().green())]),
            text::Line::from(format!("{}", Parameters(target_parameters.to_vec()))),
        ];

        let global_stats_block = Block::default().borders(Borders::ALL).title(Span::styled(
            "Target info:",
            Style::default()
                .fg(Color::LightBlue)
                .add_modifier(Modifier::BOLD),
        ));
        let paragraph = Paragraph::new(text)
            .block(global_stats_block)
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph, infos_chunk[1]);

        let worker_stats_block = Block::default().borders(Borders::ALL).title(Span::styled(
            format!("Worker {} stats: (l/r to switch)", threads_stats_idx),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ));
        Self::draw_thread_stats_block(frame, chunks[1], &threads_stats[threads_stats_idx]);
        frame.render_widget(worker_stats_block, chunks[1]);

        let graph_block = Block::default().borders(Borders::ALL).title(Span::styled(
            "Graphs (arrow key to switch)",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ));
        Self::draw_graph_block(frame, chunks[2], stats, coverages, crashes, index);
        frame.render_widget(graph_block, chunks[2]);
    }

    fn draw_thread_stats_block<B>(frame: &mut Frame<B>, area: Rect, stats: &Arc<RwLock<Stats>>)
    where
        B: Backend,
    {
        let chunks = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .margin(1)
            .direction(Direction::Horizontal)
            .split(area);

        let text = vec![
            text::Line::from(format!("Crashes: {}", stats.read().unwrap().crashes)),
            text::Line::from(format!("Total execs: {}", stats.read().unwrap().execs)),
            text::Line::from(format!("Execs/s: {}", stats.read().unwrap().execs_per_sec)),
        ];
        let global_stats_block = Block::default();
        let paragraph = Paragraph::new(text)
            .block(global_stats_block)
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph, chunks[0]);
    }

    fn draw_events_block<B>(
        frame: &mut Frame<B>,
        area: Rect,
        _stats: &Stats,
        events: &VecDeque<UiEvent>,
    ) where
        B: Backend,
    {
        let chunks = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .margin(1)
            .direction(Direction::Horizontal)
            .split(area);

        // Generates listitems for events

        let events: Vec<ListItem> = events
            .iter()
            .map(|event| match event {
                UiEvent::NewCoverage(data) => {
                    let style = Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD);
                    let event_type = "COVERAGE".to_string();
                    create_event_item(
                        data.time,
                        style,
                        event_type,
                        format!(" with input: {}", data.message),
                    )
                }
                UiEvent::NewCrash(data) => {
                    let style = Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);
                    let error = data.error.clone().unwrap();
                    let event_type = format!("CRASH Type: {}", error).to_string();
                    create_event_item(
                        data.time,
                        style,
                        event_type,
                        format!(" with input: {}", data.message),
                    )
                }
                UiEvent::DetectorTriggered(data) => {
                    let style = Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD);
                    let event_type = format!("Detector triggered: ").to_string();
                    create_event_item(data.time, style, event_type, data.message.clone())
                }
            })
            .collect();
        let events = List::new(events).start_corner(Corner::BottomLeft);
        frame.render_widget(events, chunks[0]);
    }

    fn draw_graph_block<B>(
        frame: &mut Frame<B>,
        area: Rect,
        stats: &Stats,
        coverages: &mut Vec<(f64, f64)>,
        crashes: &mut Vec<(f64, f64)>,
        index: usize,
    ) where
        B: Backend,
    {
        // Avoid dividing by zero
        if stats.time_running == 0 {
            return;
        }
        let chunks = Layout::default()
            .constraints([Constraint::Percentage(100)].as_ref())
            .margin(1)
            .direction(Direction::Horizontal)
            .split(area);

        let chunks = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(chunks[0]);
        let titles = vec!["Coverage", "Crashes"]
            .iter()
            .map(|t| text::Line::from(Span::styled(*t, Style::default().fg(Color::Green))))
            .collect();
        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(Style::default().fg(Color::Blue))
            .select(index);
        frame.render_widget(tabs, chunks[0]);

        // Adds new stats to execs_speeds vector
        coverages.push((stats.time_running as f64, stats.coverage_size as f64));
        crashes.push((
            stats.time_running as f64,
            (stats.crashes / stats.time_running) as f64,
        ));

        if index == 0 {
            Self::draw_graph(
                frame,
                chunks[1],
                "Coverage",
                Color::Yellow,
                stats,
                coverages,
            );
        } else {
            Self::draw_graph(frame, chunks[1], "Crashes", Color::Red, stats, crashes);
        }
    }

    fn draw_graph<B>(
        frame: &mut Frame<B>,
        area: Rect,
        title: &str,
        color: Color,
        stats: &Stats,
        data: &mut Vec<(f64, f64)>,
    ) where
        B: Backend,
    {
        let datasets = vec![Dataset::default()
            .name(title)
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(color))
            .data(&data)];

        // Finds min and max for dynamic graph
        let min = data
            .iter()
            .fold(data[0].1, |min, &x| if x.1 < min { x.1 } else { min });
        let max = data
            .iter()
            .fold(data[0].1, |max, &x| if x.1 > max { x.1 } else { max });

        // Bindings for graph labels
        let binding1 = (max as u64).to_string();
        let binding_max = binding1.bold();
        let binding2 = ((max / 2.0) as u64).to_string();
        let binding_mid = binding2.bold();
        let binding3 = (min as u64).to_string();
        let binding_min = binding3.bold();
        let chart = Chart::new(datasets)
            .x_axis(
                Axis::default()
                    .title("Time")
                    .style(Style::default().fg(Color::Gray))
                    .bounds([0.0, stats.time_running as f64]),
            )
            .y_axis(
                Axis::default()
                    .style(Style::default().fg(Color::Gray))
                    .labels(vec![binding_min, binding_mid, binding_max])
                    .bounds([min, max]),
            );
        frame.render_widget(chart, area);
    }
}
