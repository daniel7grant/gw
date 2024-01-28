use super::Trigger;
use crate::Result;
use std::sync::mpsc::Sender;
use tiny_http::{Response, Server};

/// A trigger that runs on an HTTP request.
/// 
/// This could be used to trigger checks from git remotes (e.g. GitHub, GitLab) with webhooks.
/// Given that your server can be reached from the outside, you can pass your server's hostname
/// or IP address and have actions running on git changes immediately.
pub struct HttpTrigger {
    http: String,
}

impl HttpTrigger {
    /// Create an new HTTP trigger with a HTTP url. It accepts an address as a string,
	/// for example "1234" or "0.0.0.0:1234".
    pub fn new(http: String) -> Self {
        Self { http }
    }
}

impl Trigger for HttpTrigger {
	/// Starts a minimal HTTP 1.1 server, that triggers on every request.
	/// 
	/// Every method and every URL returns 200 status code with plaintext "OK".
    fn listen(&self, tx: Sender<Option<()>>) -> Result<()> {
        let listener = Server::http(&self.http)
            .map_err(|_| format!("Cannot start server on {}", self.http))?;
        println!("Listening on {}...", self.http);
        for request in listener.incoming_requests() {
            println!("Received request on {} {}", request.method(), request.url());

            tx.send(Some(()))?;

            request.respond(Response::from_string("OK"))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::mpsc, thread};

    use super::*;

    #[test]
    fn it_should_be_created_from_http_url() {
        let trigger = HttpTrigger::new(String::from("0.0.0.0:1234"));
        assert_eq!("0.0.0.0:1234", &trigger.http);
    }

    #[test]
    fn it_should_return_ok_on_every_request() -> Result<()> {
        let trigger = HttpTrigger::new(String::from("0.0.0.0:1234"));
        let (tx, rx) = mpsc::channel::<Option<()>>();

        thread::spawn(move || {
            let _ = trigger.listen(tx);
        });

        let result = ureq::get("http://localhost:1234").call()?;
        assert_eq!(200, result.status());
        assert_eq!("OK", result.into_string()?);

        let result = ureq::post("http://localhost:1234/trigger").call()?;
        assert_eq!(200, result.status());
        assert_eq!("OK", result.into_string()?);

        let msg = rx.recv()?;
        assert_eq!(Some(()), msg);

        let msg = rx.recv()?;
        assert_eq!(Some(()), msg);

        Ok(())
    }
}
