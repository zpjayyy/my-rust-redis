use mini_redis::client;
use mini_redis::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = client::connect("127.0.0.1:6379").await?;

    client.set("hello", "world".into()).await?;

    let result = client.get("hello").await?;

    println!("服务端获取到数据数 = {:?}", result);

    Ok(())
}
