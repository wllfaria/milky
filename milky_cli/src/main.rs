use milky_protocols::Protocol;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut engine = milky_chess::Milky::new();
    let mut uci = milky_protocols::uci::Uci;

    uci.start_loop(&mut engine)?;

    Ok(())
}
