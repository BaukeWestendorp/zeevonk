use tokio::io;
use zeevonk::client::ZeevonkClient;
use zeevonk::gdcs::Attribute;

#[tokio::main]
async fn main() -> io::Result<()> {
    pretty_env_logger::init();

    ZeevonkClient::connect("127.0.0.1:7334")
        .await?
        .register_processor(|patch, values| {
            for fixture in patch.fixtures() {
                let dimmer_channel_functions =
                    fixture.channel_functions().filter(|(attr, _cf)| **attr == Attribute::Dimmer);

                for (attr, cf) in dimmer_channel_functions {
                    values.set(fixture.path(), *attr, cf.to());
                }
            }
        })
        .await;

    Ok(())
}
