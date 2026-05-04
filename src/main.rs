use std::io;

use kintoun::repl;
use kintoun::storage::InMemoryStorage;

fn main() -> io::Result<()> {
    let stdin = io::stdin().lock();
    let stdout = io::stdout().lock();
    let mut storage = InMemoryStorage::new();
    repl::run(stdin, stdout, &mut storage)
}
