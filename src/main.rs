use async_std::task;
use std::time::Duration;
use tide::sse;
use tide::{Response, StatusCode};
use broadcaster::BroadcastChannel;
use futures::stream::StreamExt;

#[derive(Clone)]
struct State {
    chan: broadcaster::BroadcastChannel<u32>,
}

#[async_std::main]
async fn main() -> Result<(), std::io::Error> {
    tide::log::start();
    let chan = BroadcastChannel::new();
    let state = State {
        chan: chan.clone(),
    };
    let mut app = tide::with_state(state);
    app.at("/").get(|_| async {
        let body = r#"<html>
<head>
<script type="text/javascript">
const source = new EventSource("//localhost:8080/sse")
source.addEventListener("number", function(event) {
    document.body.innerHTML = event.data
})
</script>
</head>
</html>"#;
        let mut res = Response::new(StatusCode::Ok);
        res.set_body(body);
        res.insert_header("Content-Type", "text/html");
        Ok(res)
    });
    app.at("/sse").get(sse::endpoint(
        |req: tide::Request<State>, sender| async move {
            let mut chan = req.state().chan.clone();
            while let Some(event) = chan.next().await {
                sender
                    .send("number", event.to_string(), Some(&event.to_string()))
                    .await?;
            }
            Ok(())
        },
    ));
    task::spawn(async move {
        for i in 0u32.. {
            task::sleep(Duration::from_secs(1)).await;
            chan.send(&i).await.unwrap()
        }
    });
    app.listen("localhost:8080").await?;
    Ok(())
}
