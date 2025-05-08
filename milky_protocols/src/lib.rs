use uci::error::Result;

pub mod uci;

pub trait Protocol {
    fn start_loop(&mut self, engine: &mut milky_chess::Milky) -> Result<()>;
}
