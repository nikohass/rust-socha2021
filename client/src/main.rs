use argparse::{ArgumentParser, Store};
mod xml_client;
mod xml_node;
use player::search::Searcher;
use xml_client::XMLClient;
mod test_client;
use test_client::run_test_client;

fn main() {
    let mut host = "localhost".to_string();
    let mut port = "13050".to_string();
    let mut reservation = "".to_string();
    let mut time: u128 = 1980;
    let mut test = false;
    let mut weights_file: String = "weights".to_string();

    {
        let mut parser = ArgumentParser::new();
        parser
            .refer(&mut host)
            .add_option(&["-h", "--host"], Store, "Host");
        parser
            .refer(&mut port)
            .add_option(&["-p", "--port"], Store, "Port");
        parser
            .refer(&mut reservation)
            .add_option(&["-r", "--reservation"], Store, "Reservation");
        parser.refer(&mut time).add_option(
            &["-t", "--time"],
            Store,
            "Time per action in milliseconds",
        );
        parser.refer(&mut test).add_option(
            &["-c", "--testclient"],
            Store,
            "Run the test client insetead of the xml client.",
        );
        parser.refer(&mut weights_file).add_option(
            &["-w", "--wfile"],
            Store,
            "File to load neural network weights from",
        );
        parser.parse_args_or_exit();
    }

    println!(
        "Server: {}:{}\nReservation: \"{}\"\nTime/Action: {}ms\nTest: {}\nweights: \"{}\"",
        host, port, reservation, time, test, weights_file
    );
    let searcher = Searcher::new(time, &weights_file);

    if test {
        run_test_client(searcher);
    } else {
        let mut client = XMLClient::new(host, port, reservation, searcher);
        client.run();
    }
}
