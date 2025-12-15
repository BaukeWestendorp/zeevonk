use std::io;

use zeevonk::attr::Attribute;
use zeevonk::client::ProcessorContext;
use zeevonk::fpath;

#[tokio::main]
async fn main() -> io::Result<()> {
    pretty_env_logger::formatted_builder().filter_level(log::LevelFilter::Debug);

    let client = zeevonk::client::Client::connect("127.0.0.1:7334").await?;
    client.register_processor(processor).await;

    Ok(())
}

fn processor(mut cx: ProcessorContext) {
    #[rustfmt::skip] let spots = [fpath![101], fpath![102], fpath![103], fpath![104]];
    #[rustfmt::skip] let washes = [fpath![201], fpath![202], fpath![203], fpath![204], fpath![205], fpath![206], fpath![207], fpath![208]];
    #[rustfmt::skip] let beams = [fpath![301], fpath![302], fpath![303], fpath![304], fpath![305], fpath![306], fpath![307], fpath![308], fpath![309], fpath![310], fpath![311], fpath![312]];
    #[rustfmt::skip] let _blinders = [fpath![401, 1, 1], fpath![402, 1, 1], fpath![403, 1, 1], fpath![404, 1, 1]];
    #[rustfmt::skip] let leds = [fpath![401, 1, 2], fpath![402, 1, 2], fpath![403, 1, 2], fpath![404, 1, 2]];
    #[rustfmt::skip] let fronts = [fpath![501], fpath![502], fpath![503], fpath![504], fpath![505], fpath![506]];
    #[rustfmt::skip] let crowds = [fpath![601]];

    for (ix, fixture) in spots
        .iter()
        .chain(washes.iter())
        .chain(beams.iter())
        .chain(leds.iter())
        .chain(fronts.iter())
        .chain(crowds.iter())
        .enumerate()
    {
        let t = cx.frame() as f32 + ix as f32 * 5.0;
        let value = (t % 100.0) / 100.0;
        cx.set_attribute(*fixture, Attribute::Dimmer, value, true);
    }
    cx.set_attribute(leds, Attribute::ColorAddR, 1.0, true);
    cx.set_attribute(leds, Attribute::ColorAddG, 0.0, true);
    cx.set_attribute(leds, Attribute::ColorAddB, 1.0, true);
}
