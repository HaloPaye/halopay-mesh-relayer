use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;
use tokio::sync::mpsc;

pub struct TuiApp {
    pub logs: Vec<String>,
    pub active_peers: Vec<String>,
    pub mempool_size: usize,
}

pub async fn run_tui(mut rx: mpsc::Receiver<String>) -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = TuiApp {
        logs: Vec::new(),
        active_peers: Vec::new(),
        mempool_size: 0,
    };

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(20),
                        Constraint::Percentage(60),
                        Constraint::Percentage(20),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            let peers: Vec<ListItem> = app
                .active_peers
                .iter()
                .map(|p| ListItem::new(p.as_str()))
                .collect();
            let peers_list = List::new(peers).block(Block::default().borders(Borders::ALL).title("Active Peers"));
            f.render_widget(peers_list, chunks[0]);

            let logs: Vec<ListItem> = app
                .logs
                .iter()
                .map(|l| ListItem::new(l.as_str()))
                .collect();
            let logs_list = List::new(logs).block(Block::default().borders(Borders::ALL).title("Gossip & Settlement Logs"));
            f.render_widget(logs_list, chunks[1]);

            let mempool = Paragraph::new(format!("Mempool Size: {}", app.mempool_size))
                .block(Block::default().borders(Borders::ALL).title("SQLite Mempool"));
            f.render_widget(mempool, chunks[2]);
        })?;

        if crossterm::event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if let KeyCode::Char('q') = key.code {
                    break;
                }
            }
        }

        while let Ok(msg) = rx.try_recv() {
            if msg.starts_with("PEER:") {
                let p = msg.replace("PEER:", "");
                if !app.active_peers.contains(&p) {
                    app.active_peers.push(p);
                }
            } else if msg.starts_with("MEMPOOL:") {
                if let Ok(size) = msg.replace("MEMPOOL:", "").parse::<usize>() {
                    app.mempool_size = size;
                }
            } else {
                app.logs.push(msg);
                if app.logs.len() > 100 {
                    app.logs.remove(0);
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
