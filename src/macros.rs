// use chrono::offset::Local;

//println with formatted time stamp included
#[macro_export]
macro_rules! printfmt {
    // take a token tree as input
    ($($arg:tt)*) => {
        // Print the timestamp
        print!("{}: ", chrono::offset::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"));
        // Print the arguments passed
        println!($($arg)*);

    };

}

#[macro_export]
macro_rules! sendmsg {
    // take an expression as input
    ($t:expr, $i:expr) => {
        // boilerplate to send single instructions to the channel
        {
            let _tx = $t.clone();
            let join_handle = thread::spawn(move || {
                let instr = $i;
                let _ = _tx.send(instr).unwrap();
            });
            join_handle.join().unwrap();
        }
    };
}
