mod app;
mod crab;
mod render;

pub use self::mod_impl::run;

mod mod_impl {
	use std::error::Error;
	use std::io;
	use std::path::PathBuf;
	use std::sync::mpsc;
	use std::thread;
	use std::time::{Duration, Instant};

	use crossterm::event::{self, Event, KeyCode};
	use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
	use crossterm::execute;

	use ratatui::backend::CrosstermBackend;
	use ratatui::Terminal;

	use crate::ports::{FileSystem, SentenceRanker, SentenceTokenizer};
	use crate::summarizer::Summarizer;

	use super::app::{discover_topics, App, View, WorkerResult};
	use super::{crab, render};

	pub fn run<I, T, R>(
		docs_dir: PathBuf,
		fs: I,
		summarizer: Summarizer<I, T, R>,
		top_n: usize,
	) -> Result<(), Box<dyn Error>>
	where
		I: FileSystem + Clone + Send + Sync + 'static,
		T: SentenceTokenizer + Clone + Send + Sync + 'static,
		R: SentenceRanker + Clone + Send + Sync + 'static,
	{
		let topics = discover_topics(&fs, &docs_dir)?;
		let mut app = App::new(docs_dir, topics, top_n);

		if app.topics.is_empty() {
			app.view = View::Empty {
				message: format!(
					"No se encontraron subcarpetas (tópicos).\n\nCreá una carpeta por tópico dentro de:\n{}",
					app.docs_dir.display()
				),
			};
		}

		let mut tui = Tui::new()?;
		let tick_rate = Duration::from_millis(crab::TICK_MS);
		let mut last_tick = Instant::now();

		loop {
			tui.terminal.draw(|f| render::draw(f, &app))?;

			app.poll_worker();

			let timeout = tick_rate.saturating_sub(last_tick.elapsed());
			if event::poll(timeout)? {
				if let Event::Key(key) = event::read()? {
					match key.code {
						KeyCode::Char('q') | KeyCode::Char('Q') => break,
						_ => handle_key_event(&mut app, &summarizer, key.code),
					}
				}
			}

			if last_tick.elapsed() >= tick_rate {
				app.on_tick(crab::frame_count());
				last_tick = Instant::now();
			}
		}

		Ok(())
	}

	fn handle_key_event<I, T, R>(
		app: &mut App,
		summarizer: &Summarizer<I, T, R>,
		code: KeyCode,
	) where
		I: FileSystem + Clone + Send + Sync + 'static,
		T: SentenceTokenizer + Clone + Send + Sync + 'static,
		R: SentenceRanker + Clone + Send + Sync + 'static,
	{
		match &app.view {
			View::Selector => match code {
				KeyCode::Up => app.select_prev(),
				KeyCode::Down => app.select_next(),
				KeyCode::Enter => {
					if app.is_exit_selected() {
						app.request_exit();
						return;
					}

					let Some(topic) = app.selected_topic().cloned() else {
						return;
					};

					let (tx, rx) = mpsc::channel::<WorkerResult>();
					let summarizer = summarizer.clone();
					let top_n = app.top_n;

					thread::spawn(move || {
						let result = match summarizer.summarize_dir(&topic.path, top_n) {
							Ok(report) => WorkerResult::Ok(report),
							Err(e) => WorkerResult::Err(e.to_string()),
						};
						let _ = tx.send(result);
					});

					app.begin_processing(topic, rx);
				}
				_ => {}
			},
			View::Processing { .. } => {
				// Solo se puede salir con q
			}
			View::Summary { .. } | View::Error { .. } | View::Empty { .. } => match code {
				KeyCode::Esc | KeyCode::Char('b') | KeyCode::Char('B') => app.back_to_selector(),
				KeyCode::Enter => {
					if matches!(app.view, View::Empty { .. }) {
						app.request_exit();
					}
				}
				_ => {}
			},
		}
	}

	struct Tui {
		terminal: Terminal<CrosstermBackend<io::Stdout>>,
	}

	impl Tui {
		fn new() -> Result<Self, Box<dyn Error>> {
			enable_raw_mode()?;
			let mut stdout = io::stdout();
			execute!(stdout, EnterAlternateScreen)?;
			let backend = CrosstermBackend::new(stdout);
			let mut terminal = Terminal::new(backend)?;
			terminal.clear()?;
			Ok(Self { terminal })
		}
	}

	impl Drop for Tui {
		fn drop(&mut self) {
			let _ = disable_raw_mode();
			let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
			let _ = self.terminal.show_cursor();
		}
	}
}
