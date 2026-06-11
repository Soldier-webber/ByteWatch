/// Terminal UI rendering module
/// 
/// This module handles all UI rendering using ratatui.
/// It displays:
/// - System statistics (CPU, memory, disk, network)
/// - Process list with detailed information
/// - Help text and status information

use crate::app::{App, SortBy};
use crate::system::format_bytes;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Gauge, Paragraph, Table, Row, Constraint},
    text::{Line, Span},
};

/// Render the entire UI
pub fn render(f: &mut Frame, app: &App) {
    let size = f.size();

    // Main layout: vertical split into header and content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(1),  // Title
                Constraint::Length(12), // System stats
                Constraint::Min(5),     // Process list
                Constraint::Length(1),  // Footer
            ]
            .as_ref(),
        )
        .split(size);

    // Title
    render_title(f, chunks[0]);

    // System statistics
    render_system_stats(f, chunks[1], app);

    // Process list
    render_process_list(f, chunks[2], app);

    // Footer
    render_footer(f, chunks[3]);
}

/// Render the title bar
fn render_title(f: &mut Frame, area: Rect) {
    let title = Paragraph::new(
        Span::styled(
            "ByteWatch - System Monitor",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
    );
    f.render_widget(title, area);
}

/// Render system statistics section
fn render_system_stats(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(4), // CPU & Memory
                Constraint::Length(4), // Disk & Network
                Constraint::Length(4), // Additional info
            ]
        )
        .split(area);

    // CPU and Memory gauges
    render_cpu_memory(f, chunks[0], app);

    // Disk and Network info
    render_disk_network(f, chunks[1], app);

    // Additional system info
    render_system_info(f, chunks[2], app);
}

/// Render CPU and memory usage gauges
fn render_cpu_memory(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title("CPU & Memory")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .margin(1)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    // CPU gauge
    let cpu_percent = app.cpu_usage();
    let cpu_color = gauge_color(cpu_percent);
    
    let cpu_gauge = Gauge::default()
        .block(Block::default().title("CPU"))
        .gauge_style(Style::default().fg(cpu_color))
        .percent(cpu_percent as u16);

    f.render_widget(cpu_gauge, chunks[0]);

    // Memory gauge
    let mem_percent = app.memory_usage();
    let mem_color = gauge_color(mem_percent);

    let mem_gauge = Gauge::default()
        .block(Block::default().title("Memory"))
        .gauge_style(Style::default().fg(mem_color))
        .percent(mem_percent as u16);

    f.render_widget(mem_gauge, chunks[1]);
}

/// Render disk and network information
fn render_disk_network(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title("Disk & Network")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(inner);

    // Disk info
    let (disk_used, disk_total) = app.disk_usage();
    let disk_percent = if disk_total > 0 {
        (disk_used as f32 / disk_total as f32 * 100.0) as u16
    } else {
        0
    };

    let disk_gauge = Gauge::default()
        .block(Block::default().title("Disk"))
        .gauge_style(Style::default().fg(gauge_color(disk_percent as f32)))
        .label(format!(
            "{} / {}",
            format_bytes(disk_used),
            format_bytes(disk_total)
        ))
        .percent(disk_percent);

    f.render_widget(disk_gauge, chunks[0]);

    // Network info
    let (rx, tx) = app.network_throughput();
    let net_text = format!(
        "↓ {}/s  ↑ {} /s",
        format_bytes(rx),
        format_bytes(tx)
    );

    let network_para = Paragraph::new(net_text)
        .block(Block::default().title("Network"))
        .alignment(Alignment::Center);

    f.render_widget(network_para, chunks[1]);
}

/// Render additional system information
fn render_system_info(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title("System Info")
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let (mem_used, mem_total) = app.memory_info();
    let uptime = crate::system::format_uptime(app.uptime());

    let info_lines = vec![
        Line::from(vec![
            Span::raw("Memory: "),
            Span::styled(
                format!("{} / {}", format_bytes(mem_used), format_bytes(mem_total)),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(vec![
            Span::raw("Uptime: "),
            Span::styled(uptime, Style::default().fg(Color::Green)),
        ]),
        Line::from(vec![
            Span::raw("Processes: "),
            Span::styled(
                app.process_count().to_string(),
                Style::default().fg(Color::Magenta),
            ),
        ]),
    ];

    let info_para = Paragraph::new(info_lines)
        .block(Block::default())
        .left_margin(1)
        .top_margin(1);

    f.render_widget(info_para, inner);
}

/// Render the process list table
fn render_process_list(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(match app.sort_by() {
            SortBy::Cpu => "Processes (sorted by CPU)",
            SortBy::Memory => "Processes (sorted by Memory)",
            SortBy::Name => "Processes (sorted by Name)",
            SortBy::Pid => "Processes (sorted by PID)",
        })
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let header = Row::new(vec!["PID", "Name", "CPU %", "Memory"])
        .style(
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .bottom_margin(1);

    let rows = app
        .processes()
        .iter()
        .take((inner.height.saturating_sub(2)) as usize)
        .map(|proc| {
            let cpu_color = if proc.cpu_usage > 50.0 {
                Color::Red
            } else if proc.cpu_usage > 20.0 {
                Color::Yellow
            } else {
                Color::Green
            };

            Row::new(vec![
                Span::raw(proc.pid.to_string()),
                Span::styled(
                    truncate_string(&proc.name, 30),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    format!("{:.1}%", proc.cpu_usage),
                    Style::default().fg(cpu_color),
                ),
                Span::raw(format_bytes(proc.memory_usage)),
            ])
        });

    let table = Table::new(rows, [
        Constraint::Length(8),
        Constraint::Min(30),
        Constraint::Length(8),
        Constraint::Length(10),
    ])
    .header(header)
    .block(Block::default());

    f.render_widget(table, inner);
}

/// Render the footer with keyboard controls
fn render_footer(f: &mut Frame, area: Rect) {
    let help_text = "q/Esc: Quit  |  r: Refresh";
    let footer = Paragraph::new(
        Span::styled(
            help_text,
            Style::default()
                .fg(Color::DarkGray)
                .italic(),
        )
    )
    .alignment(Alignment::Center);

    f.render_widget(footer, area);
}

/// Determine gauge color based on percentage
fn gauge_color(percent: f32) -> Color {
    match percent {
        p if p >= 80.0 => Color::Red,
        p if p >= 60.0 => Color::Yellow,
        p if p >= 40.0 => Color::Yellow,
        _ => Color::Green,
    }
}

/// Truncate a string to a maximum length
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}…", &s[..max_len - 1])
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gauge_color() {
        assert_eq!(gauge_color(90.0), Color::Red);
        assert_eq!(gauge_color(70.0), Color::Yellow);
        assert_eq!(gauge_color(30.0), Color::Green);
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("hello", 10), "hello");
        assert_eq!(truncate_string("hello world test", 10), "hello wor…");
    }
}
