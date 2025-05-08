mod uci;

pub trait Protocol {
    fn start_loop(engine: &mut milky_chess::Milky);
}
