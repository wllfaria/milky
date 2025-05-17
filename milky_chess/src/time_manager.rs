use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub struct ConventionalTimeControl {
    time_left: Duration,
    increment: Duration,
    moves_to_go: Option<u32>,
}

#[derive(Debug, Clone, Copy)]
pub enum TimeControl {
    Conventional(ConventionalTimeControl),
    MoveTime(Duration),
    Infinite,
    FixedDepth(u8),
    FixedNodes(u64),
    MateIn(u32),
}

#[derive(Debug)]
pub struct SearchLimits {
    start_time: Instant,
    time_control: TimeControl,
    game_ply: u32,
}

impl SearchLimits {
    pub fn start_time(&self) -> Instant {
        self.start_time
    }

    pub fn time_control(&self) -> TimeControl {
        self.time_control
    }

    pub fn game_ply(&self) -> u32 {
        self.game_ply
    }
}

#[derive(Debug)]
pub struct TimeManager {
    search_limits: SearchLimits,
    stop_time: Option<Instant>,
}

impl TimeManager {
    pub fn new(search_limits: SearchLimits) -> Self {
        let mut time_manager = Self {
            search_limits,
            stop_time: None,
        };

        time_manager.compute_stop_time();

        time_manager
    }

    fn compute_stop_time(&mut self) {
        let start_time = self.search_limits.start_time();

        match &self.search_limits.time_control {
            TimeControl::MoveTime(duration) => self.stop_time = Some(start_time + *duration),
            TimeControl::Conventional(ConventionalTimeControl {
                time_left,
                increment,
                moves_to_go,
            }) => {
                let mut time_per_move = *time_left / moves_to_go.unwrap_or(40);
                time_per_move += *increment * 3 / 4;
                let safety_margin = Duration::from_millis(50);
                let stop_time = start_time + time_per_move - safety_margin;
                self.stop_time = Some(stop_time);
            }
            // no fixed stop_time for the following time controls
            TimeControl::FixedDepth(_) => {}
            TimeControl::FixedNodes(_) => {}
            TimeControl::MateIn(_) => {}
            TimeControl::Infinite => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aa() {
        let tm = TimeManager::new(SearchLimits {
            start_time: Instant::now(),
            game_ply: 26,
            time_control: TimeControl::Conventional(ConventionalTimeControl {
                time_left: Duration::from_millis(180_000),
                increment: Duration::from_millis(2000),
                moves_to_go: None,
            }),
        });

        panic!();
    }
}
