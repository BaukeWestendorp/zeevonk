use std::thread;
use std::time::Duration;

use zv_core::attr::{Attribute, CustomName};
use zv_core::fixture::{FixtureId, FixtureIdPart};
use zv_core::value::AttributeValues;
use zv_server::Server;

fn main() {
    pretty_env_logger::init();

    let server = Server::new();
    server.start();

    let mut values = AttributeValues::new();

    let fid = FixtureId::from(
        [FixtureIdPart::new(1).unwrap(), FixtureIdPart::new(1).unwrap()].as_slice(),
    );

    let mut i = 0;
    loop {
        let (r, g, b) = rainbow(i);
        values.set(fid, Attribute::Dimmer, 1.0);
        values.set(fid, Attribute::Ctc, 0.0);
        values.set(fid, Attribute::Tint, 0.5);
        values.set(fid, Attribute::Custom(CustomName::new("Color XF".to_string())), 1.0);
        values.set(fid, Attribute::ColorAddR, r as f32 / 255.0);
        values.set(fid, Attribute::ColorAddG, g as f32 / 255.0);
        values.set(fid, Attribute::ColorAddB, b as f32 / 255.0);

        server.test_send(values.clone());

        thread::sleep(Duration::from_secs_f32(1.0 / 60.0));

        i += 1;
    }
}

fn rainbow(i: i32) -> (u8, u8, u8) {
    let hue = (i as f32 * (1.0 / 60.0 / 10.0)) % 1.0;
    let s = 1.0;
    let v = 1.0;

    let h = hue * 6.0;
    let c = v * s;
    let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
    let (r1, g1, b1) = match h as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = v - c;
    let r = ((r1 + m) * 255.0).round() as u8;
    let g = ((g1 + m) * 255.0).round() as u8;
    let b = ((b1 + m) * 255.0).round() as u8;
    (r, g, b)
}
