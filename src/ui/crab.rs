pub const TICK_MS: u64 = 160;

pub const FRAMES: [&str; 2] = [
    r#"   _     _
  ( \\___/ )
  ( o   o )
  (   V   )
 /|\\   /|\\
/_|_\\_/_|_\\
   /   \\
  /_____\\"#,
    r#"   _     _
  (  ___  )
  ( o   o )
  (   V   )
 /|\\   /|\\
/_|_\\_/_|_\\
   /   \\
  /_____\\"#,
];

pub fn frame(frame_index: usize) -> &'static str {
    let idx = frame_index % FRAMES.len();
    FRAMES[idx]
}

pub fn frame_count() -> usize {
    FRAMES.len()
}

pub fn max_width() -> u16 {
    FRAMES
        .iter()
        .flat_map(|f| f.lines())
        .map(|line| line.chars().count() as u16)
        .max()
        .unwrap_or(0)
}
