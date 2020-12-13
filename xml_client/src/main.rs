use argparse::{ArgumentParser, Store};
mod xml_client;
mod xml_node;
use player::search::Searcher;
use xml_client::XMLClient;

fn main() {
    let mut host = "localhost".to_string();
    let mut port = "13050".to_string();
    let mut reservation = "".to_string();
    let mut time: u128 = 1900;
    let mut verbose: usize = 1;

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
        parser
            .refer(&mut time)
            .add_option(&["-t", "--time"], Store, "Time in milliseconds");
        parser
            .refer(&mut verbose)
            .add_option(&["-v", "--verbose"], Store, "Verbose 0/1/2");
        parser.parse_args_or_exit();
    }

    println!("{}:{} {} {}ms {}", host, port, reservation, time, verbose);
    let mut client = XMLClient::new(host, port, reservation, Searcher::new(time, false, verbose));
    client.run();
    println!("bye");
}
