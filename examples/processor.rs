use tokio::io;

use zeevonk::fpath;
use zeevonk::prelude::*;

#[tokio::main]
async fn main() -> io::Result<()> {
    pretty_env_logger::init();

    let client = Client::connect("127.0.0.1:7334").await?;
    client.register_processor(processor).await;

    Ok(())
}

fn processor(ProcessorContext { patch, values, .. }: ProcessorContext) {
    #[rustfmt::skip] let spots = [fpath![101], fpath![102], fpath![103], fpath![104]];
    #[rustfmt::skip] let washes = [fpath![201], fpath![202], fpath![203], fpath![204], fpath![205], fpath![206], fpath![207], fpath![208]];
    #[rustfmt::skip] let beams = [fpath![301], fpath![302], fpath![303], fpath![304], fpath![305], fpath![306], fpath![307], fpath![308], fpath![309], fpath![310], fpath![311], fpath![312]];
    #[rustfmt::skip] let blinders = [fpath![401, 1, 1], fpath![402, 1, 1], fpath![403, 1, 1], fpath![404, 1, 1]];
    #[rustfmt::skip] let leds = [fpath![401, 1, 2], fpath![402, 1, 2], fpath![403, 1, 2], fpath![404, 1, 2]];
    #[rustfmt::skip] let _fronts = [fpath![501], fpath![502], fpath![503], fpath![504], fpath![505], fpath![506]];
    #[rustfmt::skip] let _crowds = [fpath![601]];

    for path in spots {
        let spots = patch.fixtures().iter().filter(|f| f.path().contains(path));
        for spot in spots {
            values.set(spot.path(), Attribute::Dimmer, ClampedValue::MAX);
            values.set(spot.path(), Attribute::Color(1), ClampedValue::new(0.2));
        }
    }

    for path in washes {
        let washs = patch.fixtures().iter().filter(|f| f.path().contains(path));
        for wash in washs {
            values.set(wash.path(), Attribute::Dimmer, ClampedValue::MAX);
            values.set(wash.path(), Attribute::ColorAddR, ClampedValue::MIN);
            values.set(wash.path(), Attribute::ColorAddG, ClampedValue::MIN);
            values.set(wash.path(), Attribute::ColorAddB, ClampedValue::MAX);
        }
    }

    for path in beams {
        let beams = patch.fixtures().iter().filter(|f| f.path().contains(path));
        for beam in beams {
            values.set(beam.path(), Attribute::Dimmer, ClampedValue::MAX);
            values.set(beam.path(), Attribute::ColorAddR, ClampedValue::MIN);
            values.set(beam.path(), Attribute::ColorAddG, ClampedValue::MAX);
            values.set(beam.path(), Attribute::ColorAddB, ClampedValue::MAX);
        }
    }

    for path in blinders {
        let blinders = patch.fixtures().iter().filter(|f| f.path().contains(path));
        for blind in blinders {
            values.set(blind.path(), Attribute::Dimmer, ClampedValue::new(0.1));
        }
    }

    for path in leds {
        let leds = patch.fixtures().iter().filter(|f| f.path().contains(path));
        for led in leds {
            values.set(led.path(), Attribute::Dimmer, ClampedValue::MAX);
            values.set(led.path(), Attribute::ColorAddR, ClampedValue::MAX);
            values.set(led.path(), Attribute::ColorAddG, ClampedValue::MIN);
            values.set(led.path(), Attribute::ColorAddB, ClampedValue::MAX);
        }
    }
}
