use super::Trigger;
use crate::Result as GwResult;
use std::sync::mpsc::Sender;
use thiserror::Error;
use tiny_http::{Response, Server};

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
    ReceiverHangup(#[from] std::sync::mpsc::SendError<Option<()>>),
    /// Failed to send response.
    #[error("failed to send response")]
    ResponseFailed(#[from] std::io::Error),
}

impl HttpTrigger {
    /// Create an new HTTP trigger with a HTTP url. It accepts an address as a string,
    /// for example "0.0.0.0:1234".
    pub fn new(http: String) -> Self {
        Self { http }
    }

    fn listen_inner(&self, tx: Sender<Option<()>>) -> Result<(), HttpError> {
        let listener =
            Server::http(&self.http).map_err(|_| HttpError::CantStartServer(self.http.clone()))?;
        println!("Listening on {}...", self.http);
        for request in listener.incoming_requests() {
            println!("Received request on {} {}", request.method(), request.url());

            tx.send(Some(())).map_err(HttpError::from)?;

            request.respond(Response::from_string("OK"))?;
        }
        Ok(())
    }
}

impl Trigger for HttpTrigger {
    /// Starts a minimal HTTP 1.1 server, that triggers on every request.
    ///
    /// Every method and every URL triggers and returns 200 status code with plaintext "OK".
    fn listen(&self, tx: Sender<Option<()>>) -> GwResult<()> {
        self.listen_inner(tx)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::mpsc,
        thread::{self, sleep},
        time::Duration,
    };

    use super::*;

    #[test]
    fn it_should_be_created_from_http_url() {
        let trigger = HttpTrigger::new(String::from("0.0.0.0:1234"));
        assert_eq!("0.0.0.0:1234", &trigger.http);
    }

    #[test]
    fn it_should_return_ok_on_every_request() -> GwResult<()> {
        let trigger = HttpTrigger::new(String::from("0.0.0.0:10101"));
        let (tx, rx) = mpsc::channel::<Option<()>>();

        thread::spawn(move || {
            let _ = trigger.listen_inner(tx);
        });

        // Sleep for the HTTP server to start up.
        sleep(Duration::from_millis(100));

        let result = ureq::get("http://localhost:10101").call()?;
        assert_eq!(200, result.status());
        assert_eq!("OK", result.into_string()?);

        let result = ureq::post("http://localhost:10101/trigger").call()?;
        assert_eq!(200, result.status());
        assert_eq!("OK", result.into_string()?);

        let msg = rx.recv()?;
        assert_eq!(Some(()), msg);

        let msg = rx.recv()?;
        assert_eq!(Some(()), msg);

        Ok(())
    }

    #[test]
    fn it_should_fail_if_http_url_invalid() {
        let trigger = HttpTrigger::new(String::from("aaaaa"));

        let (tx, _rx) = mpsc::channel::<Option<()>>();

        let result = trigger.listen_inner(tx);
        assert!(
            matches!(result, Err(HttpError::CantStartServer(_))),
            "{result:?} should be CantStartServer"
        )
    }

    #[test]
    fn it_should_fail_if_sending_fails() -> GwResult<()> {
        let trigger = HttpTrigger::new(String::from("0.0.0.0:10102"));
        let (tx, rx) = mpsc::channel::<Option<()>>();

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
