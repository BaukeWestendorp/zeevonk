use tokio::io;
use zeevonk::client::ZeevonkClient;
use zeevonk::gdcs::Attribute;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut client = ZeevonkClient::connect("127.0.0.1:7334").await?;

    let patch = client.request_patch().await?;

    let dmx_output = client.request_dmx_output().await?;
    println!("before: {dmx_output:?}");

    let mut values = Vec::new();
    for fixture in patch.fixtures() {
        let dimmer_channel_functions = fixture
            .channel_functions()
            .into_iter()
            .filter(|(attr, _cf)| **attr == Attribute::Dimmer);

        for (attr, cf) in dimmer_channel_functions {
            values.push((fixture.path(), attr.clone(), cf.to()));
        }
    }

    client.set_attribute_values(values).await?;

    let dmx_output = client.request_dmx_output().await?;
    println!("after: {dmx_output:?}");

    Ok(())
}
