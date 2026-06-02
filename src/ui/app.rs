use std::io;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, TryRecvError};

use crate::ports::FileSystem;
use crate::summarizer::SummaryReport;

#[derive(Debug, Clone)]
pub struct TopicItem {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug)]
pub enum View {
    Selector,
    Processing { topic: TopicItem },
    Summary { topic: TopicItem, report: SummaryReport },
    Error { message: String },
    Empty { message: String },
}

#[derive(Debug)]
pub enum WorkerResult {
    Ok(SummaryReport),
    Err(String),
}

#[derive(Debug)]
pub struct App {
    pub docs_dir: PathBuf,
    pub topics: Vec<TopicItem>,
    pub selected: usize,
    pub view: View,
    pub crab_frame: usize,
    pub top_n: usize,
    pub worker_rx: Option<Receiver<WorkerResult>>,
    pub exit_requested: bool,
}

impl App {
    pub fn new(docs_dir: PathBuf, mut topics: Vec<TopicItem>, top_n: usize) -> Self {
        topics.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        Self {
            docs_dir,
            topics,
            selected: 0,
            view: View::Selector,
            crab_frame: 0,
            top_n,
            worker_rx: None,
            exit_requested: false,
        }
    }

    pub fn request_exit(&mut self) {
        self.exit_requested = true;
    }

    pub fn menu_len(&self) -> usize {
        self.topics.len().saturating_add(1) // + "Salir"
    }

    pub fn is_exit_selected(&self) -> bool {
        self.selected >= self.topics.len()
    }

    pub fn selected_topic(&self) -> Option<&TopicItem> {
        if self.is_exit_selected() {
            None
        } else {
            self.topics.get(self.selected)
        }
    }

    pub fn select_next(&mut self) {
        let len = self.menu_len();
        if len == 0 {
            return;
        }
        self.selected = (self.selected + 1) % len;
    }

    pub fn select_prev(&mut self) {
        let len = self.menu_len();
        if len == 0 {
            return;
        }

        if self.selected == 0 {
            self.selected = len - 1;
        } else {
            self.selected -= 1;
        }
    }

    pub fn on_tick(&mut self, frame_count: usize) {
        if frame_count == 0 {
            return;
        }
        self.crab_frame = (self.crab_frame + 1) % frame_count;
    }

    pub fn begin_processing(&mut self, topic: TopicItem, rx: Receiver<WorkerResult>) {
        self.view = View::Processing { topic };
        self.worker_rx = Some(rx);
    }

    pub fn poll_worker(&mut self) {
        if !matches!(self.view, View::Processing { .. }) {
            return;
        }

        let Some(rx) = self.worker_rx.as_ref() else {
            return;
        };

        match rx.try_recv() {
            Ok(result) => {
                self.worker_rx = None;

                match result {
                    WorkerResult::Ok(report) => {
                        if let View::Processing { topic } = &self.view {
                            let topic = topic.clone();
                            self.view = View::Summary { topic, report };
                        }
                    }
                    WorkerResult::Err(message) => {
                        self.view = View::Error { message };
                    }
                }
            }
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                self.worker_rx = None;
                self.view = View::Error {
                    message: "El proceso de resumen se desconectó.".to_string(),
                };
            }
        }
    }

    pub fn back_to_selector(&mut self) {
        if self.topics.is_empty() {
            self.view = View::Empty {
                message: format!(
                    "No se encontraron subcarpetas (tópicos).\n\nCreá una carpeta por tópico dentro de:\n{}",
                    self.docs_dir.display()
                ),
            };
        } else {
            self.view = View::Selector;
        }

        self.worker_rx = None;
    }
}

pub fn discover_topics<I: FileSystem>(fs: &I, docs_dir: &Path) -> Result<Vec<TopicItem>, io::Error> {
    let dirs = fs.list_dirs(docs_dir)?;

    let topics = dirs
        .into_iter()
        .map(|path| {
            let name = path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| path.to_string_lossy().to_string());

            TopicItem { name, path }
        })
        .collect::<Vec<_>>();

    Ok(topics)
}
