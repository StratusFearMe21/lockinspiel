pub enum TimerState {
    Paused(jiff::SignedDuration),
    Going(jiff::Timestamp),
}

pub struct Timer {}
