use crate::app_state::AppState;
use axum::extract::State;
use axum::response::sse::Event;
use axum::response::Sse;
use axum_extra::headers;
use axum_extra::TypedHeader;
use futures::Stream;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

pub(crate) async fn log(
    State(state): State<Arc<AppState>>,
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    println!("`{}` connected", user_agent.as_str());

    let mut receiver = state.log.sender.subscribe();

    let stream = async_stream::stream! {
        loop {
            match receiver.recv().await {
                Ok((event, message)) => {
                    yield Ok(Event::default().event(event).data(message));
                },
                Err(_) => {
                    yield Ok(Event::default().data("Receiver closed"));
                    break;
                }
            }
        }
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}

pub(crate) async fn all(
    State(state): State<Arc<AppState>>,
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    println!("`{}` connected", user_agent.as_str());

    let (mut user_receiver, mut token_receiver, mut sender_receiver) = state.get_receivers().await;

    let stream = async_stream::stream! {
        loop {
            tokio::select! {
                msg = user_receiver.recv() => match msg {
                    Ok((event, message)) => yield Ok(Event::default().event(event).data(message)),
                    Err(_) => {
                        yield Ok(Event::default().data("User receiver closed"));
                        break;
                    },
                },
                msg = token_receiver.recv() => match msg {
                    Ok((event, message)) => yield Ok(Event::default().event(event).data( message)),
                    Err(_) => {
                        yield Ok(Event::default().data("Token receiver closed"));
                        break;
                    },
                },
                msg = sender_receiver.recv() => match msg {
                    Ok((event, message)) => yield Ok(Event::default().event(event).data(message)),
                    Err(_) => {
                        yield Ok(Event::default().data("Sender receiver closed"));
                        break;
                    },
                },
            }
        }
    };

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}
