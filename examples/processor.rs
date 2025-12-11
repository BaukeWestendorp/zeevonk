use tokio::io;
use zeevonk::client::{Client, ProcessorContext};

#[tokio::main]
async fn main() -> io::Result<()> {
    pretty_env_logger::init();

    let client = Client::connect("127.0.0.1:7334").await?;
    client.register_processor(processor).await;

    Ok(())
}

fn processor(ProcessorContext { frame, patch, values, .. }: ProcessorContext) {
    use std::f32::consts::TAU;
    use zeevonk::gdcs::Attribute;

    for (j, fixture) in patch.fixtures().iter().enumerate() {
        let t = ((frame + j * 100) % 150) as f32 / 150.0;
        let r = (t * TAU).sin() * 0.5 + 0.5;
        let g = ((t + 1.0 / 3.0) * TAU).sin() * 0.5 + 0.5;
        let b = ((t + 2.0 / 3.0) * TAU).sin() * 0.5 + 0.5;

        for (attr, cf) in fixture.channel_functions() {
            let value = match *attr {
                Attribute::ColorAddR => cf.min().lerp(&cf.max(), r),
                Attribute::ColorAddG => cf.min().lerp(&cf.max(), g),
                Attribute::ColorAddB => cf.min().lerp(&cf.max(), b),
                Attribute::Dimmer => cf.min().lerp(&cf.max(), 0.5),
                _ => continue,
            };
            values.set(fixture.path(), *attr, value);
        }
    }
}
