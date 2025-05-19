use std::time::{Duration, Instant};

use milky_bitboard::Side;

pub trait IntoTimeControl {
    fn into_time_control(self, side_to_move: Side) -> TimeControl;
}

pub struct TimeManagerContext {
    pub depth: u8,
    pub nodes: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct ConventionalTimeControl {
    pub time_left: Duration,
    pub increment: Duration,
    pub moves_to_go: Option<u32>,
}

#[derive(Debug, Clone, Copy)]
pub enum TimeControl {
    Conventional(ConventionalTimeControl),
    MoveTime(Duration),
    Infinite,
    FixedDepth(u8),
    FixedNodes(u64),
    MateIn(u8),
}

#[derive(Debug)]
pub(crate) struct SearchLimits {
    start_time: Instant,
    time_control: TimeControl,
}

impl SearchLimits {
    pub fn new(time_control: TimeControl) -> Self {
        Self {
            time_control,
            start_time: Instant::now(),
        }
    }

    pub fn start_time(&self) -> Instant {
        self.start_time
    }
}

#[derive(Debug)]
pub(crate) struct TimeManager {
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

    pub fn should_stop(&self, ctx: TimeManagerContext) -> bool {
        if let Some(stop_time) = self.stop_time {
            return Instant::now() >= stop_time;
        };

        if let TimeControl::FixedDepth(max_depth) = self.search_limits.time_control {
            return ctx.depth >= max_depth;
        }

        if let TimeControl::FixedNodes(max_nodes) = self.search_limits.time_control {
            return ctx.nodes >= max_nodes;
        }

        if let TimeControl::MateIn(mate_depth) = self.search_limits.time_control {
            return ctx.depth > mate_depth * 2;
        }

        false
    }
}
