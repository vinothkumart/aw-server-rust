extern crate aw_client_rust;
extern crate aw_datastore;
extern crate aw_server;
extern crate chrono;
extern crate serde_json;

#[cfg(test)]
mod test {
    use aw_client_rust::AwClient;
    use aw_client_rust::Event;
    use chrono::{Duration, Utc};
    use serde_json::Map;
    use std::path::PathBuf;
    use std::sync::Mutex;
    use std::thread;

    // A random port, but still not guaranteed to not be bound
    // FIXME: Bind to a port that is free for certain and use that for the client instead
    static PORT: u16 = 41293;

    fn wait_for_server(timeout: i64, client: &AwClient) -> () {
        // Wait for server to come online
        let start = Utc::now();
        loop {
            match client.get_info() {
                Ok(_) => break,
                Err(err) => {
                    let passed = Utc::now() - start;
                    if passed >= chrono::Duration::seconds(timeout) {
                        panic!("Timed out starting aw-server after {}s: {:?}", timeout, err);
                    }
                }
            }
            use std::time;
            let duration = time::Duration::from_secs(1);
            thread::sleep(duration);
        }
    }

    fn setup_testserver() -> () {
        // Start testserver
        // TODO: Properly shutdown
        use aw_server::endpoints::ServerState;
        let state = ServerState {
            datastore: Mutex::new(aw_datastore::Datastore::new_in_memory(false)),
            asset_path: PathBuf::from("."), // webui won't be used, so it's invalidly set
        };
        let mut aw_config = aw_server::config::AWConfig::default();
        aw_config.port = PORT;
        let server = aw_server::endpoints::build_rocket(state, aw_config);

        thread::spawn(move || {
            server.launch();
        });
    }

    #[test]
    fn test_full() {
        // Set to true to use a temporary test server
        // Set to false to use your local testing instance (port 5666)
        let temp_server = true;

        let ip = "127.0.0.1";
        let port: String = if temp_server {
            PORT.to_string()
        } else {
            "5666".to_string()
        };

        let clientname = "aw-client-rust-test";
        let client: AwClient = AwClient::new(ip, &port, clientname);

        setup_testserver();

        wait_for_server(20, &client);

        let info = client.get_info().unwrap();
        assert!(info.testing == true);

        let bucketname = format!("aw-client-rust-test_{}", client.hostname);
        let buckettype = "test-type";
        client.create_bucket(&bucketname, &buckettype).unwrap();

        let bucket = client.get_bucket(&bucketname).unwrap();
        assert!(bucket.id == bucketname);
        println!("{}", bucket.id);

        let buckets = client.get_buckets().unwrap();
        println!("Buckets: {:?}", buckets);

        let start = Utc::now();
        let end = start + Duration::seconds(1);
        let mut event = Event {
            id: None,
            timestamp: start,
            duration: Duration::seconds(0),
            data: Map::new(),
        };
        println!("{:?}", event);
        client.insert_event(&bucketname, &event).unwrap();

        event.timestamp = end;
        client.heartbeat(&bucketname, &event, 10.0).unwrap();

        let events = client
            .get_events(&bucketname, Some(start), Some(end), None)
            .unwrap();
        println!("Events: {:?}", events);
        assert!(events[0].duration == Duration::seconds(1));

        client
            .delete_event(&bucketname, events[0].id.unwrap())
            .unwrap();

        let count = client
            .get_event_count(&bucketname, Some(start), Some(end))
            .unwrap();
        assert_eq!(count, 0);

        client.delete_bucket(&bucketname).unwrap();
    }
}
