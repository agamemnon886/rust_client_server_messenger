use std::thread;
use std::net::{TcpStream};
use std::io::{Write, stdin, stdout, BufReader, BufWriter, BufRead};

fn receive_from_client(mut reader: BufReader<TcpStream>) {
	let mut response = String::new();

	loop {
		reader.read_line(&mut response).expect("Could not read from reader");

		if response.is_empty() {
			println!("Client disconnected");
			break;
		}

		println!("\nClient received [{}] bytes: {}\n",
			response.len(), response.trim());

		response.clear();
	}
}

fn send_to_client(mut writer: BufWriter<TcpStream>) {
	let mut msg = String::new();

	loop {
		print!("Input text: ");
		stdout().flush().expect("Failed to flush stdout");

		stdin().read_line(&mut msg).expect("Failed to read from STDIN");

		writer.write_all(msg.as_bytes()).expect("Could not write to writer");
		writer.flush().expect("Could not flush writer");

		msg.clear();
	}
}

fn main() {
	let reader_stream = TcpStream::connect("127.0.0.1:3333").
		expect("Failed to connect");

	println!("New connection: {}",
		reader_stream.peer_addr().expect("Failed to get peer addr"));
	println!("Client connected\n");

	println!("Successfully connected to server in port 3333\n");

	let writer_stream = reader_stream.try_clone().expect("Cannot clone stream");

	let writer = BufWriter::new(writer_stream);

	let sender = thread::spawn(move || {
		// connection succeeded
		send_to_client(writer)
	});

	let reader = BufReader::new(reader_stream);

	let receiver = thread::spawn(move || {
		// connection succeeded
		receive_from_client(reader)
	});

	sender.join().expect("The sender thread has panicked");
	receiver.join().expect("The receiver thread has panicked");
}
