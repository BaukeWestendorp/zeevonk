use tokio::io;
use zeevonk::client::Client;

#[tokio::main]
async fn main() -> io::Result<()> {
    pretty_env_logger::init();

    Client::connect("127.0.0.1:7334")
        .await?
        .register_processor(|i, patch, values| {
            use zeevonk::gdcs::Attribute::{ColorAddB, ColorAddG, ColorAddR, Dimmer};

            for (j, fixture) in patch.fixtures().iter().enumerate() {
                let t = ((i + j * 100) % 150) as f32 / 150.0;
                let r = (t * std::f32::consts::TAU).sin() * 0.5 + 0.5;
                let g = ((t + 1.0 / 3.0) * std::f32::consts::TAU).sin() * 0.5 + 0.5;
                let b = ((t + 2.0 / 3.0) * std::f32::consts::TAU).sin() * 0.5 + 0.5;

                for (attr, cf) in fixture.channel_functions() {
                    let value = match *attr {
                        ColorAddR => cf.min().lerp(&cf.max(), r),
                        ColorAddG => cf.min().lerp(&cf.max(), g),
                        ColorAddB => cf.min().lerp(&cf.max(), b),
                        Dimmer => cf.min().lerp(&cf.max(), 0.5),
                        _ => continue,
                    };
                    values.set(fixture.path(), *attr, value);
                }
            }
        })
        .await;

    Ok(())
}
