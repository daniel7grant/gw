use super::{Trigger, TriggerError};
use crate::context::Context;
use log::{debug, info};
use std::{collections::HashMap, sync::mpsc::Sender};
use thiserror::Error;
use tiny_http::{Response, Server};

const TRIGGER_NAME: &str = "HTTP";

/// A trigger that runs on an HTTP request.
///
/// This could be used to trigger checks from git remotes (e.g. GitHub, GitLab) with webhooks.
/// Given that your server can be reached from the outside, you can pass your server's hostname
/// or IP address and have actions running on git changes immediately.
pub struct HttpTrigger {
    http: String,
}

/// Custom error describing the error cases for the HttpTrigger.
#[derive(Debug, Error)]
pub enum HttpError {
    /// Initializing the HTTP server failed. It usually means the configuration was incorrect
    /// or the port was already allocated.
    #[error("cannot start server on {0}")]
    CantStartServer(String),
    /// Cannot send trigger with Sender. This usually because the receiver is dropped.
    #[error("cannot trigger changes, receiver hang up")]
    ReceiverHangup(#[from] std::sync::mpsc::SendError<Option<Context>>),
    /// Failed to send response.
    #[error("failed to send response")]
    FailedResponse(#[from] std::io::Error),
}

impl From<HttpError> for TriggerError {
    fn from(val: HttpError) -> Self {
        match val {
            HttpError::CantStartServer(s) => TriggerError::Misconfigured(s),
            HttpError::ReceiverHangup(s) => TriggerError::ReceiverHangup(s),
            HttpError::FailedResponse(s) => TriggerError::FailedTrigger(s.to_string()),
        }
    }
}

impl HttpTrigger {
    /// Create an new HTTP trigger with a HTTP url. It accepts an address as a string,
    /// for example "0.0.0.0:1234".
    pub fn new(http: String) -> Self {
        Self { http }
    }

    fn listen_inner(&self, tx: Sender<Option<Context>>) -> Result<(), HttpError> {
        let listener =
            Server::http(&self.http).map_err(|_| HttpError::CantStartServer(self.http.clone()))?;
        info!("Listening on {}...", self.http);
        for request in listener.incoming_requests() {
            debug!("Received request on {} {}", request.method(), request.url());

            let context: Context = HashMap::from([
                ("TRIGGER_NAME", TRIGGER_NAME.to_string()),
                ("HTTP_METHOD", request.method().to_string()),
                ("HTTP_URL", request.url().to_string()),
            ]);
            tx.send(Some(context)).map_err(HttpError::from)?;

            request.respond(Response::from_string("OK"))?;
        }
        Ok(())
    }
}

impl Trigger for HttpTrigger {
    /// Starts a minimal HTTP 1.1 server, that triggers on every request.
    ///
    /// Every method and every URL triggers and returns 200 status code with plaintext "OK".
    fn listen(&self, tx: Sender<Option<Context>>) -> Result<(), TriggerError> {
        self.listen_inner(tx)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        error::Error,
        sync::mpsc,
        thread::{self, sleep},
        time::Duration,
    };

    #[test]
    fn it_should_be_created_from_http_url() {
        let trigger = HttpTrigger::new(String::from("0.0.0.0:1234"));
        assert_eq!("0.0.0.0:1234", &trigger.http);
    }

    #[test]
    fn it_should_return_ok_on_every_request() -> Result<(), Box<dyn Error>> {
        let trigger = HttpTrigger::new(String::from("0.0.0.0:10101"));
        let (tx, rx) = mpsc::channel::<Option<Context>>();

        thread::spawn(move || {
            let _ = trigger.listen_inner(tx);
        });

        // Sleep for the HTTP server to start up.
        sleep(Duration::from_millis(100));

        let result = ureq::get("http://localhost:10101").call()?;
        assert_eq!(200, result.status());
        assert_eq!("OK", result.into_body().read_to_string()?);

        let result = ureq::post("http://localhost:10101/trigger").send_empty()?;
        assert_eq!(200, result.status());
        assert_eq!("OK", result.into_body().read_to_string()?);

        let msg = rx.recv()?;
        let context = msg.unwrap();
        assert_eq!(TRIGGER_NAME, context.get("TRIGGER_NAME").unwrap());
        assert_eq!("GET", context.get("HTTP_METHOD").unwrap());
        assert_eq!("/", context.get("HTTP_URL").unwrap());

        let msg = rx.recv()?;
        let context = msg.unwrap();
        assert_eq!(TRIGGER_NAME, context.get("TRIGGER_NAME").unwrap());
        assert_eq!("POST", context.get("HTTP_METHOD").unwrap());
        assert_eq!("/trigger", context.get("HTTP_URL").unwrap());

        Ok(())
    }

    #[test]
    fn it_should_fail_if_http_url_invalid() {
        let trigger = HttpTrigger::new(String::from("aaaaa"));

        let (tx, _rx) = mpsc::channel::<Option<Context>>();

        let result = trigger.listen_inner(tx);
        assert!(
            matches!(result, Err(HttpError::CantStartServer(_))),
            "{result:?} should be CantStartServer"
        )
    }

    #[test]
    fn it_should_fail_if_sending_fails() -> Result<(), Box<dyn Error>> {
        let trigger = HttpTrigger::new(String::from("0.0.0.0:10102"));
        let (tx, rx) = mpsc::channel::<Option<Context>>();

        thread::spawn(move || {
            // Sleep for the HTTP server to start up.
            sleep(Duration::from_millis(200));

            let _ = ureq::get("http://localhost:10102").call();
        });

        // Drop receiver to create a hangup error
        drop(rx);

        let result = trigger.listen_inner(tx);
        assert!(
            matches!(result, Err(HttpError::ReceiverHangup(_))),
            "{result:?} should be ReceiverHangup"
        );

        Ok(())
    }
}
