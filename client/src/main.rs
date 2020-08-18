use std::thread;
use std::net::{TcpStream};
use std::io::{Write, stdin, stdout, BufReader, BufWriter, BufRead};
use crossterm::{  ExecutableCommand, QueueableCommand, terminal, cursor};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::fs::{File, read_to_string};
use chrono::{Local};

fn receive_from_client(tx: Sender<String>, mut reader: BufReader<TcpStream>) {
	let mut response = String::new();

	loop {
		// Receive message from client
		reader.read_line(&mut response).expect("Could not read from reader");

		if response.is_empty() {
			println!("Client disconnected");
			break;
		}

		// Send received message to terminal thread
		tx.send(
			[
				Local::now().format("%v %T").to_string(),
				" | From server |: ".to_string(),
				response.to_string()
			].concat()
		).unwrap();

		response.clear();
	}
}

fn terminal_ui(rx: Receiver<String>)
{
	let mut stdout = stdout();

	// Get full path to current executable file and add ".log" extention
	let path = [
		std::env::current_exe().
			expect("Failed to get file name").
			display().
			to_string(),
		".log".to_string()
	].concat();

	// Create log file or truncate old log file
	let mut file = File::create(&path).expect("Failed to open file");

	// Clear terminal
	stdout.execute(terminal::Clear(terminal::ClearType::All)).
		expect("Execute error");

	// Receive messages from sender and receiver threads and print them
	while let Ok(n) = rx.recv() { // Note: `recv()` always blocks
		let mut output = stdout.lock();

		// Write message into log file
		file.write_all(n.as_bytes()).expect("Failed to write to log file");

		output.queue(cursor::SavePosition).expect("queue error");
		output.flush().expect("Flush error");

		// Always print from cursor position (1,1)
		output.queue(cursor::MoveTo(1, 1)).expect("queue error");
		output.flush().expect("Flush error");

		// Read all lines from log file and print them out
		let str = read_to_string(&path).expect("Failed to read file");
		println!("\n{}\n", str);
		output.flush().expect("Flush error");

		output.queue(cursor::RestorePosition).expect("queue error");
		output.flush().expect("Flush error");
	}
}

fn send_to_client(tx: Sender<String>, mut writer: BufWriter<TcpStream>) {
	let mut stdout = stdout();
	let mut msg = String::new();

	// Get number of terminal rows
	let (_, rows) = terminal::size().expect("Failed to get terminal size");
	stdout.execute(terminal::Clear(terminal::ClearType::All)).
		expect("Execute error");

	// Input message to send it to server
	loop {
		// Input prompt should always be at the bottom of the terminal window
		stdout.queue(cursor::MoveTo(1, rows)).expect("queue error");
		stdout.flush().expect("Flush error");

		print!("Input text : ");
		stdout.flush().expect("Failed to flush output");

		// Read line from STDIN
		stdin().read_line(&mut msg).expect("Failed to read from STDIN");

		// Remove everything before current cursor position
		stdout.execute(terminal::Clear(terminal::ClearType::FromCursorUp)).
			expect("Execute error");
		stdout.flush().expect("Failed to flush output");

		// Send message with date into terminal UI thread
		tx.send(
			[
				Local::now().format("%v %T").to_string(),
				" | From Me |: ".to_string(),
				msg.to_string()
			].concat()
		).unwrap();

		// Send message to client
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

	let (tx_sender, rx_ui) = mpsc::channel();

	let terminal = thread::spawn(move || {
		// connection succeeded
		terminal_ui(rx_ui);
	});

	let tx_receiver = tx_sender.clone();

	let writer_stream = reader_stream.try_clone().
		expect("Cannot clone stream");

	let writer = BufWriter::new(writer_stream);

	let sender = thread::spawn(move || {
		// connection succeeded
		send_to_client(tx_sender, writer)
	});

	let reader = BufReader::new(reader_stream);

	let receiver = thread::spawn(move|| {
		// connection succeeded
		receive_from_client(tx_receiver, reader)
	});

	sender.join().expect("The sender thread has panicked");
	receiver.join().expect("The receiver thread has panicked");
	terminal.join().expect("The terminal UI thread has panicked");
}

